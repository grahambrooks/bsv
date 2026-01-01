use crate::docs::{parse_docs_refs, DocsBrowser, DocsRef};
use crate::entity::{EntityIndex, EntityWithSource};
use crate::graph::RelationshipGraph;
use crate::parser::load_all_entities;
use crate::tree::{EntityTree, TreeNode, TreeState};
use anyhow::Result;
use std::path::{Path, PathBuf};

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
}
