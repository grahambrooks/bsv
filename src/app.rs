//! Application state management and navigation logic.
//!
//! This module manages the overall application state including the entity tree, selection,
//! search state, relationship graph view, and documentation browser. It provides methods
//! for navigation (up/down), expansion/collapse, search, and mode switching between normal
//! view, graph view, and docs browser.
//!
//! # Examples
//!
//! ## Creating and Using the App
//!
//! ```no_run
//! use bsv::app::App;
//! use std::path::Path;
//!
//! let mut app = App::new(Path::new("."))?;
//! println!("Loaded {} entities", app.entity_count);
//!
//! // Navigate through entities
//! app.move_down();
//! app.move_down();
//! app.toggle_expand();
//!
//! // Get currently selected entity
//! if let Some(entity) = app.selected_entity() {
//!     println!("Selected: {}", entity.entity.display_name());
//! }
//! # Ok::<(), anyhow::Error>(())
//! ```
//!
//! ## Search Functionality
//!
//! ```no_run
//! # use bsv::app::App;
//! # use std::path::Path;
//! # let mut app = App::new(Path::new("."))?;
//! // Start search mode
//! app.start_search();
//! app.search_input('u');
//! app.search_input('s');
//! app.search_input('e');
//! app.search_input('r');
//!
//! // Get filtered results
//! let visible = app.visible_nodes();
//! println!("Found {} matches", visible.len());
//!
//! // Confirm search (exits input mode but keeps filter)
//! app.confirm_search();
//! # Ok::<(), anyhow::Error>(())
//! ```
//!
//! ## Viewing Relationship Graph
//!
//! ```no_run
//! # use bsv::app::App;
//! # use std::path::Path;
//! # let mut app = App::new(Path::new("."))?;
//! # app.move_down();
//! // Toggle graph view for selected entity
//! app.toggle_graph();
//!
//! if app.show_graph {
//!     if let Some(graph) = app.get_relationship_graph() {
//!         println!("Center: {}", graph.center.display_name);
//!         println!("Outgoing relationships: {}", graph.outgoing.len());
//!         println!("Incoming relationships: {}", graph.incoming.len());
//!     }
//! }
//! # Ok::<(), anyhow::Error>(())
//! ```
//!
//! ## Documentation Browser
//!
//! ```no_run
//! # use bsv::app::App;
//! # use std::path::Path;
//! # let mut app = App::new(Path::new("."))?;
//! // Check if selected entity has docs
//! let docs_refs = app.get_docs_refs();
//! if !docs_refs.is_empty() {
//!     app.open_docs();
//!     assert!(app.is_docs_active());
//! }
//!
//! // Close docs browser
//! app.close_docs();
//! # Ok::<(), anyhow::Error>(())
//! ```
//!
//! ## Reloading Entities
//!
//! ```no_run
//! # use bsv::app::App;
//! # use std::path::Path;
//! # let mut app = App::new(Path::new("."))?;
//! // Reload entities from disk (e.g., after file changes)
//! app.reload();
//! println!("Reloaded {} entities", app.entity_count);
//! # Ok::<(), anyhow::Error>(())
//! ```
//!
//! # Key Types
//!
//! - [`App`] - Main application state container
//! - [`InputMode`] - Current input mode (Normal, Search, DocsBrowser)

use crate::docs::{parse_docs_refs, DocsBrowser, DocsRef};
use crate::entity::{EntityIndex, EntityWithSource};
use crate::graph::RelationshipGraph;
use crate::parser::load_catalog;
use crate::tree::{EntityTree, TreeNode, TreeState};
use anyhow::Result;
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub enum InputMode {
    Normal,
    Search,
    DocsBrowser,
}

/// Which pane currently receives navigation keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Focus {
    /// The entity tree on the left (default).
    #[default]
    Tree,
    /// The details/graph panel on the right (scrollable).
    Detail,
}

pub struct App {
    pub tree: EntityTree,
    pub tree_state: TreeState,
    pub should_quit: bool,
    pub entity_count: usize,
    pub search_query: String,
    pub search_active: bool,
    pub entity_index: EntityIndex,
    pub entities: Vec<EntityWithSource>,
    pub show_graph: bool,
    pub show_raw: bool,
    pub focus: Focus,
    /// Vertical scroll offset (in rows) for the right-hand detail/graph panel.
    pub detail_scroll: u16,
    /// Non-fatal messages from the last load (unparsable docs, reload failures).
    pub load_warnings: Vec<String>,
    /// Whether the keyboard-shortcut help overlay is showing.
    pub show_help: bool,
    pub docs_browser: Option<DocsBrowser>,
    /// Lazily-built relationship graph for the selected entity, keyed by its
    /// node id so it is reused across frames instead of rebuilt every draw.
    relationship_cache: RefCell<Option<(usize, Rc<RelationshipGraph>)>>,
    root_path: PathBuf,
}

