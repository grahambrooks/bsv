use crate::app::App;
use crate::entity::{EntityIndex, EntityKind, EntityRef, EntityWithSource};
use crate::ui::theme::*;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn draw_details(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Details ")
        .borders(Borders::ALL)
        .border_style(border_style());

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
            .style(dimmed_style());
        frame.render_widget(paragraph, area);
    }
}

fn format_entity_details(ews: &EntityWithSource, index: &EntityIndex, all_entities: &[EntityWithSource]) -> Vec<Line<'static>> {
    let entity = &ews.entity;
    let mut lines = Vec::new();

    // Header
    lines.push(Line::from(vec![
        Span::styled("Kind: ", label_style()),
        Span::styled(
            entity.kind.to_string(),
            Style::default().add_modifier(Modifier::BOLD),
        ),
    ]));

    lines.push(Line::from(vec![
        Span::styled("Name: ", label_style()),
        Span::styled(
            entity.metadata.name.clone(),
            Style::default().add_modifier(Modifier::BOLD),
        ),
    ]));

    if let Some(title) = &entity.metadata.title {
        lines.push(Line::from(vec![
            Span::styled("Title: ", label_style()),
            Span::raw(title.clone()),
        ]));
    }

    if let Some(ns) = &entity.metadata.namespace {
        lines.push(Line::from(vec![
            Span::styled("Namespace: ", label_style()),
            Span::raw(ns.clone()),
        ]));
    }

    lines.push(Line::from(""));

    // Description
    if let Some(desc) = &entity.metadata.description {
        lines.push(Line::from(Span::styled(
            "Description:",
            label_style(),
        )));
        lines.push(Line::from(desc.clone()));
        lines.push(Line::from(""));
    }

    // Spec details with reference validation
    if let Some(owner) = entity.owner() {
        let ref_line = format_entity_ref(&owner, "group", index);
        lines.push(Line::from(
            std::iter::once(Span::styled("Owner: ", label_style()))
                .chain(ref_line)
                .collect::<Vec<_>>(),
        ));
    }

    if let Some(system) = entity.system() {
        let ref_line = format_entity_ref(&system, "system", index);
        lines.push(Line::from(
            std::iter::once(Span::styled("System: ", label_style()))
                .chain(ref_line)
                .collect::<Vec<_>>(),
        ));
    }

    if let Some(domain) = entity.domain() {
        let ref_line = format_entity_ref(&domain, "domain", index);
        lines.push(Line::from(
            std::iter::once(Span::styled("Domain: ", label_style()))
                .chain(ref_line)
                .collect::<Vec<_>>(),
        ));
    }

    if let Some(lifecycle) = entity.lifecycle() {
        lines.push(Line::from(vec![
            Span::styled("Lifecycle: ", label_style()),
            Span::raw(lifecycle),
        ]));
    }

    if let Some(etype) = entity.entity_type() {
        lines.push(Line::from(vec![
            Span::styled("Type: ", label_style()),
            Span::raw(etype),
        ]));
    }

    // Group-specific information
    if matches!(entity.kind, EntityKind::Group) {
        format_group_details(entity, index, all_entities, &mut lines);
    }

    // Tags
    if !entity.metadata.tags.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Tags:",
            label_style(),
        )));
        lines.push(Line::from(entity.metadata.tags.join(", ")));
    }

    // Links
    if !entity.metadata.links.is_empty() {
        format_links(&entity.metadata.links, &mut lines);
    }

    // Annotations
    if !entity.metadata.annotations.is_empty() {
        format_annotations(&entity.metadata.annotations, &mut lines);
    }

    // Source file
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("Source: ", dimmed_style()),
        Span::styled(
            ews.source_file.display().to_string(),
            dimmed_style(),
        ),
    ]));

    // Validation errors
    if !ews.validation_errors.is_empty() {
        format_validation_errors(&ews.validation_errors, &mut lines);
    }

    lines
}

