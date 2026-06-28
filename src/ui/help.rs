use crate::app::App;
use crate::ui::theme::dimmed_style;
use ratatui::{
    layout::Rect,
    widgets::{Block, Paragraph},
    Frame,
};

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
    let help_text = if app.search_active {
        " Enter: Confirm | Esc: Cancel | Type to search... ".to_string()
    } else if app.is_detail_focused() {
        // Right panel has focus: navigation keys scroll it.
        format!(
            " q: Quit | Tab: Focus tree | ↑↓/PgUp/PgDn: Scroll | Home/End: Top/Bottom | g: {panel_name} "
        )
    } else {
        format!(
            " q: Quit | Tab: Focus panel | g: {panel_name}{raw_hint} | /: Search | r: Reload{docs_hint} | ↑↓: Navigate | ←→: Expand/Collapse "
        )
    };
    let help = Paragraph::new(help_text)
        .style(dimmed_style())
        .block(Block::default());
    frame.render_widget(help, area);
}
