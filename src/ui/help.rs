use crate::app::App;
use crate::ui::theme::{border_style, dimmed_style, label_style};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

const HELP_ROWS: &[(&str, &str)] = &[
    ("Tab", "Switch focus between tree and detail panel"),
    ("↑ / k, ↓ / j", "Move selection (tree) or scroll (panel)"),
    ("PgUp / PgDn", "Move / scroll by a page"),
    ("Home / End", "Jump to first / last (or top / bottom)"),
    ("← / h", "Collapse node / return focus to tree"),
    ("→ / l / Enter", "Expand node"),
    ("e", "Expand all nodes"),
    ("c", "Collapse all nodes"),
    ("/", "Search (incremental, case-insensitive)"),
    ("g", "Toggle relationship graph"),
    (
        "Enter (graph)",
        "Jump to the highlighted related entity (Tab to focus graph)",
    ),
    ("y", "Toggle raw YAML view"),
    ("d", "Open documentation browser (when available)"),
    ("r", "Reload catalog from disk"),
    (
        "x / X",
        "Jump to next / previous entity with validation errors",
    ),
    ("?", "Toggle this help"),
    ("Esc", "Clear search / close / return focus to tree"),
    ("q", "Quit"),
];

/// Draw the keyboard-shortcut help overlay centered over the screen.
pub fn draw_help_overlay(frame: &mut Frame, area: Rect) {
    let popup = centered_rect(60, 80, area);

    let mut lines: Vec<Line> = vec![Line::from("")];
    for (keys, desc) in HELP_ROWS {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(format!("{keys:<16}"), label_style()),
            Span::raw(*desc),
        ]));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  Press any key to close",
        dimmed_style(),
    )));

    let block = Block::default()
        .title(" Keyboard Shortcuts ")
        .borders(Borders::ALL)
        .border_style(border_style());

    frame.render_widget(Clear, popup);
    frame.render_widget(Paragraph::new(lines).block(block), popup);
}

/// Compute a rectangle centered within `area`, sized as a percentage of it.
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}

pub fn draw_help_footer(frame: &mut Frame, app: &App, area: Rect) {
    // Don't draw footer if docs browser is active (it has its own)
    if app.docs_browser.is_some() {
        return;
    }

    let has_docs = !app.get_docs_refs().is_empty();
    let docs_hint = if has_docs { " | d: Docs" } else { "" };

    let raw_hint = if app.show_raw {
        " | y: Details"
    } else {
        " | y: Raw YAML"
    };
    let panel_name = if app.show_graph { "Details" } else { "Graph" };
    let warn_hint = if app.load_warnings.is_empty() {
        String::new()
    } else {
        format!(" | ⚠ {} warning(s)", app.load_warnings.len())
    };
    let help_text = if app.search_active {
        " Enter: Confirm | Esc: Cancel | Type to search... ".to_string()
    } else if app.is_detail_focused() && app.show_graph {
        // Graph pane focused: up/down pick a related entity, Enter jumps.
        format!(
            " q: Quit | ?: Help | Tab: Focus tree | ↑↓: Select related | Enter: Jump | PgUp/PgDn: Scroll | g: Details{warn_hint} "
        )
    } else if app.is_detail_focused() {
        // Detail pane focused: navigation keys scroll it.
        format!(
            " q: Quit | ?: Help | Tab: Focus tree | ↑↓/PgUp/PgDn: Scroll | Home/End: Top/Bottom | g: {panel_name}{warn_hint} "
        )
    } else {
        let err_hint = if app.error_count() > 0 {
            " | x: Errors"
        } else {
            ""
        };
        format!(
            " q: Quit | ?: Help | Tab: Focus | g: {panel_name}{raw_hint} | /: Search | r: Reload{docs_hint}{err_hint} | ↑↓: Nav | ←→: Expand{warn_hint} "
        )
    };
    let help = Paragraph::new(help_text)
        .style(dimmed_style())
        .block(Block::default());
    frame.render_widget(help, area);
}
