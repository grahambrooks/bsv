use crate::app::App;
use crate::entity::{EntityIndex, EntityRef, EntityWithSource};
use crate::graph::{RelationType, RelationshipGraph};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

pub fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(frame.area());

    draw_tree(frame, app, chunks[0]);

    if app.show_graph {
        draw_graph(frame, app, chunks[1]);
    } else {
        draw_details(frame, app, chunks[1]);
    }
}

fn draw_tree(frame: &mut Frame, app: &App, area: Rect) {
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
                    "[-] "
                } else {
                    "[+] "
                }
            } else {
                "    "
            };

            let indent = "  ".repeat(node.depth);
            let label = format!("{}{}{}", indent, prefix, node.label);

            let style = if is_selected {
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else if node.is_category {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
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
        .border_style(Style::default().fg(Color::Cyan));

    let list = List::new(items).block(tree_block);

    frame.render_widget(list, chunks[1]);
}

fn draw_search(frame: &mut Frame, app: &App, area: Rect) {
    let (border_color, cursor) = if app.search_active {
        (Color::Yellow, "_")
    } else {
        (Color::Cyan, "")
    };

    let search_text = if app.search_query.is_empty() && !app.search_active {
        "Press / to search...".to_string()
    } else {
        format!("{}{}", app.search_query, cursor)
    };

    let style = if app.search_query.is_empty() && !app.search_active {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };

    let search_block = Block::default()
        .title(" Search ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let search = Paragraph::new(search_text).style(style).block(search_block);

    frame.render_widget(search, area);
}

fn draw_details(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Details ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    if let Some(ews) = app.selected_entity() {
        let content = format_entity_details(ews, &app.entity_index);
        let paragraph = Paragraph::new(content)
            .block(block)
            .wrap(Wrap { trim: false });
        frame.render_widget(paragraph, area);
    } else {
        let node = app.tree.get_node(app.tree_state.selected);
        let text = match node {
            Some(n) if n.is_category => "Category node - select an entity to view details",
            _ => "No entity selected",
        };
        let paragraph = Paragraph::new(text)
            .block(block)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(paragraph, area);
    }
}

fn format_entity_details(ews: &EntityWithSource, index: &EntityIndex) -> Vec<Line<'static>> {
    let entity = &ews.entity;
    let mut lines = Vec::new();

    // Header
    lines.push(Line::from(vec![
        Span::styled("Kind: ", Style::default().fg(Color::Yellow)),
        Span::styled(
            entity.kind.to_string(),
            Style::default().add_modifier(Modifier::BOLD),
        ),
    ]));

    lines.push(Line::from(vec![
        Span::styled("Name: ", Style::default().fg(Color::Yellow)),
        Span::styled(
            entity.metadata.name.clone(),
            Style::default().add_modifier(Modifier::BOLD),
        ),
    ]));

    if let Some(title) = &entity.metadata.title {
        lines.push(Line::from(vec![
            Span::styled("Title: ", Style::default().fg(Color::Yellow)),
            Span::raw(title.clone()),
        ]));
    }

    if let Some(ns) = &entity.metadata.namespace {
        lines.push(Line::from(vec![
            Span::styled("Namespace: ", Style::default().fg(Color::Yellow)),
            Span::raw(ns.clone()),
        ]));
    }

    lines.push(Line::from(""));

    // Description
    if let Some(desc) = &entity.metadata.description {
        lines.push(Line::from(Span::styled(
            "Description:",
            Style::default().fg(Color::Yellow),
        )));
        lines.push(Line::from(desc.clone()));
        lines.push(Line::from(""));
    }

    // Spec details with reference validation
    if let Some(owner) = entity.owner() {
        let ref_line = format_entity_ref(&owner, "group", index);
        lines.push(Line::from(
            std::iter::once(Span::styled("Owner: ", Style::default().fg(Color::Yellow)))
                .chain(ref_line)
                .collect::<Vec<_>>(),
        ));
    }

    if let Some(system) = entity.system() {
        let ref_line = format_entity_ref(&system, "system", index);
        lines.push(Line::from(
            std::iter::once(Span::styled("System: ", Style::default().fg(Color::Yellow)))
                .chain(ref_line)
                .collect::<Vec<_>>(),
        ));
    }

    if let Some(domain) = entity.domain() {
        let ref_line = format_entity_ref(&domain, "domain", index);
        lines.push(Line::from(
            std::iter::once(Span::styled("Domain: ", Style::default().fg(Color::Yellow)))
                .chain(ref_line)
                .collect::<Vec<_>>(),
        ));
    }

    if let Some(lifecycle) = entity.lifecycle() {
        lines.push(Line::from(vec![
            Span::styled("Lifecycle: ", Style::default().fg(Color::Yellow)),
            Span::raw(lifecycle),
        ]));
    }

    if let Some(etype) = entity.entity_type() {
        lines.push(Line::from(vec![
            Span::styled("Type: ", Style::default().fg(Color::Yellow)),
            Span::raw(etype),
        ]));
    }

    // Tags
    if !entity.metadata.tags.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Tags:",
            Style::default().fg(Color::Yellow),
        )));
        lines.push(Line::from(entity.metadata.tags.join(", ")));
    }

    // Source file
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("Source: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            ews.source_file.display().to_string(),
            Style::default().fg(Color::DarkGray),
        ),
    ]));

    lines
}

