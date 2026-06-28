mod details;
mod docs;
mod graph;
mod help;
mod theme;
mod tree;

use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

// Re-export the main draw function
pub use help::draw_help_footer;

/// Below this width the tree/detail panes stack vertically instead of
/// sitting side by side, so neither pane becomes unusably narrow.
const NARROW_WIDTH: u16 = 90;

/// Geometry of the two main panes for a given content area.
pub struct Panes {
    pub tree: Rect,
    pub detail: Rect,
}

/// Split a content area into the tree and detail panes, choosing a side-by-side
/// or stacked layout based on width. Shared by rendering and mouse hit-testing.
pub fn panes(area: Rect) -> Panes {
    let chunks = if area.width < NARROW_WIDTH {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
            .split(area)
    };
    Panes {
        tree: chunks[0],
        detail: chunks[1],
    }
}

/// Number of content lines the right-hand panel would render for the current
/// selection. Used to clamp detail-panel scrolling.
pub fn right_panel_line_count(app: &App) -> usize {
    if app.show_graph {
        graph::graph_lines(app).map_or(0, |lines| lines.len())
    } else {
        details::detail_lines(app).map_or(0, |lines| lines.len())
    }
}

pub fn draw(frame: &mut Frame, app: &App) {
    // If docs browser is active, show full-screen docs view
    if let Some(docs_browser) = &app.docs_browser {
        docs::draw_docs_browser(frame, docs_browser, frame.area());
        return;
    }

    let layout = panes(frame.area());

    tree::draw_tree(frame, app, layout.tree);

    if app.show_graph {
        graph::draw_graph(frame, app, layout.detail);
    } else {
        details::draw_details(frame, app, layout.detail);
    }

    // The help overlay floats above everything else.
    if app.show_help {
        help::draw_help_overlay(frame, frame.area());
    }
}
