use crate::docs::DocsBrowser;
use crate::ui::theme::*;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

pub fn draw_docs_browser(frame: &mut Frame, browser: &DocsBrowser, area: Rect) {
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
            .style(dimmed_style());
        frame.render_widget(help, help_area);
    } else {
        // Show file list
        draw_docs_file_list(frame, browser, content_area);

        let help = Paragraph::new(" Esc: Close docs | Enter: Open file | ↑↓: Navigate ")
            .style(dimmed_style());
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
            .style(dimmed_style());
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
                normal_style()
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
        let scroll_span = Span::styled(scroll_info, dimmed_style());
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
