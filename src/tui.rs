use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use hindsight_core::db::{self, HistoryItem};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::io;

use crate::search;

pub fn run(initial_query: &str) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_loop(&mut terminal, initial_query);

    let _ = disable_raw_mode();
    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);

    match res {
        Ok(Some(cmd)) => {
            print!("{}", cmd);
            Ok(())
        }
        Ok(None) => Ok(()),
        Err(err) => Err(err),
    }
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    initial_query: &str,
) -> Result<Option<String>> {
    let conn = db::open()?;
    let mut query = initial_query.to_string();
    let mut results = do_search(&conn, &query)?;
    let mut selected: usize = 0;

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(1)])
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
                .map(|(i, item)| {
                    let style = if i == selected {
                        Style::default().fg(Color::Black).bg(Color::White)
                    } else {
                        Style::default()
                    };
                    ListItem::new(Line::from(Span::styled(search::format_item(item), style)))
                })
                .collect();

            let list = List::new(items).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("results ({})", results.len())),
            );

            f.render_widget(list, chunks[1]);
        })?;

        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match (key.code, key.modifiers) {
                    (KeyCode::Esc, _) => return Ok(None),
                    (KeyCode::Enter, _) => {
                        return Ok(results.get(selected).map(|item| item.command.clone()));
                    }
                    (KeyCode::Up, _) | (KeyCode::Char('p'), KeyModifiers::CONTROL) => {
                        if selected > 0 {
                            selected -= 1;
                        }
                    }
                    (KeyCode::Down, _) | (KeyCode::Char('n'), KeyModifiers::CONTROL) => {
                        if selected < results.len().saturating_sub(1) {
                            selected += 1;
                        }
                    }
                    (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                        query.push(c);
                        results = do_search(&conn, &query)?;
                        selected = 0;
                    }
                    (KeyCode::Backspace, _) => {
                        query.pop();
                        results = do_search(&conn, &query)?;
                        selected = 0;
                    }
                    _ => {}
                }
            }
        }
    }
}

fn do_search(conn: &rusqlite::Connection, query: &str) -> Result<Vec<HistoryItem>> {
    if query.is_empty() {
        search::recent(conn, 50)
    } else {
        search::search(conn, query, None, None, false, 50)
    }
}
