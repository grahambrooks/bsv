use crate::parser::should_exclude_dir;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Documentation reference parsed from annotations
#[derive(Debug, Clone)]
pub struct DocsRef {
    pub ref_type: DocsRefType,
    pub path: PathBuf,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DocsRefType {
    TechDocs,
    Adr,
}

impl DocsRefType {
    pub fn label(&self) -> &'static str {
        match self {
            DocsRefType::TechDocs => "TechDocs",
            DocsRefType::Adr => "ADR",
        }
    }
}

/// A markdown documentation file
#[derive(Debug, Clone)]
pub struct DocFile {
    pub path: PathBuf,
    pub name: String,
    pub relative_path: String,
}

/// Browser state for documentation viewing
#[derive(Debug, Clone)]
pub struct DocsBrowser {
    pub docs_ref: DocsRef,
    pub files: Vec<DocFile>,
    pub selected_index: usize,
    pub viewing_content: Option<DocContent>,
    pub scroll_offset: usize,
}

#[derive(Debug, Clone)]
pub struct DocContent {
    pub file: DocFile,
    pub lines: Vec<String>,
}

impl DocsBrowser {
    pub fn new(docs_ref: DocsRef) -> Self {
        let files = discover_doc_files(&docs_ref.path);
        Self {
            docs_ref,
            files,
            selected_index: 0,
            viewing_content: None,
            scroll_offset: 0,
        }
    }

    pub fn move_up(&mut self) {
        if self.viewing_content.is_some() {
            // Scroll up in content view
            if self.scroll_offset > 0 {
                self.scroll_offset -= 1;
            }
        } else if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn move_down(&mut self, visible_height: usize) {
        if let Some(content) = &self.viewing_content {
            // Scroll down in content view
            let max_scroll = content.lines.len().saturating_sub(visible_height);
            if self.scroll_offset < max_scroll {
                self.scroll_offset += 1;
            }
        } else if self.selected_index < self.files.len().saturating_sub(1) {
            self.selected_index += 1;
        }
    }

    pub fn page_up(&mut self, page_size: usize) {
        if self.viewing_content.is_some() {
            self.scroll_offset = self.scroll_offset.saturating_sub(page_size);
        }
    }

    pub fn page_down(&mut self, visible_height: usize, page_size: usize) {
        if let Some(content) = &self.viewing_content {
            let max_scroll = content.lines.len().saturating_sub(visible_height);
            self.scroll_offset = (self.scroll_offset + page_size).min(max_scroll);
        }
    }

    pub fn open_selected(&mut self) {
        if self.viewing_content.is_some() {
            return;
        }

        if let Some(file) = self.files.get(self.selected_index) {
            if let Ok(content) = fs::read_to_string(&file.path) {
                let lines: Vec<String> = content.lines().map(String::from).collect();
                self.viewing_content = Some(DocContent {
                    file: file.clone(),
                    lines,
                });
                self.scroll_offset = 0;
            }
        }
    }

    pub fn close_content(&mut self) {
        self.viewing_content = None;
        self.scroll_offset = 0;
    }

    pub fn is_viewing_content(&self) -> bool {
        self.viewing_content.is_some()
    }
}

/// Parse documentation references from entity annotations
pub fn parse_docs_refs(annotations: &HashMap<String, String>, source_file: &Path) -> Vec<DocsRef> {
    let mut refs = Vec::new();
    let source_dir = source_file.parent().unwrap_or(Path::new("."));

    for (key, value) in annotations {
        // TechDocs annotation
        if key == "backstage.io/techdocs-ref" {
            if let Some(path) = parse_techdocs_ref(value, source_dir) {
                refs.push(DocsRef {
                    ref_type: DocsRefType::TechDocs,
                    path,
                });
            }
        }

        // ADR location annotation
        if key == "backstage.io/adr-location" || key.contains("adr") {
            let path = resolve_relative_path(value, source_dir);
            if path.exists() {
                refs.push(DocsRef {
                    ref_type: DocsRefType::Adr,
                    path,
                });
            }
        }
    }

    refs
}

/// Parse techdocs-ref annotation value
/// Formats: "dir:." "dir:./docs" "url:https://..."
fn parse_techdocs_ref(value: &str, source_dir: &Path) -> Option<PathBuf> {
    if let Some(dir_path) = value.strip_prefix("dir:") {
        let path = resolve_relative_path(dir_path, source_dir);
        if path.exists() {
            return Some(path);
        }
    }

    // Try as plain path
    let path = resolve_relative_path(value, source_dir);
    if path.exists() {
        return Some(path);
    }

    None
}

/// Resolve a relative path against a base directory
fn resolve_relative_path(path_str: &str, base_dir: &Path) -> PathBuf {
    let path = Path::new(path_str);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base_dir.join(path)
    }
}

/// Discover markdown files in a documentation directory
fn discover_doc_files(docs_path: &Path) -> Vec<DocFile> {
    let mut files = Vec::new();

    if !docs_path.exists() {
        return files;
    }

    // If it's a file, just return that
    if docs_path.is_file() {
        if is_markdown_file(docs_path) {
            files.push(DocFile {
                path: docs_path.to_path_buf(),
                name: docs_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                relative_path: docs_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
            });
        }
        return files;
    }

    // Recursively find markdown files
    collect_markdown_files(docs_path, docs_path, &mut files);

    // Sort by path for consistent ordering
    files.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));

    files
}

fn collect_markdown_files(base_path: &Path, current_path: &Path, files: &mut Vec<DocFile>) {
    if let Ok(entries) = fs::read_dir(current_path) {
        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                // Skip hidden directories and build output directories
                let should_skip = path
                    .file_name()
                    .map(|n| should_exclude_dir(&n.to_string_lossy()))
                    .unwrap_or(false);

                if !should_skip {
                    collect_markdown_files(base_path, &path, files);
                }
            } else if is_markdown_file(&path) {
                let relative = path
                    .strip_prefix(base_path)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .to_string();

                files.push(DocFile {
                    path: path.clone(),
                    name: path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                    relative_path: relative,
                });
            }
        }
    }
}

fn is_markdown_file(path: &Path) -> bool {
    path.extension()
        .map(|ext| ext == "md" || ext == "markdown")
        .unwrap_or(false)
}
