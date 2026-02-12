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
use crate::parser::load_all_entities;
use crate::tree::{EntityTree, TreeNode, TreeState};
use anyhow::Result;
use std::path::{Path, PathBuf};

pub enum InputMode {
    Normal,
    Search,
    DocsBrowser,
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
    pub docs_browser: Option<DocsBrowser>,
    root_path: PathBuf,
}

impl App {
    /// Create a new app by loading entities from the given path.
    ///
    /// Root categories are expanded by default for immediate visibility.
    pub fn new(root: &Path) -> Result<Self> {
        let entities = load_all_entities(root)?;
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
            docs_browser: None,
            root_path: root.to_path_buf(),
        })
    }

    /// Reload all entities from disk and reset state.
    ///
    /// Useful when catalog files have changed and need to be re-parsed.
    pub fn reload(&mut self) {
        if let Ok(entities) = load_all_entities(&self.root_path) {
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
            self.docs_browser = None;
        }
    }

    pub fn toggle_graph(&mut self) {
        self.show_graph = !self.show_graph;
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

    pub fn get_relationship_graph(&self) -> Option<RelationshipGraph> {
        self.selected_entity()
            .map(|e| RelationshipGraph::build(e, &self.entities))
    }

    /// Get visible nodes filtered by search query if active.
    pub fn visible_nodes(&self) -> Vec<&TreeNode> {
        let nodes = self.tree.visible_nodes(&self.tree_state);
        if self.search_query.is_empty() {
            nodes
        } else {
            let query = self.search_query.to_lowercase();
            nodes
                .into_iter()
                .filter(|n| n.label.to_lowercase().contains(&query))
                .collect()
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
        }
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
