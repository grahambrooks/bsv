use ratatui::style::{Color, Modifier, Style};

// Tree symbols
pub const EXPANDED_SYMBOL: &str = "[-] ";
pub const COLLAPSED_SYMBOL: &str = "[+] ";
pub const LEAF_INDENT: &str = "    ";
pub const TREE_INDENT: &str = "  ";
pub const ERROR_INDICATOR: &str = " ⚠ ";

// Doc browser indicators
pub const SELECTED_INDICATOR: &str = "_";

// Colors and styles
pub fn selected_style() -> Style {
    Style::default()
        .bg(Color::Blue)
        .fg(Color::White)
        .add_modifier(Modifier::BOLD)
}

pub fn error_style() -> Style {
    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
}

pub fn category_style() -> Style {
    Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD)
}

pub fn normal_style() -> Style {
    Style::default().fg(Color::White)
}

pub fn border_style() -> Style {
    Style::default().fg(Color::Cyan)
}

pub fn label_style() -> Style {
    Style::default().fg(Color::Yellow)
}

pub fn dimmed_style() -> Style {
    Style::default().fg(Color::DarkGray)
}
