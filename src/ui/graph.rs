use crate::app::App;
use crate::graph::{RelationType, RelationshipGraph};
use crate::ui::theme::*;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn draw_graph(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Relationships (g to toggle) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));

    if let Some(graph) = app.get_relationship_graph() {
        let content = format_graph(&graph);
        let paragraph = Paragraph::new(content)
            .block(block)
            .wrap(Wrap { trim: false });
        frame.render_widget(paragraph, area);
    } else {
        let paragraph = Paragraph::new("Select an entity to view relationships")
            .block(block)
            .style(dimmed_style());
        frame.render_widget(paragraph, area);
    }
}

fn format_graph(graph: &RelationshipGraph) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    // Center entity
    lines.push(Line::from(vec![
        Span::styled("◉ ", Style::default().fg(Color::Cyan)),
        Span::styled(
            format!("[{}] ", graph.center.kind),
            dimmed_style(),
        ),
        Span::styled(
            graph.center.display_name.clone(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
    ]));

    lines.push(Line::from(""));

    // Outgoing relationships
    if !graph.outgoing.is_empty() {
        format_outgoing_relationships(&graph.outgoing, &mut lines);
    }

    // Incoming relationships
    if !graph.incoming.is_empty() {
        format_incoming_relationships(&graph.incoming, &mut lines);
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

    lines
}

fn format_outgoing_relationships(
    outgoing: &[(RelationType, crate::graph::EntityNode)],
    lines: &mut Vec<Line<'static>>,
) {
    lines.push(Line::from(Span::styled(
        "─── Outgoing ───────────────────",
        Style::default().fg(Color::Green),
    )));

    // Group by relationship type
    let mut by_type: std::collections::HashMap<&str, Vec<_>> = std::collections::HashMap::new();
    for (rel_type, node) in outgoing {
        by_type.entry(rel_type.label()).or_default().push(node);
    }

    for (label, nodes) in by_type {
        for node in nodes {
            let (icon, color) = if node.exists {
                ("→", Color::Green)
            } else {
                ("⚠", Color::Yellow)
            };

            lines.push(Line::from(vec![
                Span::styled(format!("  {icon} "), Style::default().fg(color)),
                Span::styled(format!("{label}: "), dimmed_style()),
                Span::styled(
                    format!("[{}] ", node.kind),
                    dimmed_style(),
                ),
                Span::styled(node.display_name.clone(), Style::default().fg(color)),
                if node.exists {
                    Span::raw("")
                } else {
                    Span::styled(" (not found)", Style::default().fg(Color::Red))
                },
            ]));
        }
    }
}

fn format_incoming_relationships(
    incoming: &[(RelationType, crate::graph::EntityNode)],
    lines: &mut Vec<Line<'static>>,
) {
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "─── Incoming ───────────────────",
        Style::default().fg(Color::Blue),
    )));

    // Group by relationship type
    let mut by_type: std::collections::HashMap<&str, Vec<_>> = std::collections::HashMap::new();
    for (rel_type, node) in incoming {
        by_type
            .entry(inverse_label(rel_type))
            .or_default()
            .push(node);
    }

    for (label, nodes) in by_type {
        for node in nodes {
            lines.push(Line::from(vec![
                Span::styled("  ← ", Style::default().fg(Color::Blue)),
                Span::styled(format!("{label}: "), dimmed_style()),
                Span::styled(
                    format!("[{}] ", node.kind),
                    dimmed_style(),
                ),
                Span::styled(node.display_name.clone(), Style::default().fg(Color::Blue)),
            ]));
        }
    }
}

fn inverse_label(rel_type: &RelationType) -> &'static str {
    match rel_type {
        RelationType::Owner => "owns",
        RelationType::System => "contains",
        RelationType::Domain => "contains",
        RelationType::Child => "child of",
        RelationType::DependencyOf => "depended on by",
        RelationType::ConsumedBy => "consumed by",
        RelationType::ProvidedBy => "provides",
        RelationType::HasMember => "member",
        _ => rel_type.label(),
    }
}
