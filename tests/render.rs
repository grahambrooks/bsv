//! End-to-end rendering smoke tests using ratatui's TestBackend. These exercise
//! `ui::draw` against a real (in-memory) terminal so panics or layout errors in
//! the render path are caught.

use bsv::app::App;
use bsv::ui;
use ratatui::{backend::TestBackend, Terminal};
use std::path::Path;

fn render(app: &App, width: u16, height: u16) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|f| ui::draw(f, app)).unwrap();
    terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect()
}

fn test_app() -> App {
    App::new(Path::new("testdata/large-catalog.yaml")).expect("load catalog")
}

#[test]
fn renders_tree_and_details_without_panicking() {
    let app = test_app();
    let text = render(&app, 120, 40);
    assert!(text.contains("Entities"), "tree panel title present");
}

#[test]
fn help_overlay_renders_when_enabled() {
    let mut app = test_app();
    app.show_help = true;
    let text = render(&app, 120, 40);
    assert!(
        text.contains("Keyboard Shortcuts"),
        "help overlay should be visible"
    );
}

#[test]
fn graph_view_renders() {
    let mut app = test_app();
    app.expand_all();
    app.move_down();
    app.toggle_graph();
    let text = render(&app, 120, 40);
    assert!(text.contains("Relationships"), "graph panel title present");
}