impl App {
    /// Create a new app by loading entities from the given path.
    ///
    /// Root categories are expanded by default for immediate visibility.
    pub fn new(root: &Path) -> Result<Self> {
        let (entities, load_warnings) = load_catalog(root)?;
        let entity_count = entities.len();
        let entity_index = EntityIndex::build(&entities);
        let tree = EntityTree::build(entities.clone());

        let mut tree_state = TreeState::new();
        // Expand root categories by default
        for &root_id in &tree.root_children {
            tree_state.expanded.insert(root_id);
        }

        Ok(Self {
            tree,
            tree_state,
            should_quit: false,
            entity_count,
            search_query: String::new(),
            search_active: false,
            entity_index,
            entities,
            show_graph: false,
            show_raw: false,
            focus: Focus::Tree,
            detail_scroll: 0,
            load_warnings,
            show_help: false,
            docs_browser: None,
            relationship_cache: RefCell::new(None),
            root_path: root.to_path_buf(),
        })
    }

    /// Reload all entities from disk and reset state.
    ///
    /// Useful when catalog files have changed and need to be re-parsed.
    pub fn reload(&mut self) {
        match load_catalog(&self.root_path) {
            Ok((entities, warnings)) => {
                self.entity_count = entities.len();
                self.entity_index = EntityIndex::build(&entities);
                self.tree = EntityTree::build(entities.clone());
                self.entities = entities;
                self.tree_state = TreeState::new();
                // Expand root categories by default
                for &root_id in &self.tree.root_children {
                    self.tree_state.expanded.insert(root_id);
                }
                self.search_query.clear();
                self.search_active = false;
                self.show_graph = false;
                self.show_raw = false;
                self.focus = Focus::Tree;
                self.detail_scroll = 0;
                self.load_warnings = warnings;
                self.docs_browser = None;
                self.relationship_cache = RefCell::new(None);
            }
            Err(e) => {
                // Keep the current catalog but make the failure visible.
                self.load_warnings = vec![format!("Reload failed: {e}")];
            }
        }
    }

    /// Toggle the keyboard-shortcut help overlay.
    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn toggle_graph(&mut self) {
        self.show_graph = !self.show_graph;
        self.detail_scroll = 0;
    }

    /// Toggle the details panel between formatted details and the raw YAML
    /// definition of the selected entity.
    pub fn toggle_raw(&mut self) {
        self.show_raw = !self.show_raw;
        self.detail_scroll = 0;
    }

