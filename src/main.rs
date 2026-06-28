use anyhow::Result;
use bsv::app::{App, InputMode};
use bsv::cli::{parse_args, Command};
use bsv::parser::load_all_entities;
use bsv::{report, ui};
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
use std::{env, io, path::PathBuf, process::ExitCode};

const HELP: &str = "\
bsv - Backstage Entity Visualizer

USAGE:
    bsv [PATH]
    bsv --validate [PATH]
    bsv --json [PATH]

ARGS:
    PATH    Directory to scan for catalog-info.yaml files, or a single
            catalog file. Defaults to the current directory.

OPTIONS:
    --validate       Validate the catalog and print a report (non-zero exit on errors)
    --json           Print the parsed catalog as JSON
    -h, --help       Print this help and exit
    -V, --version    Print version and exit";

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    match parse_args(&args) {
        Command::Help => {
            println!("{HELP}");
            ExitCode::SUCCESS
        }
        Command::Version => {
            println!("bsv {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Command::Unknown(opt) => {
            eprintln!("error: unknown option '{opt}'\n");
            eprintln!("{HELP}");
            ExitCode::from(2)
        }
        Command::Validate(path) => run_validate(resolve_path(path)),
        Command::Json(path) => run_json(resolve_path(path)),
        Command::Run(path) => match run_tui(resolve_path(path)) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("Application error: {e}");
                ExitCode::FAILURE
            }
        },
    }
}

fn resolve_path(path: Option<PathBuf>) -> PathBuf {
    path.unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

/// Validate the catalog and print a report; exit non-zero on any problem.
fn run_validate(root: PathBuf) -> ExitCode {
    let entities = match load_all_entities(&root) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("error: failed to load catalog from {}: {e}", root.display());
            return ExitCode::FAILURE;
        }
    };
    let report = report::build_report(&entities);
    let mut stdout = io::stdout().lock();
    let _ = report::write_report(&report, &mut stdout);
    if report.has_errors() {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

/// Print the parsed catalog as JSON.
fn run_json(root: PathBuf) -> ExitCode {
    let entities = match load_all_entities(&root) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("error: failed to load catalog from {}: {e}", root.display());
            return ExitCode::FAILURE;
        }
    };
    let mut stdout = io::stdout().lock();
    match report::write_json(&entities, &mut stdout) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Launch the interactive terminal UI.
fn run_tui(root: PathBuf) -> Result<()> {
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
    result
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
                // While the help overlay is up, any key dismisses it.
                if app.show_help {
                    app.show_help = false;
                    continue;
                }
                let visible_height = terminal.size()?.height.saturating_sub(4) as usize;
                match app.input_mode() {
                    InputMode::Normal => handle_normal_mode(&mut app, key.code, visible_height),
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

fn handle_normal_mode(app: &mut App, key_code: KeyCode, visible_height: usize) {
    // Keys that apply regardless of which pane has focus.
    match key_code {
        KeyCode::Char('q') => return app.quit(),
        KeyCode::Char('?') => return app.toggle_help(),
        KeyCode::Esc => return app.focus_tree_and_clear_search(),
        KeyCode::Tab => return app.toggle_focus(),
        KeyCode::Char('/') => return app.start_search(),
        KeyCode::Char('r') => return app.reload(),
        KeyCode::Char('g') => return app.toggle_graph(),
        KeyCode::Char('y') => return app.toggle_raw(),
        KeyCode::Char('d') => return app.open_docs(),
        _ => {}
    }

    if app.is_detail_focused() {
        // Navigation keys scroll the right-hand panel.
        let max = right_panel_max_scroll(app, visible_height);
        match key_code {
            KeyCode::Up | KeyCode::Char('k') => app.scroll_detail_up(1),
            KeyCode::Down | KeyCode::Char('j') => app.scroll_detail_down(1, max),
            KeyCode::PageUp => app.scroll_detail_up(visible_height as u16),
            KeyCode::PageDown => app.scroll_detail_down(visible_height as u16, max),
            KeyCode::Home => app.scroll_detail_home(),
            KeyCode::End => app.scroll_detail_end(max),
            KeyCode::Left | KeyCode::Char('h') => app.focus_tree(),
            _ => {}
        }
    } else {
        // Navigation keys move the tree selection.
        match key_code {
            KeyCode::Up | KeyCode::Char('k') => app.move_up(),
            KeyCode::Down | KeyCode::Char('j') => app.move_down(),
            KeyCode::PageUp => app.page_up(visible_height),
            KeyCode::PageDown => app.page_down(visible_height),
            KeyCode::Home => app.move_home(),
            KeyCode::End => app.move_end(),
            KeyCode::Left | KeyCode::Char('h') => app.collapse(),
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter => app.toggle_expand(),
            KeyCode::Char('e') => app.expand_all(),
            _ => {}
        }
    }
}

/// Furthest the right-hand panel can scroll: content lines minus visible rows.
fn right_panel_max_scroll(app: &App, visible_height: usize) -> u16 {
    ui::right_panel_line_count(app).saturating_sub(visible_height) as u16
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
