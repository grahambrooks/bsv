mod app;
mod docs;
mod entity;
mod graph;
mod parser;
mod tree;
mod ui;
mod validator;

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

use app::{App, InputMode};

fn main() -> Result<()> {
    let root = env::args()
        .nth(1).map_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")), PathBuf::from);

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
            eprintln!("Error loading entities: {e}");
            return Err(e);
        }
    };

    // Restore terminal
    restore_terminal(&mut terminal)?;

    if let Err(e) = result {
        eprintln!("Application error: {e}");
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
                let visible_height = terminal.size()?.height.saturating_sub(4) as usize;
                match app.input_mode() {
                    InputMode::Normal => handle_normal_mode(&mut app, key.code),
                    InputMode::Search => handle_search_mode(&mut app, key.code),
                    InputMode::DocsBrowser => handle_docs_mode(&mut app, key.code, visible_height),
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn handle_normal_mode(app: &mut App, key_code: KeyCode) {
    match key_code {
        KeyCode::Char('q') => app.quit(),
        KeyCode::Esc => app.clear_search(),
        KeyCode::Char('/') => app.start_search(),
        KeyCode::Up | KeyCode::Char('k') => app.move_up(),
        KeyCode::Down | KeyCode::Char('j') => app.move_down(),
        KeyCode::Left | KeyCode::Char('h') => app.collapse(),
        KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter => app.toggle_expand(),
        KeyCode::Char('e') => app.expand_all(),
        KeyCode::Char('r') => app.reload(),
        KeyCode::Char('g') => app.toggle_graph(),
        KeyCode::Char('d') => app.open_docs(),
        _ => {}
    }
}

fn handle_search_mode(app: &mut App, key_code: KeyCode) {
    match key_code {
        KeyCode::Esc => app.cancel_search(),
        KeyCode::Enter => app.confirm_search(),
        KeyCode::Backspace => app.search_backspace(),
        KeyCode::Char(c) => app.search_input(c),
        _ => {}
    }
}

fn handle_docs_mode(app: &mut App, key_code: KeyCode, visible_height: usize) {
    match key_code {
        KeyCode::Esc => app.close_docs(),
        KeyCode::Up | KeyCode::Char('k') => {
            if let Some(browser) = &mut app.docs_browser {
                browser.move_up();
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if let Some(browser) = &mut app.docs_browser {
                browser.move_down(visible_height);
            }
        }
        KeyCode::PageUp => {
            if let Some(browser) = &mut app.docs_browser {
                browser.page_up(visible_height);
            }
        }
        KeyCode::PageDown => {
            if let Some(browser) = &mut app.docs_browser {
                browser.page_down(visible_height, visible_height);
            }
        }
        KeyCode::Enter => {
            if let Some(browser) = &mut app.docs_browser {
                browser.open_selected();
            }
        }
        KeyCode::Char('q') => app.quit(),
        _ => {}
    }
}
