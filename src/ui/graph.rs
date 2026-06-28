use crate::app::App;
use crate::graph::RelationshipGraph;
use crate::ui::theme::*;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Build the relationship lines for the selected entity, or `None` when nothing
/// is selected. Used for rendering and to measure content height for scrolling.
pub fn graph_lines(app: &App) -> Option<Vec<Line<'static>>> {
    app.relationship_graph()
        .map(|graph| format_graph(&graph, app.graph_selection))
}

pub fn draw_graph(frame: &mut Frame, app: &App, area: Rect) {
    let border = if app.is_detail_focused() {
        focused_border_style()
    } else {
        Style::default().fg(Color::Magenta)
    };
    let block = Block::default()
        .title(" Relationships (g to toggle) ")
        .borders(Borders::ALL)
        .border_style(border);

    if let Some(content) = graph_lines(app) {
        let paragraph = Paragraph::new(content)
            .block(block)
            .wrap(Wrap { trim: false })
            .scroll((app.detail_scroll, 0));
        frame.render_widget(paragraph, area);
    } else {
        let paragraph = Paragraph::new("Select an entity to view relationships")
            .block(block)
            .style(dimmed_style());
        frame.render_widget(paragraph, area);
    }
}

/// Render the graph. `selected` is the index into the navigable (existing)
/// related entities, which is highlighted so it can be jumped to.
fn format_graph(graph: &RelationshipGraph, selected: usize) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    // Center entity
    lines.push(Line::from(vec![
        Span::styled("◉ ", Style::default().fg(Color::Cyan)),
        Span::styled(format!("[{}] ", graph.center.kind), dimmed_style()),
        Span::styled(
            graph.center.display_name.clone(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(""));

    let entries = graph.ordered_related();
    let mut emitted_outgoing = false;
    let mut emitted_incoming = false;
    let mut nav_index = 0usize;

    for entry in &entries {
        if entry.outgoing && !emitted_outgoing {
            lines.push(section_header(
                "─── Outgoing ───────────────────",
                Color::Green,
            ));
            emitted_outgoing = true;
        }
        if !entry.outgoing && !emitted_incoming {
            if emitted_outgoing {
                lines.push(Line::from(""));
            }
            lines.push(section_header(
                "─── Incoming ───────────────────",
                Color::Blue,
            ));
            emitted_incoming = true;
        }

        let navigable = entry.node.exists;
        let highlighted = navigable && nav_index == selected;
        lines.push(relationship_line(entry, highlighted));
        if navigable {
            nav_index += 1;
        }
    }

    // Summary
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        format!(
            "Total: {} outgoing, {} incoming",
            graph.outgoing.len(),
            graph.incoming.len()
        ),
        dimmed_style(),
    )]));
    if nav_index > 0 {
        lines.push(Line::from(Span::styled(
            "↑↓ select related · Enter to jump",
            dimmed_style(),
        )));
    }

    lines
}

fn section_header(text: &'static str, color: Color) -> Line<'static> {
    Line::from(Span::styled(text, Style::default().fg(color)))
}

fn relationship_line(entry: &crate::graph::RelatedEntry, highlighted: bool) -> Line<'static> {
    let node = &entry.node;
    let (arrow, color) = if !entry.outgoing {
        ("←", Color::Blue)
    } else if node.exists {
        ("→", Color::Green)
    } else {
        ("⚠", Color::Yellow)
    };

    let mut spans = vec![
        Span::styled(format!("  {arrow} "), Style::default().fg(color)),
        Span::styled(format!("{}: ", entry.label), dimmed_style()),
        Span::styled(format!("[{}] ", node.kind), dimmed_style()),
        Span::styled(node.display_name.clone(), Style::default().fg(color)),
    ];
    if !node.exists {
        spans.push(Span::styled(
            " (not found)",
            Style::default().fg(Color::Red),
        ));
    }

    let line = Line::from(spans);
    if highlighted {
        line.style(selected_style())
    } else {
        line
    }
}
