use anyhow::Result;
use rusqlite::{params, Connection, Row};
use serde::Serialize;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Serialize)]
pub struct HistoryItem {
    pub id: i64,
    pub timestamp: i64,
    pub command: String,
    pub cwd: String,
    pub exit_code: Option<i32>,
    pub duration: Option<i64>,
}

fn db_path() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("cannot find home dir"))?;
    let dir = home.join(".hindsight");
    std::fs::create_dir_all(&dir)?;
    Ok(dir.join("history.db"))
}

pub fn open() -> Result<Connection> {
    let conn = Connection::open(db_path()?)?;
    conn.busy_timeout(Duration::from_secs(2))?;
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;",
    )?;
    init_schema(&conn)?;
    Ok(conn)
}

fn init_schema(conn: &Connection) -> Result<()> {
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

pub fn search(conn: &Connection, query: &str, limit: usize) -> Result<Vec<HistoryItem>> {
    let like_pattern = format!("%{}%", query);
    let mut stmt = conn.prepare(
        "SELECT id, timestamp, command, cwd, exit_code, duration
         FROM history
         WHERE command LIKE ?1 COLLATE NOCASE OR cwd LIKE ?1 COLLATE NOCASE
         ORDER BY timestamp DESC, id DESC
         LIMIT ?2",
    )?;

    let rows = stmt.query_map(params![like_pattern, limit as i64], item_from_row)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn recent(conn: &Connection, limit: usize) -> Result<Vec<HistoryItem>> {
    let mut stmt = conn.prepare(
        "SELECT id, timestamp, command, cwd, exit_code, duration
         FROM history
         ORDER BY timestamp DESC, id DESC
         LIMIT ?1",
    )?;

    let rows = stmt.query_map([limit as i64], item_from_row)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
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
    use super::{clear_history, init_schema};
    use rusqlite::{params, Connection};

    #[test]
    fn clear_history_removes_rows_and_search_index() {
        let conn = Connection::open_in_memory().expect("open in-memory database");
        init_schema(&conn).expect("initialize schema");
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
}
