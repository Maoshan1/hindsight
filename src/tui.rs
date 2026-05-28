use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::io;

use crate::db;

pub fn run(initial_query: &str) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let conn = db::open()?;
    let mut query = initial_query.to_string();
    let mut results = do_search(&conn, &query)?;
    let mut selected: usize = 0;

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(1),
                ])
                .split(f.size());

            // Search input
            let input = Paragraph::new(Line::from(vec![
                Span::styled(" > ", Style::default().fg(Color::Yellow)),
                Span::raw(&query),
            ]))
            .block(Block::default().borders(Borders::ALL).title("hindsight"));

            f.render_widget(input, chunks[0]);

            // Results list
            let items: Vec<ListItem> = results
                .iter()
                .enumerate()
                .map(|(i, r)| {
                    let style = if i == selected {
                        Style::default().fg(Color::Black).bg(Color::White)
                    } else {
                        Style::default()
                    };
                    ListItem::new(Line::from(Span::styled(r.clone(), style)))
                })
                .collect();

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title(format!(
                    "results ({})",
                    results.len()
                )));

            f.render_widget(list, chunks[1]);
        })?;

        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Esc => break,
                    KeyCode::Enter => {
                        // Print selected command to stdout and exit
                        if let Some(result) = results.get(selected) {
                            // Extract just the command part (4th field)
                            let cmd = extract_command(result);
                            disable_raw_mode()?;
                            execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
                            print!("{}", cmd);
                            return Ok(());
                        }
                        break;
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        if selected > 0 {
                            selected -= 1;
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if selected < results.len().saturating_sub(1) {
                            selected += 1;
                        }
                    }
                    KeyCode::Char(c) => {
                        query.push(c);
                        results = do_search(&conn, &query)?;
                        selected = 0;
                    }
                    KeyCode::Backspace => {
                        query.pop();
                        results = do_search(&conn, &query)?;
                        selected = 0;
                    }
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

fn do_search(conn: &rusqlite::Connection, query: &str) -> Result<Vec<String>> {
    if query.is_empty() {
        crate::search::recent(conn, 50)
    } else {
        crate::search::search(conn, query, None, None, 50)
    }
}

fn extract_command(formatted: &str) -> String {
    // Format: "  2026-05-28 06:35    ✓  git status                                /Users/yyx/projects"
    // We want just "git status"
    // Split by double-space sequences and take the 4th field
    let parts: Vec<&str> = formatted.splitn(4, "  ").collect();
    if parts.len() >= 4 {
        let rest = parts[3];
        // Trim leading spaces and extract command (before the long padding + cwd)
        let trimmed = rest.trim_start();
        // Find the command - it's the first part before the padding to cwd
        if let Some(pos) = trimmed.rfind("  ") {
            let cmd = &trimmed[..pos];
            return cmd.trim().to_string();
        }
        return trimmed.to_string();
    }
    formatted.to_string()
}
