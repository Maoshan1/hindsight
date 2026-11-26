use anyhow::Result;
use rusqlite::{params, Connection, Row};
use serde::Serialize;
use std::path::PathBuf;
use std::time::Duration;

use crate::{sanitize, settings};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HistoryItem {
    pub id: i64,
    pub timestamp: i64,
    pub command: String,
    pub cwd: String,
    pub exit_code: Option<i32>,
    pub duration: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stats {
    pub total: usize,
    pub unique: usize,
    pub sessions: usize,
}

#[derive(Debug, Clone, Default)]
pub struct SearchOptions<'a> {
    pub query: &'a str,
    pub cwd: Option<&'a str>,
    pub exit_code: Option<i32>,
    pub failed_only: bool,
    pub limit: usize,
}

fn db_path() -> Result<PathBuf> {
    Ok(settings::data_dir()?.join("history.db"))
}

pub fn open() -> Result<Connection> {
    let conn = Connection::open(db_path()?)?;
    configure_connection(&conn)?;
    init_schema(&conn)?;
    Ok(conn)
}

pub fn configure_connection(conn: &Connection) -> Result<()> {
    conn.busy_timeout(Duration::from_secs(2))?;
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;",
    )?;
    Ok(())
}

pub fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS history (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp   INTEGER NOT NULL,
            command     TEXT NOT NULL,
            cwd         TEXT NOT NULL,
            exit_code   INTEGER,
            duration    INTEGER,
            session_id  TEXT NOT NULL DEFAULT '',
            hostname    TEXT NOT NULL DEFAULT ''
        );

        CREATE INDEX IF NOT EXISTS idx_history_timestamp ON history(timestamp);
        CREATE INDEX IF NOT EXISTS idx_history_cwd ON history(cwd);
        CREATE INDEX IF NOT EXISTS idx_history_exit_code ON history(exit_code);

        CREATE VIRTUAL TABLE IF NOT EXISTS history_fts USING fts5(
            command,
            cwd,
            content=history,
            content_rowid=id
        );

        CREATE TRIGGER IF NOT EXISTS history_ai AFTER INSERT ON history BEGIN
            INSERT INTO history_fts(rowid, command, cwd) VALUES (new.id, new.command, new.cwd);
        END;

        CREATE TRIGGER IF NOT EXISTS history_ad AFTER DELETE ON history BEGIN
            INSERT INTO history_fts(history_fts, rowid, command, cwd) VALUES('delete', old.id, old.command, old.cwd);
        END;

        CREATE TRIGGER IF NOT EXISTS history_au AFTER UPDATE ON history BEGIN
            INSERT INTO history_fts(history_fts, rowid, command, cwd) VALUES('delete', old.id, old.command, old.cwd);
            INSERT INTO history_fts(rowid, command, cwd) VALUES (new.id, new.command, new.cwd);
        END;",
    )?;

    Ok(())
}

pub fn insert(
    conn: &Connection,
    command: &str,
    cwd: &str,
    exit_code: Option<i32>,
    duration: Option<u64>,
) -> Result<()> {
    let current_settings = settings::load_or_default();
    insert_with_settings(conn, command, cwd, exit_code, duration, &current_settings)
}

pub fn insert_with_settings(
    conn: &Connection,
    command: &str,
    cwd: &str,
    exit_code: Option<i32>,
    duration: Option<u64>,
    current_settings: &settings::Settings,
) -> Result<()> {
    if !current_settings.recording_enabled {
        return Ok(());
    }

    let Some(sanitized) = sanitize::prepare_command(command) else {
        return Ok(());
    };

    let now = chrono::Utc::now().timestamp();
    let host = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_default();

    conn.execute(
        "INSERT INTO history (timestamp, command, cwd, exit_code, duration, session_id, hostname)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            now,
            sanitized,
            cwd,
            exit_code,
            duration.map(|d| d as i64),
            "",
            host
        ],
    )?;

    prune(conn, current_settings.retention_days)?;
    Ok(())
}

fn item_from_row(row: &Row<'_>) -> rusqlite::Result<HistoryItem> {
    Ok(HistoryItem {
        id: row.get(0)?,
        timestamp: row.get(1)?,
        command: row.get(2)?,
        cwd: row.get(3)?,
        exit_code: row.get(4)?,
        duration: row.get(5)?,
    })
}

fn escape_like(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\\' | '%' | '_' => {
                escaped.push('\\');
                escaped.push(ch);
            }
            _ => escaped.push(ch),
        }
    }
    escaped
}

