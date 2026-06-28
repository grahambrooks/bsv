use anyhow::Result;
use bsv::app::{App, InputMode};
use bsv::cli::{parse_args, Command};
use bsv::parser::load_all_entities;
use bsv::watcher::CatalogWatcher;
use bsv::{report, ui};
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, MouseEvent,
        MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    Terminal,
};
use std::{
    env, io,
    path::PathBuf,
    process::ExitCode,
    time::{Duration, Instant},
};

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

    // Watch the catalog for changes (best-effort; the UI still works without it).
    let watcher = CatalogWatcher::new(&root).ok();

    // Create app and run
    let result = match App::new(&root) {
        Ok(app) => run_app(&mut terminal, app, watcher),
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

/// How long the catalog must be quiet after a change before we reload, so a
/// burst of editor writes coalesces into a single reload.
const RELOAD_DEBOUNCE: Duration = Duration::from_millis(300);
/// How often the event loop wakes to service the file watcher when idle.
const POLL_INTERVAL: Duration = Duration::from_millis(200);

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    mut app: App,
    watcher: Option<CatalogWatcher>,
) -> Result<()> {
    let mut pending_reload: Option<Instant> = None;

    loop {
        terminal.draw(|frame| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(3), Constraint::Length(1)])
                .split(frame.area());

            ui::draw(frame, &app);
            ui::draw_help_footer(frame, &app, chunks[1]);
        })?;

        // Note a filesystem change; the actual reload is debounced below.
        if let Some(w) = &watcher {
            if w.drain() {
                pending_reload = Some(Instant::now());
            }
        }
        if let Some(since) = pending_reload {
            if since.elapsed() >= RELOAD_DEBOUNCE {
                app.reload();
                pending_reload = None;
            }
        }

        // Poll so the loop also wakes to service the watcher while idle.
        if !event::poll(POLL_INTERVAL)? {
            if app.should_quit {
                return Ok(());
            }
            continue;
        }

        let visible_height = terminal.size()?.height.saturating_sub(4) as usize;
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                // While the help overlay is up, any key dismisses it.
                if app.show_help {
                    app.show_help = false;
                    continue;
                }
                match app.input_mode() {
                    InputMode::Normal => handle_normal_mode(&mut app, key.code, visible_height),
                    InputMode::Search => handle_search_mode(&mut app, key.code),
                    InputMode::DocsBrowser => handle_docs_mode(&mut app, key.code, visible_height),
                }
            }
            Event::Mouse(mouse) => {
                let size = terminal.size()?;
                let area = Rect::new(0, 0, size.width, size.height);
                handle_mouse(&mut app, mouse, area, visible_height);
            }
            _ => {}
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

/// Scroll-wheel handling: scroll whichever pane the cursor is over.
fn handle_mouse(app: &mut App, mouse: MouseEvent, area: Rect, visible_height: usize) {
    let down = match mouse.kind {
        MouseEventKind::ScrollDown => true,
        MouseEventKind::ScrollUp => false,
        _ => return,
    };

    // The docs browser is full-screen and owns all scrolling.
    if let Some(browser) = &mut app.docs_browser {
        if down {
            browser.move_down(visible_height);
        } else {
            browser.move_up();
        }
        return;
    }

    let layout = ui::panes(area);
    let over_detail = rect_contains(layout.detail, mouse.column, mouse.row);
    if over_detail {
        let max = right_panel_max_scroll(app, visible_height);
        if down {
            app.scroll_detail_down(3, max);
        } else {
            app.scroll_detail_up(3);
        }
    } else if down {
        app.move_down();
    } else {
        app.move_up();
    }
}

fn rect_contains(rect: Rect, x: u16, y: u16) -> bool {
    x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height
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
        KeyCode::Char('x') => return app.next_error(),
        KeyCode::Char('X') => return app.prev_error(),
        KeyCode::Char('g') => return app.toggle_graph(),
        KeyCode::Char('y') => return app.toggle_raw(),
        KeyCode::Char('d') => return app.open_docs(),
        _ => {}
    }

    if app.is_detail_focused() {
        let max = right_panel_max_scroll(app, visible_height);
        match key_code {
            // In the graph view, up/down pick a related entity and Enter jumps
            // to it; PageUp/PageDown still scroll. Elsewhere up/down scroll.
            KeyCode::Up | KeyCode::Char('k') if app.show_graph => app.graph_select_prev(),
            KeyCode::Down | KeyCode::Char('j') if app.show_graph => app.graph_select_next(),
            KeyCode::Enter if app.show_graph => {
                app.jump_to_related();
            }
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
            KeyCode::Char('c') => app.collapse_all(),
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
