mod details;
mod docs;
mod graph;
mod help;
mod theme;
mod tree;

use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

// Re-export the main draw function
pub use help::draw_help_footer;

pub fn draw(frame: &mut Frame, app: &App) {
    // If docs browser is active, show full-screen docs view
    if let Some(docs_browser) = &app.docs_browser {
        docs::draw_docs_browser(frame, docs_browser, frame.area());
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(frame.area());

    tree::draw_tree(frame, app, chunks[0]);

    if app.show_graph {
        graph::draw_graph(frame, app, chunks[1]);
    } else {
        details::draw_details(frame, app, chunks[1]);
    }
}