/// Format an entity reference with resolved kind/namespace and validation
/// Explicit parts shown in bright colors, inferred parts shown dim in [brackets]
fn format_entity_ref(
    reference: &str,
    default_kind: &str,
    index: &EntityIndex,
) -> Vec<Span<'static>> {
    let entity_ref = EntityRef::parse(reference, default_kind);
    let mut spans = Vec::new();

    // Check for errors
    let exists = index.contains(&entity_ref);
    let known_kind = entity_ref.is_known_kind();

    // Determine base color based on validation status
    let (explicit_color, inferred_color, error_suffix) = if !known_kind {
        (Color::Red, Color::DarkGray, Some(" [unknown kind]"))
    } else if !exists {
        (Color::Yellow, Color::DarkGray, Some(" [not found]"))
    } else {
        (Color::Green, Color::DarkGray, None)
    };

    // Format kind - show in brackets if inferred
    if entity_ref.kind_inferred {
        spans.push(Span::styled(
            format!("[{}]", entity_ref.kind),
            Style::default()
                .fg(inferred_color)
                .add_modifier(Modifier::DIM),
        ));
    } else {
        spans.push(Span::styled(
            entity_ref.kind.clone(),
            Style::default()
                .fg(explicit_color)
                .add_modifier(Modifier::BOLD),
        ));
    }

    spans.push(Span::styled(":", Style::default().fg(Color::DarkGray)));

    // Format namespace - show in brackets if inferred
    if entity_ref.namespace_inferred {
        spans.push(Span::styled(
            format!("[{}]", entity_ref.namespace),
            Style::default()
                .fg(inferred_color)
                .add_modifier(Modifier::DIM),
        ));
    } else {
        spans.push(Span::styled(
            entity_ref.namespace.clone(),
            Style::default().fg(explicit_color),
        ));
    }

    spans.push(Span::styled("/", Style::default().fg(Color::DarkGray)));

    // Name is always explicit
    spans.push(Span::styled(
        entity_ref.name.clone(),
        Style::default()
            .fg(explicit_color)
            .add_modifier(Modifier::BOLD),
    ));

    // Add error suffix if needed
    if let Some(suffix) = error_suffix {
        spans.push(Span::styled(
            suffix.to_string(),
            Style::default().fg(Color::Red),
        ));
    }

    spans
}

fn draw_graph(frame: &mut Frame, app: &App, area: Rect) {
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
            .style(Style::default().fg(Color::DarkGray));
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
            Style::default().fg(Color::DarkGray),
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
        lines.push(Line::from(Span::styled(
            "─── Outgoing ───────────────────",
            Style::default().fg(Color::Green),
        )));

        // Group by relationship type
        let mut by_type: std::collections::HashMap<&str, Vec<_>> = std::collections::HashMap::new();
        for (rel_type, node) in &graph.outgoing {
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
                    Span::styled(format!("  {} ", icon), Style::default().fg(color)),
                    Span::styled(format!("{}: ", label), Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("[{}] ", node.kind),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(node.display_name.clone(), Style::default().fg(color)),
                    if !node.exists {
                        Span::styled(" (not found)", Style::default().fg(Color::Red))
                    } else {
                        Span::raw("")
                    },
                ]));
            }
        }
    }

    // Incoming relationships
    if !graph.incoming.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "─── Incoming ───────────────────",
            Style::default().fg(Color::Blue),
        )));

        // Group by relationship type
        let mut by_type: std::collections::HashMap<&str, Vec<_>> = std::collections::HashMap::new();
        for (rel_type, node) in &graph.incoming {
            by_type
                .entry(inverse_label(rel_type))
                .or_default()
                .push(node);
        }

        for (label, nodes) in by_type {
            for node in nodes {
                lines.push(Line::from(vec![
                    Span::styled("  ← ", Style::default().fg(Color::Blue)),
                    Span::styled(format!("{}: ", label), Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("[{}] ", node.kind),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(node.display_name.clone(), Style::default().fg(Color::Blue)),
                ]));
            }
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
        Style::default().fg(Color::DarkGray),
    )]));

    lines
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

pub fn draw_help_footer(frame: &mut Frame, app: &App, area: Rect) {
    let help_text = if app.search_active {
        " Enter: Confirm | Esc: Cancel | Type to search... "
    } else if app.show_graph {
        " q: Quit | g: Details | /: Search | r: Reload | ↑↓: Navigate "
    } else {
        " q: Quit | g: Graph | /: Search | r: Reload | ↑↓: Navigate | ←→: Expand/Collapse "
    };
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default());
    frame.render_widget(help, area);
}
