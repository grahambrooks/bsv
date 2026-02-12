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

    let help_text = if app.search_active {
        " Enter: Confirm | Esc: Cancel | Type to search... ".to_string()
    } else if app.show_graph {
        format!(" q: Quit | g: Details | /: Search | r: Reload{docs_hint} | ↑↓: Navigate ")
    } else {
        format!(
            " q: Quit | g: Graph | /: Search | r: Reload{docs_hint} | ↑↓: Navigate | ←→: Expand/Collapse "
        )
    };
    let help = Paragraph::new(help_text)
        .style(dimmed_style())
        .block(Block::default());
    frame.render_widget(help, area);
}
