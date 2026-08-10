use anyhow::Result;
use rusqlite::Connection;

pub struct Stats {
    pub total: usize,
    pub unique: usize,
    pub sessions: usize,
}

pub fn search(
    conn: &Connection,
    query: &str,
    cwd: Option<&str>,
    exit_code: Option<i32>,
    limit: usize,
) -> Result<Vec<String>> {
    let mut sql = String::from(
        "SELECT h.command, h.cwd, h.exit_code, h.timestamp
         FROM history_fts f
         JOIN history h ON f.rowid = h.id
         WHERE history_fts MATCH ?1",
    );

    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(query.to_string())];
    let mut param_idx = 2;

    if let Some(cwd_filter) = cwd {
        sql.push_str(&format!(" AND h.cwd LIKE ?{}", param_idx));
        params.push(Box::new(format!("{}%", cwd_filter)));
        param_idx += 1;
    }

    if let Some(exit) = exit_code {
        sql.push_str(&format!(" AND h.exit_code = ?{}", param_idx));
        params.push(Box::new(exit));
        param_idx += 1;
    }

    sql.push_str(" ORDER BY h.timestamp DESC");
    sql.push_str(&format!(" LIMIT ?{}", param_idx));
    params.push(Box::new(limit as i64));

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(param_refs.as_slice(), |row| {
        let command: String = row.get(0)?;
        let cwd: String = row.get(1)?;
        let exit_code: Option<i32> = row.get(2)?;
        let timestamp: i64 = row.get(3)?;

        let dt = chrono::DateTime::from_timestamp(timestamp, 0)
            .map(|d| {
                d.with_timezone(&chrono::Local)
                    .format("%Y-%m-%d %H:%M")
                    .to_string()
            })
            .unwrap_or_default();

        let exit_str = match exit_code {
            Some(0) => "✓".to_string(),
            Some(e) => format!("✗{}", e),
            None => "?".to_string(),
        };

        Ok(format!(
            "{:>16}  {:>3}  {:40}  {}",
            dt,
            exit_str,
            truncate(&command, 40),
            cwd
        ))
    })?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }

    Ok(results)
}

pub fn recent(conn: &Connection, limit: usize) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT command, cwd, exit_code, timestamp FROM history ORDER BY timestamp DESC LIMIT ?1",
    )?;

    let rows = stmt.query_map([limit], |row| {
        let command: String = row.get(0)?;
        let cwd: String = row.get(1)?;
        let exit_code: Option<i32> = row.get(2)?;
        let timestamp: i64 = row.get(3)?;

        let dt = chrono::DateTime::from_timestamp(timestamp, 0)
            .map(|d| {
                d.with_timezone(&chrono::Local)
                    .format("%Y-%m-%d %H:%M")
                    .to_string()
            })
            .unwrap_or_default();

        let exit_str = match exit_code {
            Some(0) => "✓".to_string(),
            Some(e) => format!("✗{}", e),
            None => "?".to_string(),
        };

        Ok(format!(
            "{:>16}  {:>3}  {:40}  {}",
            dt,
            exit_str,
            truncate(&command, 40),
            cwd
        ))
    })?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }

    Ok(results)
}

pub fn top(conn: &Connection, n: usize) -> Result<Vec<(String, usize)>> {
    let mut stmt = conn.prepare(
        "SELECT command, COUNT(*) as cnt FROM history GROUP BY command ORDER BY cnt DESC LIMIT ?1",
    )?;

    let rows = stmt.query_map([n], |row| {
        let command: String = row.get(0)?;
        let count: usize = row.get(1)?;
        Ok((command, count))
    })?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }

    Ok(results)
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

fn truncate(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        let visible_len = max_len.saturating_sub(3);
        format!("{}...", s.chars().take(visible_len).collect::<String>())
    }
}

#[cfg(test)]
mod tests {
    use super::truncate;

    #[test]
    fn truncate_preserves_utf8_boundaries() {
        assert_eq!(truncate("你好世界abc", 6), "你好世...");
    }

    #[test]
    fn truncate_keeps_short_commands_intact() {
        assert_eq!(truncate("git status", 40), "git status");
    }
}
