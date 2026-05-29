use anyhow::Result;
use rusqlite::{Connection, params};
use std::path::PathBuf;

fn db_path() -> PathBuf {
    let home = dirs::home_dir().expect("cannot find home dir");
    home.join(".hindsight").join("history.db")
}

pub fn open() -> Result<Connection> {
    let path = db_path();
    let conn = Connection::open(&path)?;
    conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL;")?;
    Ok(conn)
}

pub fn search(conn: &Connection, query: &str, limit: usize) -> Result<Vec<String>> {
    let like_pattern = format!("%{}%", query);
    let mut stmt = conn.prepare(
        "SELECT command, cwd, exit_code, timestamp
         FROM history
         WHERE command LIKE ?1 OR cwd LIKE ?1
         ORDER BY timestamp DESC
         LIMIT ?2"
    )?;

    let rows = stmt.query_map(params![like_pattern, limit as i64], |row| {
        let command: String = row.get(0)?;
        let cwd: String = row.get(1)?;
        let exit_code: Option<i32> = row.get(2)?;
        let timestamp: i64 = row.get(3)?;
        Ok(format_row(&command, &cwd, exit_code, timestamp))
    })?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

pub fn recent(conn: &Connection, limit: usize) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT command, cwd, exit_code, timestamp FROM history ORDER BY timestamp DESC LIMIT ?1"
    )?;

    let rows = stmt.query_map([limit], |row| {
        let command: String = row.get(0)?;
        let cwd: String = row.get(1)?;
        let exit_code: Option<i32> = row.get(2)?;
        let timestamp: i64 = row.get(3)?;
        Ok(format_row(&command, &cwd, exit_code, timestamp))
    })?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

fn format_row(command: &str, cwd: &str, exit_code: Option<i32>, timestamp: i64) -> String {
    let dt = chrono::DateTime::from_timestamp(timestamp, 0)
        .map(|d| d.with_timezone(&chrono::Local).format("%m-%d %H:%M").to_string())
        .unwrap_or_default();

    let exit_str = match exit_code {
        Some(0) => "✓".to_string(),
        Some(-1) | None => "".to_string(),
        Some(_) => "✗".to_string(),
    };

    // Show ~ instead of full home path
    let home = dirs::home_dir().map(|h| h.to_string_lossy().to_string()).unwrap_or_default();
    let cwd_display = cwd.replace(&home, "~");

    format!("{}\t{}\t{}\t{}", dt, exit_str, command, cwd_display)
}
