use crate::app::App;
use crate::docs::DocsBrowser;
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
    // If docs browser is active, show full-screen docs view
    if let Some(docs_browser) = &app.docs_browser {
        draw_docs_browser(frame, docs_browser, frame.area());
        return;
    }

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
            
            // Check for validation errors
            let has_errors = node.entity.as_ref()
                .is_some_and(|ews| !ews.validation_errors.is_empty());
            
            let error_indicator = if has_errors {
                let error_count = node.entity.as_ref()
                    .map_or(0, |ews| ews.validation_errors.len());
                format!(" ⚠ {error_count}")
            } else {
                String::new()
            };
            
            let label = format!("{}{}{}{}", indent, prefix, node.label, error_indicator);

            let style = if is_selected {
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else if has_errors {
                Style::default()
                    .fg(Color::Red)
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
        let content = format_entity_details(ews, &app.entity_index, &app.entities);
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

fn format_entity_details(ews: &EntityWithSource, index: &EntityIndex, all_entities: &[EntityWithSource]) -> Vec<Line<'static>> {
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

    // Group-specific information
    use crate::entity::EntityKind;
    if matches!(entity.kind, EntityKind::Group) {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "─── Group Hierarchy ───",
            Style::default().fg(Color::Magenta),
        )));

        // Parent group
        if let Some(parent) = entity.get_spec_string("parent") {
            let ref_line = format_entity_ref(&parent, "group", index);
            lines.push(Line::from(
                std::iter::once(Span::styled("Parent: ", Style::default().fg(Color::Yellow)))
                    .chain(ref_line)
                    .collect::<Vec<_>>(),
            ));
        } else {
            lines.push(Line::from(Span::styled(
                "Parent: (none - root group)",
                Style::default().fg(Color::DarkGray),
            )));
        }

        // Child groups
        if let Some(children) = entity.spec.get("children") {
            if let Some(children_arr) = children.as_sequence() {
                if !children_arr.is_empty() {
                    lines.push(Line::from(""));
                    lines.push(Line::from(Span::styled(
                        format!("Child Groups ({}):", children_arr.len()),
                        Style::default().fg(Color::Yellow),
                    )));
                    for child in children_arr {
                        if let Some(child_str) = child.as_str() {
                            let ref_line = format_entity_ref(child_str, "group", index);
                            lines.push(Line::from(
                                std::iter::once(Span::styled("  └─ ", Style::default().fg(Color::DarkGray)))
                                    .chain(ref_line)
                                    .collect::<Vec<_>>(),
                            ));
                        }
                    }
                }
            }
        }

        // Members (users who have memberOf pointing to this group)
        // Find all users and groups that are members of this group
        let group_ref = entity.ref_key();
        let mut members: Vec<&EntityWithSource> = all_entities
            .iter()
            .filter(|e| {
                if let Some(member_of) = e.entity.spec.get("memberOf") {
                    if let Some(member_of_arr) = member_of.as_sequence() {
                        return member_of_arr.iter().any(|m| {
                            if let Some(m_str) = m.as_str() {
                                let parsed = EntityRef::parse(m_str, "group");
                                parsed.canonical() == group_ref
                            } else {
                                false
                            }
                        });
                    }
                }
                false
            })
            .collect();

        // Sort members by kind, then name
        members.sort_by(|a, b| {
            a.entity
                .kind
                .to_string()
                .cmp(&b.entity.kind.to_string())
                .then_with(|| a.entity.metadata.name.cmp(&b.entity.metadata.name))
        });

        lines.push(Line::from(""));
        if members.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("Members: ", Style::default().fg(Color::Yellow)),
                Span::styled("(none)", Style::default().fg(Color::DarkGray)),
            ]));
        } else {
            lines.push(Line::from(Span::styled(
                format!("Members ({}):", members.len()),
                Style::default().fg(Color::Yellow),
            )));
            
            // Group members by kind for better organization
            let mut current_kind = String::new();
            for member in members {
                let kind_str = member.entity.kind.to_string();
                if kind_str != current_kind {
                    if !current_kind.is_empty() {
                        lines.push(Line::from(""));
                    }
                    current_kind = kind_str.clone();
                }
                
                let kind_label = format!("[{}]", kind_str.to_lowercase());
                lines.push(Line::from(vec![
                    Span::styled("  • ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        kind_label,
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::raw(" "),
                    Span::styled(
                        member.entity.display_name(),
                        Style::default().fg(Color::Cyan),
                    ),
                ]));
            }
        }
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

    // Links
    if !entity.metadata.links.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Links:",
            Style::default().fg(Color::Yellow),
        )));
        for link in &entity.metadata.links {
            let title = link
                .title
                .as_deref()
                .or(link.url.as_deref())
                .unwrap_or("(untitled)");
            let url = link.url.as_deref().unwrap_or("");
            let icon = link
                .icon
                .as_ref()
                .map(|i| format!("[{i}] "))
                .unwrap_or_default();
            lines.push(Line::from(vec![
                Span::styled(format!("  {icon}"), Style::default().fg(Color::DarkGray)),
                Span::styled(title.to_string(), Style::default().fg(Color::Cyan)),
                Span::styled(format!(" ({url})"), Style::default().fg(Color::DarkGray)),
            ]));
        }
    }

    // Annotations (with special highlighting for docs-related ones)
    if !entity.metadata.annotations.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Annotations:",
            Style::default().fg(Color::Yellow),
        )));

        let mut sorted_annotations: Vec<_> = entity.metadata.annotations.iter().collect();
        sorted_annotations.sort_by_key(|(k, _)| *k);

        for (key, value) in sorted_annotations {
            let is_docs_annotation = key.contains("techdocs") || key.contains("adr");
            let key_style = if is_docs_annotation {
                Style::default().fg(Color::Magenta)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            let value_style = if is_docs_annotation {
                Style::default().fg(Color::Magenta)
            } else {
                Style::default().fg(Color::White)
            };
            let doc_hint = if is_docs_annotation {
                Span::styled(" [d to view]", Style::default().fg(Color::Green))
            } else {
                Span::raw("")
            };

            lines.push(Line::from(vec![
                Span::styled(format!("  {key}: "), key_style),
                Span::styled(value.clone(), value_style),
                doc_hint,
            ]));
        }
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

    // Validation errors
    if !ews.validation_errors.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("⚠ Validation Errors ({}):", ews.validation_errors.len()),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )));
        
        for (idx, error) in ews.validation_errors.iter().enumerate() {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {}. ", idx + 1),
                    Style::default().fg(Color::Red),
                ),
                Span::styled(
                    format!("Field: {}", error.path),
                    Style::default().fg(Color::Yellow),
                ),
            ]));
            lines.push(Line::from(vec![
                Span::styled("     ", Style::default()),
                Span::styled(
                    error.message.clone(),
                    Style::default().fg(Color::White),
                ),
            ]));
        }
    }

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
                    Span::styled(format!("  {icon} "), Style::default().fg(color)),
                    Span::styled(format!("{label}: "), Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("[{}] ", node.kind),
                        Style::default().fg(Color::DarkGray),
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
                    Span::styled(format!("{label}: "), Style::default().fg(Color::DarkGray)),
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

fn draw_docs_browser(frame: &mut Frame, browser: &DocsBrowser, area: Rect) {
    // Split into main content and help footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .split(area);

    let content_area = chunks[0];
    let help_area = chunks[1];

    if let Some(doc_content) = &browser.viewing_content {
        // Show document content
        draw_doc_content(frame, doc_content, browser.scroll_offset, content_area);

        let help = Paragraph::new(" Esc: Back to list | ↑↓/jk: Scroll | PgUp/PgDn: Page scroll ")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(help, help_area);
    } else {
        // Show file list
        draw_docs_file_list(frame, browser, content_area);

        let help = Paragraph::new(" Esc: Close docs | Enter: Open file | ↑↓: Navigate ")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(help, help_area);
    }
}

fn draw_docs_file_list(frame: &mut Frame, browser: &DocsBrowser, area: Rect) {
    let title = format!(
        " {} Documentation ({} files) ",
        browser.docs_ref.ref_type.label(),
        browser.files.len()
    );

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));

    if browser.files.is_empty() {
        let paragraph = Paragraph::new("No markdown files found in documentation directory")
            .block(block)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(paragraph, area);
        return;
    }

    let items: Vec<ListItem> = browser
        .files
        .iter()
        .enumerate()
        .map(|(i, file)| {
            let is_selected = i == browser.selected_index;
            let style = if is_selected {
                Style::default()
                    .bg(Color::Magenta)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(Line::from(Span::styled(file.relative_path.clone(), style)))
        })
        .collect();

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn draw_doc_content(
    frame: &mut Frame,
    content: &crate::docs::DocContent,
    scroll: usize,
    area: Rect,
) {
    let title = format!(" {} ", content.file.name);

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));

    // Calculate visible area height (minus borders)
    let inner_height = area.height.saturating_sub(2) as usize;

    // Create lines with basic markdown rendering
    let lines: Vec<Line> = content
        .lines
        .iter()
        .skip(scroll)
        .take(inner_height)
        .map(|line| format_markdown_line(line))
        .collect();

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);

    // Show scroll indicator
    if content.lines.len() > inner_height {
        let scroll_info = format!(
            " {}-{}/{} ",
            scroll + 1,
            (scroll + inner_height).min(content.lines.len()),
            content.lines.len()
        );
        let scroll_len = scroll_info.len();
        let scroll_span = Span::styled(scroll_info, Style::default().fg(Color::DarkGray));
        let scroll_para = Paragraph::new(Line::from(scroll_span));
        let scroll_area = Rect {
            x: area.x + area.width.saturating_sub(scroll_len as u16 + 2),
            y: area.y,
            width: scroll_len as u16 + 1,
            height: 1,
        };
        frame.render_widget(scroll_para, scroll_area);
    }
}

