mod app;
mod entity;
mod graph;
mod parser;
mod tree;
mod ui;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    Terminal,
};
use std::{env, io, path::PathBuf};

use app::App;

fn main() -> Result<()> {
    let root = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run
    let result = match App::new(&root) {
        Ok(app) => run_app(&mut terminal, app),
        Err(e) => {
            restore_terminal(&mut terminal)?;
            eprintln!("Error loading entities: {}", e);
            return Err(e);
        }
    };

    // Restore terminal
    restore_terminal(&mut terminal)?;

    if let Err(e) = result {
        eprintln!("Application error: {}", e);
        return Err(e);
    }

    Ok(())
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, mut app: App) -> Result<()> {
    loop {
        terminal.draw(|frame| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(3), Constraint::Length(1)])
                .split(frame.area());

            ui::draw(frame, &app);
            ui::draw_help_footer(frame, &app, chunks[1]);
        })?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                if app.search_active {
                    // Search mode input handling
                    match key.code {
                        KeyCode::Esc => {
                            app.cancel_search();
                        }
                        KeyCode::Enter => {
                            app.confirm_search();
                        }
                        KeyCode::Backspace => {
                            app.search_backspace();
                        }
                        KeyCode::Char(c) => {
                            app.search_input(c);
                        }
                        _ => {}
                    }
                } else {
                    // Normal mode input handling
                    match key.code {
                        KeyCode::Char('q') => {
                            app.quit();
                        }
                        KeyCode::Esc => {
                            app.clear_search();
                        }
                        KeyCode::Char('/') => {
                            app.start_search();
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            app.move_up();
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            app.move_down();
                        }
                        KeyCode::Left | KeyCode::Char('h') => {
                            app.collapse();
                        }
                        KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter => {
                            app.toggle_expand();
                        }
                        KeyCode::Char('e') => {
                            app.expand_all();
                        }
                        KeyCode::Char('r') => {
                            app.reload();
                        }
                        KeyCode::Char('g') => {
                            app.toggle_graph();
                        }
                        _ => {}
                    }
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}
