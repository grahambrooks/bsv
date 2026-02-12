use crate::app::App;
use crate::ui::theme::*;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn draw_tree(frame: &mut Frame, app: &App, area: Rect) {
    // Split area for search bar and tree
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(area);

    // Draw search bar
    draw_search(frame, app, chunks[0]);

    let visible_nodes = app.visible_nodes();

    let items: Vec<ListItem> = visible_nodes
        .iter()
        .map(|node| {
            let is_selected = node.id == app.tree_state.selected;
            let has_children = !node.children.is_empty();
            let is_expanded = app.tree_state.is_expanded(node.id);

            let prefix = if has_children {
                if is_expanded {
                    EXPANDED_SYMBOL
                } else {
                    COLLAPSED_SYMBOL
                }
            } else {
                LEAF_INDENT
            };

            let indent = TREE_INDENT.repeat(node.depth);

            // Check for validation errors
            let has_errors = node
                .entity
                .as_ref()
                .is_some_and(|ews| !ews.validation_errors.is_empty());

            let error_indicator = if has_errors {
                let error_count = node
                    .entity
                    .as_ref()
                    .map_or(0, |ews| ews.validation_errors.len());
                format!("{ERROR_INDICATOR}{error_count}")
            } else {
                String::new()
            };

            let label = format!("{}{}{}{}", indent, prefix, node.label, error_indicator);

            let style = if is_selected {
                selected_style()
            } else if has_errors {
                error_style()
            } else if node.is_category {
                category_style()
            } else {
                normal_style()
            };

            ListItem::new(Line::from(Span::styled(label, style)))
        })
        .collect();

    let title = if app.search_query.is_empty() {
        format!(" Entities ({}) ", app.entity_count)
    } else {
        format!(" Entities ({}/{}) ", visible_nodes.len(), app.entity_count)
    };

    let tree_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style());

    let list = List::new(items).block(tree_block);

    frame.render_widget(list, chunks[1]);
}

fn draw_search(frame: &mut Frame, app: &App, area: Rect) {
    let (border_color, cursor) = if app.search_active {
        (Color::Yellow, SELECTED_INDICATOR)
    } else {
        (Color::Cyan, "")
    };

    let search_text = if app.search_query.is_empty() && !app.search_active {
        "Press / to search...".to_string()
    } else {
        format!("{}{}", app.search_query, cursor)
    };

    let style = if app.search_query.is_empty() && !app.search_active {
        dimmed_style()
    } else {
        normal_style()
    };

    let search_block = Block::default()
        .title(" Search ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let search = Paragraph::new(search_text).style(style).block(search_block);

    frame.render_widget(search, area);
}
