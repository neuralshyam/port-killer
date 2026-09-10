pub mod app;
pub mod ui;

use std::io;
use std::time::Duration;
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::tui::app::App;

pub fn run_tui() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    let res = run_loop(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    res
}

fn run_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui::render(f, app))?;

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if app.is_searching {
                    match key.code {
                        KeyCode::Enter => {
                            app.is_searching = false;
                        }
                        KeyCode::Esc => {
                            app.is_searching = false;
                            app.search_query.clear();
                            app.apply_filter();
                        }
                        KeyCode::Backspace => {
                            app.search_query.pop();
                            app.apply_filter();
                        }
                        KeyCode::Char(c) => {
                            app.search_query.push(c);
                            app.apply_filter();
                        }
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            break;
                        }
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            break;
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            app.next();
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            app.previous();
                        }
                        KeyCode::Char('/') => {
                            app.is_searching = true;
                        }
                        KeyCode::Char('p') => {
                            app.probe_current();
                        }
                        KeyCode::Char(' ') => {
                            app.toggle_selection();
                        }
                        KeyCode::Enter => {
                            app.kill_selected_batch(false);
                        }
                        KeyCode::Char('K') => {
                            app.kill_selected_batch(true);
                        }
                        KeyCode::Char('r') => {
                            app.refresh();
                            app.status_message = Some(("Refreshed ports list".to_string(), false));
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(())
}