fn format_group_details(
    entity: &crate::entity::Entity,
    index: &EntityIndex,
    all_entities: &[EntityWithSource],
    lines: &mut Vec<Line<'static>>,
) {
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "─── Group Hierarchy ───",
        Style::default().fg(Color::Magenta),
    )));

    // Parent group
    if let Some(parent) = entity.get_spec_string("parent") {
        let ref_line = format_entity_ref(&parent, "group", index);
        lines.push(Line::from(
            std::iter::once(Span::styled("Parent: ", label_style()))
                .chain(ref_line)
                .collect::<Vec<_>>(),
        ));
    } else {
        lines.push(Line::from(Span::styled(
            "Parent: (none - root group)",
            dimmed_style(),
        )));
    }

    // Child groups
    if let Some(children) = entity.spec.get("children") {
        if let Some(children_arr) = children.as_sequence() {
            if !children_arr.is_empty() {
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    format!("Child Groups ({}):", children_arr.len()),
                    label_style(),
                )));
                for child in children_arr {
                    if let Some(child_str) = child.as_str() {
                        let ref_line = format_entity_ref(child_str, "group", index);
                        lines.push(Line::from(
                            std::iter::once(Span::styled("  └─ ", dimmed_style()))
                                .chain(ref_line)
                                .collect::<Vec<_>>(),
                        ));
                    }
                }
            }
        }
    }

    // Members (users who have memberOf pointing to this group)
    format_group_members(entity, all_entities, lines);
}

fn format_group_members(
    entity: &crate::entity::Entity,
    all_entities: &[EntityWithSource],
    lines: &mut Vec<Line<'static>>,
) {
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
            Span::styled("Members: ", label_style()),
            Span::styled("(none)", dimmed_style()),
        ]));
    } else {
        lines.push(Line::from(Span::styled(
            format!("Members ({}):", members.len()),
            label_style(),
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
                Span::styled("  • ", dimmed_style()),
                Span::styled(
                    kind_label,
                    dimmed_style(),
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

fn format_links(links: &[crate::entity::Link], lines: &mut Vec<Line<'static>>) {
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Links:",
        label_style(),
    )));
    for link in links {
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
            Span::styled(format!("  {icon}"), dimmed_style()),
            Span::styled(title.to_string(), Style::default().fg(Color::Cyan)),
            Span::styled(format!(" ({url})"), dimmed_style()),
        ]));
    }
}

fn format_annotations(
    annotations: &std::collections::HashMap<String, String>,
    lines: &mut Vec<Line<'static>>,
) {
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Annotations:",
        label_style(),
    )));

    let mut sorted_annotations: Vec<_> = annotations.iter().collect();
    sorted_annotations.sort_by_key(|(k, _)| *k);

    for (key, value) in sorted_annotations {
        let is_docs_annotation = key.contains("techdocs") || key.contains("adr");
        let key_style = if is_docs_annotation {
            Style::default().fg(Color::Magenta)
        } else {
            dimmed_style()
        };
        let value_style = if is_docs_annotation {
            Style::default().fg(Color::Magenta)
        } else {
            normal_style()
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

fn format_validation_errors(
    errors: &[crate::entity::ValidationError],
    lines: &mut Vec<Line<'static>>,
) {
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!("⚠ Validation Errors ({}):", errors.len()),
        error_style(),
    )));
    
    for (idx, error) in errors.iter().enumerate() {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {}. ", idx + 1),
                Style::default().fg(Color::Red),
            ),
            Span::styled(
                format!("Field: {}", error.path),
                label_style(),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::styled("     ", Style::default()),
            Span::styled(
                error.message.clone(),
                normal_style(),
            ),
        ]));
    }
}

/// Format an entity reference with resolved kind/namespace and validation
/// 
/// Explicit parts shown in bright colors, inferred parts shown dim in \[brackets\]
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
