use anyhow::Result;
use rusqlite::{Connection, params};
use std::path::PathBuf;

fn db_path() -> PathBuf {
    let home = dirs::home_dir().expect("cannot find home dir");
    let dir = home.join(".hindsight");
    std::fs::create_dir_all(&dir).ok();
    dir.join("history.db")
}

pub fn open() -> Result<Connection> {
    let conn = Connection::open(db_path())?;
    conn.execute_batch("
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;
    ")?;
    init_schema(&conn)?;
    Ok(conn)
}

fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS history (
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
    ")?;

    // FTS5 virtual table
    conn.execute_batch("
        CREATE VIRTUAL TABLE IF NOT EXISTS history_fts USING fts5(
            command,
            cwd,
            content=history,
            content_rowid=id
        );
    ")?;

    // Triggers to keep FTS in sync
    conn.execute_batch("
        CREATE TRIGGER IF NOT EXISTS history_ai AFTER INSERT ON history BEGIN
            INSERT INTO history_fts(rowid, command, cwd) VALUES (new.id, new.command, new.cwd);
        END;

        CREATE TRIGGER IF NOT EXISTS history_ad AFTER DELETE ON history BEGIN
            INSERT INTO history_fts(history_fts, rowid, command, cwd) VALUES('delete', old.id, old.command, old.cwd);
        END;

        CREATE TRIGGER IF NOT EXISTS history_au AFTER UPDATE ON history BEGIN
            INSERT INTO history_fts(history_fts, rowid, command, cwd) VALUES('delete', old.id, old.command, old.cwd);
            INSERT INTO history_fts(rowid, command, cwd) VALUES (new.id, new.command, new.cwd);
        END;
    ")?;

    Ok(())
}

pub fn insert(conn: &Connection, command: &str, cwd: &str, exit_code: Option<i32>, duration: Option<u64>) -> Result<()> {
    let now = chrono::Utc::now().timestamp();
    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_default();

    conn.execute(
        "INSERT INTO history (timestamp, command, cwd, exit_code, duration, session_id, hostname)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![now, command, cwd, exit_code, duration.map(|d| d as i64), "", hostname],
    )?;

    Ok(())
}