    /// Move keyboard focus between the tree and the detail/graph panel.
    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Tree => Focus::Detail,
            Focus::Detail => Focus::Tree,
        };
    }

    /// Return focus to the tree (e.g. on Esc).
    pub fn focus_tree(&mut self) {
        self.focus = Focus::Tree;
    }

    /// Esc behaviour in normal mode: return focus to the tree and clear any
    /// active search filter.
    pub fn focus_tree_and_clear_search(&mut self) {
        self.focus = Focus::Tree;
        self.clear_search();
    }

    pub fn is_detail_focused(&self) -> bool {
        self.focus == Focus::Detail
    }

    /// Scroll the detail panel up by `rows`, clamping at the top.
    pub fn scroll_detail_up(&mut self, rows: u16) {
        self.detail_scroll = self.detail_scroll.saturating_sub(rows);
    }

    /// Scroll the detail panel down by `rows`, clamping so `max` rows stay the
    /// furthest the panel can scroll (content length minus visible height).
    pub fn scroll_detail_down(&mut self, rows: u16, max: u16) {
        self.detail_scroll = self.detail_scroll.saturating_add(rows).min(max);
    }

    /// Jump the detail panel to the top.
    pub fn scroll_detail_home(&mut self) {
        self.detail_scroll = 0;
    }

    /// Jump the detail panel to the bottom (`max` = content length minus height).
    pub fn scroll_detail_end(&mut self, max: u16) {
        self.detail_scroll = max;
    }

    /// Check if the current entity has documentation references
    pub fn get_docs_refs(&self) -> Vec<DocsRef> {
        self.selected_entity()
            .map(|e| parse_docs_refs(&e.entity.metadata.annotations, &e.source_file))
            .unwrap_or_default()
    }

    /// Open the documentation browser for the selected entity
    pub fn open_docs(&mut self) {
        let refs = self.get_docs_refs();
        if let Some(docs_ref) = refs.into_iter().next() {
            self.docs_browser = Some(DocsBrowser::new(docs_ref));
        }
    }

    /// Close the documentation browser
    pub fn close_docs(&mut self) {
        if let Some(browser) = &mut self.docs_browser {
            if browser.is_viewing_content() {
                browser.close_content();
            } else {
                self.docs_browser = None;
            }
        }
    }

    /// Check if docs browser is active
    pub fn is_docs_active(&self) -> bool {
        self.docs_browser.is_some()
    }

    /// Relationship graph for the selected entity, cached by node id so it is
    /// built once per selection rather than on every render frame.
    pub fn relationship_graph(&self) -> Option<Rc<RelationshipGraph>> {
        let selected = self.tree_state.selected;

        if let Some((id, graph)) = self.relationship_cache.borrow().as_ref() {
            if *id == selected {
                return Some(Rc::clone(graph));
            }
        }

        let entity = self.selected_entity()?;
        let graph = Rc::new(RelationshipGraph::build(entity, &self.entities));
        *self.relationship_cache.borrow_mut() = Some((selected, Rc::clone(&graph)));
        Some(graph)
    }

    /// Owned copy of the relationship graph for the selected entity.
    pub fn get_relationship_graph(&self) -> Option<RelationshipGraph> {
        self.relationship_graph().map(|g| (*g).clone())
    }

    /// Get visible nodes filtered by search query if active.
    pub fn visible_nodes(&self) -> Vec<&TreeNode> {
        let nodes = self.tree.visible_nodes(&self.tree_state);
        if self.search_query.is_empty() {
            nodes
        } else {
            EntityTree::filter_by_search(nodes, &self.search_query)
        }
    }

    pub fn move_up(&mut self) {
        let visible = self.visible_nodes();
        if visible.is_empty() {
            return;
        }

        let current_idx = visible
            .iter()
            .position(|n| n.id == self.tree_state.selected)
            .unwrap_or(0);

        if current_idx > 0 {
            self.tree_state.selected = visible[current_idx - 1].id;
            self.detail_scroll = 0;
        }
    }

    pub fn move_down(&mut self) {
        let visible = self.visible_nodes();
        if visible.is_empty() {
            return;
        }

        let current_idx = visible
            .iter()
            .position(|n| n.id == self.tree_state.selected)
            .unwrap_or(0);

        if current_idx < visible.len() - 1 {
            self.tree_state.selected = visible[current_idx + 1].id;
            self.detail_scroll = 0;
        }
    }

    /// Move the selection up by one page (`page` rows), clamping at the top.
    pub fn page_up(&mut self, page: usize) {
        let visible = self.visible_nodes();
        if visible.is_empty() {
            return;
        }

        let current_idx = visible
            .iter()
            .position(|n| n.id == self.tree_state.selected)
            .unwrap_or(0);

        let new_idx = current_idx.saturating_sub(page.max(1));
        self.tree_state.selected = visible[new_idx].id;
        self.detail_scroll = 0;
    }

    /// Select the first visible node.
    pub fn move_home(&mut self) {
        if let Some(first) = self.visible_nodes().first() {
            self.tree_state.selected = first.id;
            self.detail_scroll = 0;
        }
    }

    /// Select the last visible node.
    pub fn move_end(&mut self) {
        if let Some(last) = self.visible_nodes().last() {
            self.tree_state.selected = last.id;
            self.detail_scroll = 0;
        }
    }

    /// Move the selection down by one page (`page` rows), clamping at the bottom.
    pub fn page_down(&mut self, page: usize) {
        let visible = self.visible_nodes();
        if visible.is_empty() {
            return;
        }

        let current_idx = visible
            .iter()
            .position(|n| n.id == self.tree_state.selected)
            .unwrap_or(0);

        let new_idx = (current_idx + page.max(1)).min(visible.len() - 1);
        self.tree_state.selected = visible[new_idx].id;
        self.detail_scroll = 0;
    }

    /// Index of the currently selected node within the visible list, if any.
    ///
    /// Used to drive scrolling so the selected row stays on screen.
    pub fn selected_visible_index(&self) -> Option<usize> {
        self.visible_nodes()
            .iter()
            .position(|n| n.id == self.tree_state.selected)
    }

    pub fn toggle_expand(&mut self) {
        if let Some(node) = self.tree.get_node(self.tree_state.selected) {
            if !node.children.is_empty() {
                self.tree_state.toggle_expanded(self.tree_state.selected);
            }
        }
    }

    pub fn collapse(&mut self) {
        self.tree_state.expanded.remove(&self.tree_state.selected);
    }

    pub fn expand_all(&mut self) {
        self.tree_state.expand_all(&self.tree);
    }

    pub fn selected_entity(&self) -> Option<&EntityWithSource> {
        self.tree
            .get_node(self.tree_state.selected)
            .and_then(|n| n.entity.as_ref())
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn start_search(&mut self) {
        self.search_active = true;
    }

    pub fn cancel_search(&mut self) {
        self.search_active = false;
        self.search_query.clear();
    }

    pub fn confirm_search(&mut self) {
        self.search_active = false;
        // Keep query active but exit input mode
        // Select first visible match if current selection is not visible
        let visible = self.visible_nodes();
        if !visible.iter().any(|n| n.id == self.tree_state.selected) {
            if let Some(first) = visible.first() {
                self.tree_state.selected = first.id;
            }
        }
    }

    pub fn search_input(&mut self, c: char) {
        self.search_query.push(c);
        self.update_selection_for_search();
    }

    pub fn search_backspace(&mut self) {
        self.search_query.pop();
    }

    fn update_selection_for_search(&mut self) {
        let visible = self.visible_nodes();
        if !visible.iter().any(|n| n.id == self.tree_state.selected) {
            if let Some(first) = visible.first() {
                self.tree_state.selected = first.id;
            }
        }
    }

    pub fn clear_search(&mut self) {
        self.search_query.clear();
    }

    pub fn input_mode(&self) -> InputMode {
        if self.search_active {
            InputMode::Search
        } else if self.is_docs_active() {
            InputMode::DocsBrowser
        } else {
            InputMode::Normal
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn test_app() -> App {
        App::new(Path::new("testdata/large-catalog.yaml")).expect("load test catalog")
    }

    #[test]
    fn move_up_at_top_is_clamped() {
        let mut app = test_app();
        let first = app.tree_state.selected;
        app.move_up();
        assert_eq!(
            app.tree_state.selected, first,
            "should not move above the top"
        );
    }

    #[test]
    fn home_and_end_jump_to_bounds() {
        let mut app = test_app();
        app.expand_all();
        app.move_end();
        let last = app.visible_nodes().last().unwrap().id;
        assert_eq!(app.tree_state.selected, last);
        app.move_home();
        let first = app.visible_nodes().first().unwrap().id;
        assert_eq!(app.tree_state.selected, first);
    }

    #[test]
    fn focus_toggles_between_tree_and_detail() {
        let mut app = test_app();
        assert!(!app.is_detail_focused());
        app.toggle_focus();
        assert!(app.is_detail_focused());
        app.toggle_focus();
        assert!(!app.is_detail_focused());
        app.toggle_focus();
        app.focus_tree();
        assert!(!app.is_detail_focused());
    }

    #[test]
    fn detail_scroll_clamps_at_both_ends() {
        let mut app = test_app();
        app.scroll_detail_up(5);
        assert_eq!(app.detail_scroll, 0, "cannot scroll above the top");

        app.scroll_detail_down(10, 3);
        assert_eq!(app.detail_scroll, 3, "clamped to max");

        app.scroll_detail_down(10, 3);
        assert_eq!(app.detail_scroll, 3, "stays at max");

        app.scroll_detail_up(1);
        assert_eq!(app.detail_scroll, 2);

        app.scroll_detail_end(7);
        assert_eq!(app.detail_scroll, 7);

        app.scroll_detail_home();
        assert_eq!(app.detail_scroll, 0);
    }

    /// Advance the selection to the next node that has an entity, returning true
    /// if one was found.
    fn select_next_entity(app: &mut App) -> bool {
        for _ in 0..app.visible_nodes().len() {
            if app.selected_entity().is_some() {
                return true;
            }
            app.move_down();
        }
        app.selected_entity().is_some()
    }

    #[test]
    fn relationship_graph_is_cached_and_invalidated() {
        let mut app = test_app();
        app.expand_all();
        assert!(select_next_entity(&mut app), "found an entity to select");

        let g1 = app.relationship_graph().expect("graph for entity");
        let g2 = app.relationship_graph().expect("graph for entity");
        assert!(
            Rc::ptr_eq(&g1, &g2),
            "same selection reuses the cached graph"
        );
        let first_center = g1.center.display_name.clone();

        // Move to a different entity; the cache must rebuild for the new center.
        app.move_down();
        assert!(select_next_entity(&mut app), "found a second entity");
        let g3 = app.relationship_graph().expect("graph for second entity");
        assert!(!Rc::ptr_eq(&g1, &g3), "new selection rebuilds the graph");
        assert_ne!(
            first_center, g3.center.display_name,
            "graph centre tracks the selection"
        );
    }

    #[test]
    fn toggle_help_flips_flag() {
        let mut app = test_app();
        assert!(!app.show_help);
        app.toggle_help();
        assert!(app.show_help);
        app.toggle_help();
        assert!(!app.show_help);
    }

    #[test]
    fn navigation_resets_detail_scroll() {
        let mut app = test_app();
        app.expand_all();
        app.detail_scroll = 9;
        app.move_down();
        assert_eq!(app.detail_scroll, 0, "moving selection resets panel scroll");

        app.detail_scroll = 9;
        app.toggle_raw();
        assert_eq!(app.detail_scroll, 0, "toggling raw resets panel scroll");
    }
}
