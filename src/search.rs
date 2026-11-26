use anyhow::Result;
use hindsight_core::db::{self, HistoryItem, SearchOptions, Stats};
use rusqlite::Connection;

pub fn search(
    conn: &Connection,
    query: &str,
    cwd: Option<&str>,
    exit_code: Option<i32>,
    failed_only: bool,
    limit: usize,
) -> Result<Vec<HistoryItem>> {
    db::search(
        conn,
        SearchOptions {
            query,
            cwd,
            exit_code,
            failed_only,
            limit,
        },
    )
}

pub fn recent(conn: &Connection, limit: usize) -> Result<Vec<HistoryItem>> {
    db::recent(conn, limit)
}

pub fn top(conn: &Connection, n: usize) -> Result<Vec<(String, usize)>> {
    db::top(conn, n)
}

pub fn stats(conn: &Connection) -> Result<Stats> {
    db::stats(conn)
}

pub fn format_item(item: &HistoryItem) -> String {
    let dt = chrono::DateTime::from_timestamp(item.timestamp, 0)
        .map(|d| {
            d.with_timezone(&chrono::Local)
                .format("%Y-%m-%d %H:%M")
                .to_string()
        })
        .unwrap_or_default();

    let exit_str = match item.exit_code {
        Some(0) => "✓".to_string(),
        Some(e) => format!("✗{}", e),
        None => "?".to_string(),
    };

    format!(
        "{:>16}  {:>3}  {:40}  {}",
        dt,
        exit_str,
        truncate(&item.command, 40),
        item.cwd
    )
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
    use super::{format_item, truncate};
    use hindsight_core::db::HistoryItem;

    #[test]
    fn truncate_preserves_utf8_boundaries() {
        assert_eq!(truncate("你好世界abc", 6), "你好世...");
    }

    #[test]
    fn truncate_keeps_short_commands_intact() {
        assert_eq!(truncate("git status", 40), "git status");
    }

    #[test]
    fn format_item_renders_status_and_cwd() {
        let item = HistoryItem {
            id: 1,
            timestamp: 0,
            command: "git status".to_string(),
            cwd: "/tmp".to_string(),
            exit_code: Some(0),
            duration: Some(12),
        };
        let rendered = format_item(&item);
        assert!(rendered.contains("✓"));
        assert!(rendered.contains("git status"));
        assert!(rendered.contains("/tmp"));
    }
}