/// Basic markdown line formatting
fn format_markdown_line(line: &str) -> Line<'static> {
    let trimmed = line.trim_start();

    // Headers
    if trimmed.starts_with("# ") {
        return Line::from(Span::styled(
            line.to_string(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ));
    }
    if trimmed.starts_with("## ") {
        return Line::from(Span::styled(
            line.to_string(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ));
    }
    if trimmed.starts_with("### ") {
        return Line::from(Span::styled(
            line.to_string(),
            Style::default().fg(Color::Cyan),
        ));
    }

    // Code blocks
    if trimmed.starts_with("```") {
        return Line::from(Span::styled(
            line.to_string(),
            Style::default().fg(Color::Yellow),
        ));
    }

    // Bullet points
    if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
        return Line::from(Span::styled(
            line.to_string(),
            Style::default().fg(Color::Green),
        ));
    }

    // Numbered lists
    if trimmed
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_digit())
        && trimmed.contains(". ")
    {
        return Line::from(Span::styled(
            line.to_string(),
            Style::default().fg(Color::Green),
        ));
    }

    // Links (simplified detection)
    if trimmed.contains("](") || trimmed.starts_with("http") {
        return Line::from(Span::styled(
            line.to_string(),
            Style::default().fg(Color::Blue),
        ));
    }

    // Blockquotes
    if trimmed.starts_with("> ") {
        return Line::from(Span::styled(
            line.to_string(),
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        ));
    }

    // Regular text
    Line::from(line.to_string())
}

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
        format!(
            " q: Quit | g: Details | /: Search | r: Reload{docs_hint} | ↑↓: Navigate "
        )
    } else {
        format!(
            " q: Quit | g: Graph | /: Search | r: Reload{docs_hint} | ↑↓: Navigate | ←→: Expand/Collapse "
        )
    };
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default());
    frame.render_widget(help, area);
}