pub fn search(conn: &Connection, options: SearchOptions<'_>) -> Result<Vec<HistoryItem>> {
    let mut sql = String::from(
        "SELECT id, timestamp, command, cwd, exit_code, duration
         FROM history
         WHERE 1=1",
    );

    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    let mut param_idx = 1;

    for term in options.query.split_whitespace() {
        sql.push_str(&format!(
            " AND (command LIKE ?{param_idx} ESCAPE '\\' COLLATE NOCASE OR cwd LIKE ?{param_idx} ESCAPE '\\' COLLATE NOCASE)"
        ));
        params.push(Box::new(format!("%{}%", escape_like(term))));
        param_idx += 1;
    }

    if let Some(cwd_filter) = options.cwd.filter(|c| !c.is_empty()) {
        sql.push_str(&format!(" AND cwd LIKE ?{param_idx} ESCAPE '\\'"));
        params.push(Box::new(format!("{}%", escape_like(cwd_filter))));
        param_idx += 1;
    }

    if let Some(exit) = options.exit_code {
        sql.push_str(&format!(" AND exit_code = ?{param_idx}"));
        params.push(Box::new(exit));
        param_idx += 1;
    } else if options.failed_only {
        sql.push_str(" AND exit_code IS NOT NULL AND exit_code != 0");
    }

    let limit = options.limit.max(1);
    sql.push_str(&format!(" ORDER BY timestamp DESC, id DESC LIMIT ?{param_idx}"));
    params.push(Box::new(limit as i64));

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(param_refs.as_slice(), item_from_row)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn recent(conn: &Connection, limit: usize) -> Result<Vec<HistoryItem>> {
    search(
        conn,
        SearchOptions {
            limit,
            ..Default::default()
        },
    )
}

pub fn top(conn: &Connection, n: usize) -> Result<Vec<(String, usize)>> {
    let mut stmt = conn.prepare(
        "SELECT command, COUNT(*) as cnt FROM history GROUP BY command ORDER BY cnt DESC LIMIT ?1",
    )?;

    let rows = stmt.query_map([n as i64], |row| {
        let command: String = row.get(0)?;
        let count: usize = row.get(1)?;
        Ok((command, count))
    })?;

    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn stats(conn: &Connection) -> Result<Stats> {
    let total: usize = conn.query_row("SELECT COUNT(*) FROM history", [], |r| r.get(0))?;
    let unique: usize = conn.query_row("SELECT COUNT(DISTINCT command) FROM history", [], |r| {
        r.get(0)
    })?;
    let sessions: usize = conn.query_row(
        "SELECT COUNT(DISTINCT session_id) FROM history WHERE session_id != ''",
        [],
        |r| r.get(0),
    )?;

    Ok(Stats {
        total,
        unique,
        sessions,
    })
}

pub fn clear_history(conn: &Connection) -> Result<usize> {
    let tx = conn.unchecked_transaction()?;
    let removed: usize = tx.query_row("SELECT COUNT(*) FROM history", [], |row| row.get(0))?;
    tx.execute("DELETE FROM history", [])?;
    tx.execute("INSERT INTO history_fts(history_fts) VALUES('rebuild')", [])?;
    tx.commit()?;
    Ok(removed)
}

pub fn prune(conn: &Connection, retention_days: Option<u32>) -> Result<()> {
    let Some(days) = retention_days else {
        return Ok(());
    };

    let cutoff = chrono::Utc::now().timestamp() - i64::from(days) * 86_400;
    conn.execute("DELETE FROM history WHERE timestamp < ?1", [cutoff])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        clear_history, init_schema, insert_with_settings, prune, recent, search, SearchOptions,
    };
    use crate::settings::Settings;
    use rusqlite::{params, Connection};

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().expect("open in-memory database");
        init_schema(&conn).expect("initialize schema");
        conn
    }

    #[test]
    fn clear_history_removes_rows_and_search_index() {
        let conn = setup_db();
        conn.execute(
            "INSERT INTO history (timestamp, command, cwd) VALUES (?1, ?2, ?3)",
            params![1_i64, "git status", "/tmp"],
        )
        .expect("insert history row");

        assert_eq!(clear_history(&conn).expect("clear history"), 1);
        let remaining: i64 = conn
            .query_row("SELECT COUNT(*) FROM history", [], |row| row.get(0))
            .expect("count history rows");
        let indexed: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM history_fts WHERE history_fts MATCH 'git'",
                [],
                |row| row.get(0),
            )
            .expect("count indexed rows");

        assert_eq!(remaining, 0);
        assert_eq!(indexed, 0);
    }

    #[test]
    fn search_supports_multi_terms_special_chars_and_failed_filter() {
        let conn = setup_db();
        let active = Settings::default();
        insert_with_settings(
            &conn,
            "ffmpeg -i input.mp4 -vf scale=1280:720 output.mp4",
            "/Users/demo/videos",
            Some(0),
            Some(120),
            &active,
        )
        .unwrap();
        insert_with_settings(
            &conn,
            "docker-compose up --build -d 100%_ready",
            "/Users/demo/app",
            Some(1),
            Some(50),
            &active,
        )
        .unwrap();
        insert_with_settings(
            &conn,
            "git status",
            "/Users/demo/app",
            None,
            None,
            &active,
        )
        .unwrap();

        let multi = search(
            &conn,
            SearchOptions {
                query: "ffmpeg scale",
                limit: 10,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(multi.len(), 1);
        assert!(multi[0].command.contains("scale=1280:720"));

        let special = search(
            &conn,
            SearchOptions {
                query: "docker-compose 100%_",
                limit: 10,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(special.len(), 1);
        assert!(special[0].command.starts_with("docker-compose"));

        let failed = search(
            &conn,
            SearchOptions {
                query: "",
                failed_only: true,
                limit: 10,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(failed.len(), 1);
        assert_eq!(failed[0].exit_code, Some(1));
    }

    #[test]
    fn insert_respects_recording_flag_leading_space_and_retention() {
        let conn = setup_db();
        let paused = Settings {
            recording_enabled: false,
            retention_days: None,
        };
        insert_with_settings(&conn, "ls", "/tmp", Some(0), None, &paused).unwrap();
        assert!(recent(&conn, 10).unwrap().is_empty());

        let active = Settings::default();
        insert_with_settings(&conn, " echo secret", "/tmp", Some(0), None, &active).unwrap();
        assert!(recent(&conn, 10).unwrap().is_empty());

        let old_ts = chrono::Utc::now().timestamp() - 10 * 86_400;
        conn.execute(
            "INSERT INTO history (timestamp, command, cwd) VALUES (?1, ?2, ?3)",
            params![old_ts, "old command", "/tmp"],
        )
        .unwrap();
        prune(&conn, Some(7)).unwrap();
        assert!(recent(&conn, 10).unwrap().is_empty());
    }
}
