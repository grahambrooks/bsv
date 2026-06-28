//! Hierarchical tree structure for entity organization and display.
//!
//! This module builds a hierarchical tree from a flat list of entities, organizing them
//! by Domain → System → Components/APIs. The tree structure supports expansion/collapse
//! of nodes, search filtering, and efficient navigation. Entities without a system are
//! grouped under "Other Entities", and systems without a domain go under "Systems".
//!
//! # Examples
//!
//! ## Building and Navigating a Tree
//!
//! ```
//! use bsv::entity::{Entity, EntityKind, EntityWithSource, Metadata};
//! use bsv::tree::{EntityTree, TreeState};
//! use std::path::PathBuf;
//! use std::collections::HashMap;
//!
//! # let domain = Entity {
//! #     api_version: "backstage.io/v1alpha1".to_string(),
//! #     kind: EntityKind::Domain,
//! #     metadata: Metadata {
//! #         name: "platform".to_string(),
//! #         title: None,
//! #         namespace: Some("default".to_string()),
//! #         description: None,
//! #         labels: HashMap::new(),
//! #         annotations: HashMap::new(),
//! #         tags: Vec::new(),
//! #         links: Vec::new(),
//! #     },
//! #     spec: serde_yaml::Value::Null,
//! # };
//! let entities = vec![
//!     EntityWithSource::new(domain, PathBuf::from("catalog.yaml")),
//! ];
//!
//! let tree = EntityTree::build(&entities);
//! let mut state = TreeState::new();
//!
//! // Initially only root categories visible
//! let visible = tree.visible_nodes(&state);
//! println!("Visible: {} nodes", visible.len());
//!
//! // Expand first category
//! if let Some(&first_id) = tree.root_children.first() {
//!     state.toggle_expanded(first_id);
//!     let visible = tree.visible_nodes(&state);
//!     println!("After expand: {} nodes", visible.len());
//! }
//! ```
//!
//! ## Searching the Tree
//!
//! ```
//! # use bsv::entity::{Entity, EntityKind, EntityWithSource, Metadata};
//! # use bsv::tree::{EntityTree, TreeState};
//! # use std::path::PathBuf;
//! # use std::collections::HashMap;
//! # let component = Entity {
//! #     api_version: "backstage.io/v1alpha1".to_string(),
//! #     kind: EntityKind::Component,
//! #     metadata: Metadata {
//! #         name: "user-service".to_string(),
//! #         title: None,
//! #         namespace: Some("default".to_string()),
//! #         description: None,
//! #         labels: HashMap::new(),
//! #         annotations: HashMap::new(),
//! #         tags: Vec::new(),
//! #         links: Vec::new(),
//! #     },
//! #     spec: serde_yaml::Value::Null,
//! # };
//! # let entities = vec![EntityWithSource::new(component, PathBuf::from("catalog.yaml"))];
//! let tree = EntityTree::build(&entities);
//! let mut state = TreeState::new();
//! state.expand_all(&tree);
//! let visible = tree.visible_nodes(&state);
//!
//! // Filter by search query
//! let matches = EntityTree::filter_by_search(visible, "user");
//! println!("Found {} matches for 'user'", matches.len());
//! ```
//!
//! ## Expanding/Collapsing Nodes
//!
//! ```
//! # use bsv::entity::{Entity, EntityKind, EntityWithSource, Metadata};
//! # use bsv::tree::{EntityTree, TreeState};
//! # use std::path::PathBuf;
//! # use std::collections::HashMap;
//! # let entities = vec![];
//! let tree = EntityTree::build(&entities);
//! let mut state = TreeState::new();
//!
//! // Toggle expansion state
//! state.toggle_expanded(0);
//! assert!(state.is_expanded(0));
//!
//! state.toggle_expanded(0);
//! assert!(!state.is_expanded(0));
//!
//! // Expand all nodes with children
//! state.expand_all(&tree);
//! ```
//!
//! # Key Types
//!
//! - [`EntityTree`] - Hierarchical tree structure with all nodes
//! - [`TreeNode`] - Single node in the tree (category or entity)
//! - [`TreeState`] - Tracks which nodes are expanded and selected

use crate::entity::{EntityKind, EntityRef, EntityWithSource};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub id: usize,
    pub label: String,
    pub depth: usize,
    pub entity: Option<EntityWithSource>,
    pub children: Vec<usize>,
    pub is_category: bool,
}

#[derive(Debug)]
pub struct EntityTree {
    pub nodes: Vec<TreeNode>,
    pub root_children: Vec<usize>,
}

/// A visible tree node paired with its rendered branch-connector prefix.
#[derive(Debug)]
pub struct VisibleRow<'a> {
    pub node: &'a TreeNode,
    pub prefix: String,
}

#[derive(Debug)]
pub struct TreeState {
    pub selected: usize,
    pub expanded: HashSet<usize>,
}

impl TreeState {
    pub fn new() -> Self {
        Self {
            selected: 0,
            expanded: HashSet::new(),
        }
    }

    pub fn toggle_expanded(&mut self, id: usize) {
        if self.expanded.contains(&id) {
            self.expanded.remove(&id);
        } else {
            self.expanded.insert(id);
        }
    }

    pub fn is_expanded(&self, id: usize) -> bool {
        self.expanded.contains(&id)
    }

    pub fn expand_all(&mut self, tree: &EntityTree) {
        for node in &tree.nodes {
            if !node.children.is_empty() {
                self.expanded.insert(node.id);
            }
        }
    }
}

impl Default for TreeState {
    fn default() -> Self {
        Self::new()
    }
}

impl EntityTree {
    /// Build a hierarchical tree from a flat list of entities.
    ///
    /// Organizes entities as: Domain → System → Components/APIs/Resources.
    /// Entities without a system go under "Other Entities".
    /// Systems without a domain go under "Systems".
    pub fn build(entities: &[EntityWithSource]) -> Self {
        let mut nodes: Vec<TreeNode> = Vec::new();
        let mut root_children: Vec<usize> = Vec::new();

        // Group entities by kind, then by domain/system relationships
        let mut domains: HashMap<String, Vec<&EntityWithSource>> = HashMap::new();
        let mut systems: HashMap<String, Vec<&EntityWithSource>> = HashMap::new();
        let mut system_to_domain: HashMap<String, String> = HashMap::new();
        let mut components_by_system: HashMap<String, Vec<&EntityWithSource>> = HashMap::new();
        let mut ungrouped: Vec<&EntityWithSource> = Vec::new();

        // First pass: collect domains and systems
        for ews in entities {
            match ews.entity.kind {
                EntityKind::Domain => {
                    domains
                        .entry(ews.entity.metadata.name.clone())
                        .or_default()
                        .push(ews);
                }
                EntityKind::System => {
                    systems
                        .entry(ews.entity.metadata.name.clone())
                        .or_default()
                        .push(ews);
                    if let Some(domain) = ews.entity.domain() {
                        system_to_domain.insert(ews.entity.metadata.name.clone(), domain);
                    }
                }
                _ => {}
            }
        }

        // Second pass: group components/APIs/resources by system
        for ews in entities {
            match ews.entity.kind {
                EntityKind::Domain | EntityKind::System => {}
                EntityKind::Component | EntityKind::Api | EntityKind::Resource => {
                    if let Some(system) = ews.entity.system() {
                        components_by_system.entry(system).or_default().push(ews);
                    } else {
                        ungrouped.push(ews);
                    }
                }
                // Groups are organised into their own parent/child hierarchy below.
                EntityKind::Group => {}
                _ => {
                    ungrouped.push(ews);
                }
            }
        }

        // Build tree structure
        // Domains first
        if !domains.is_empty() {
            let domain_cat_id = nodes.len();
            nodes.push(TreeNode {
                id: domain_cat_id,
                label: "Domains".to_string(),
                depth: 0,
                entity: None,
                children: Vec::new(),
                is_category: true,
            });
            root_children.push(domain_cat_id);

            for domain_name in sorted_keys(&domains) {
                for ews in &domains[domain_name] {
                    let domain_id = nodes.len();
                    nodes.push(TreeNode {
                        id: domain_id,
                        label: format!("{}: {}", EntityKind::Domain, ews.entity.display_name()),
                        depth: 1,
                        entity: Some((*ews).clone()),
                        children: Vec::new(),
                        is_category: false,
                    });
                    nodes[domain_cat_id].children.push(domain_id);

                    // Add systems belonging to this domain, alphabetically.
                    let mut sys_names: Vec<&String> = systems
                        .keys()
                        .filter(|s| system_to_domain.get(*s) == Some(domain_name))
                        .collect();
                    sys_names.sort();
                    for sys_name in sys_names {
                        for sys_ews in &systems[sys_name] {
                            let sys_id = nodes.len();
                            nodes.push(TreeNode {
                                id: sys_id,
                                label: format!(
                                    "{}: {}",
                                    EntityKind::System,
                                    sys_ews.entity.display_name()
                                ),
                                depth: 2,
                                entity: Some((*sys_ews).clone()),
                                children: Vec::new(),
                                is_category: false,
                            });
                            nodes[domain_id].children.push(sys_id);

                            // Add components of this system, sorted.
                            if let Some(comps) = components_by_system.get(sys_name) {
                                for comp_ews in sorted_entities(comps) {
                                    let comp_id = nodes.len();
                                    nodes.push(TreeNode {
                                        id: comp_id,
                                        label: format!(
                                            "{}: {}",
                                            comp_ews.entity.kind,
                                            comp_ews.entity.display_name()
                                        ),
                                        depth: 3,
                                        entity: Some(comp_ews.clone()),
                                        children: Vec::new(),
                                        is_category: false,
                                    });
                                    nodes[sys_id].children.push(comp_id);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Systems without domains (or with non-existent domain references)
        let mut orphan_system_names: Vec<&String> = systems
            .keys()
            .filter(|name| match system_to_domain.get(*name) {
                None => true,                                            // No domain reference
                Some(domain_name) => !domains.contains_key(domain_name), // Domain doesn't exist
            })
            .collect();
        orphan_system_names.sort();

        if !orphan_system_names.is_empty() {
            let sys_cat_id = nodes.len();
            nodes.push(TreeNode {
                id: sys_cat_id,
                label: "Systems".to_string(),
                depth: 0,
                entity: None,
                children: Vec::new(),
                is_category: true,
            });
            root_children.push(sys_cat_id);

            for sys_name in orphan_system_names {
                for ews in &systems[sys_name] {
                    let sys_id = nodes.len();
                    nodes.push(TreeNode {
                        id: sys_id,
                        label: format!("{}: {}", EntityKind::System, ews.entity.display_name()),
                        depth: 1,
                        entity: Some((*ews).clone()),
                        children: Vec::new(),
                        is_category: false,
                    });
                    nodes[sys_cat_id].children.push(sys_id);

                    // Add components of this system, sorted.
                    if let Some(comps) = components_by_system.get(sys_name) {
                        for comp_ews in sorted_entities(comps) {
                            let comp_id = nodes.len();
                            nodes.push(TreeNode {
                                id: comp_id,
                                label: format!(
                                    "{}: {}",
                                    comp_ews.entity.kind,
                                    comp_ews.entity.display_name()
                                ),
                                depth: 2,
                                entity: Some(comp_ews.clone()),
                                children: Vec::new(),
                                is_category: false,
                            });
                            nodes[sys_id].children.push(comp_id);
                        }
                    }
                }
            }
        }

        // Groups: nest child groups under their parents (spec.parent / spec.children).
        let group_map: HashMap<String, EntityWithSource> = entities
            .iter()
            .filter(|ews| ews.entity.kind == EntityKind::Group)
            .map(|ews| (ews.entity.metadata.name.clone(), ews.clone()))
            .collect();

        if !group_map.is_empty() {
            // Resolve each group's effective parent, preferring spec.parent and
            // falling back to any group that lists it in spec.children.
            let mut parent_of: HashMap<String, String> = HashMap::new();
            for (name, ews) in &group_map {
                if let Some(parent) = ews.entity.parent() {
                    let parent_name = EntityRef::parse(&parent, "group").name;
                    if group_map.contains_key(&parent_name) {
                        parent_of.insert(name.clone(), parent_name);
                    }
                }
            }
            for (name, ews) in &group_map {
                for child in ews.entity.children() {
                    let child_name = EntityRef::parse(&child, "group").name;
                    if group_map.contains_key(&child_name) {
                        parent_of.entry(child_name).or_insert_with(|| name.clone());
                    }
                }
            }

            // Invert into a parent -> sorted children map for stable ordering.
            let mut children_of: HashMap<String, Vec<String>> = HashMap::new();
            for (child, parent) in &parent_of {
                children_of
                    .entry(parent.clone())
                    .or_default()
                    .push(child.clone());
            }
            for kids in children_of.values_mut() {
                kids.sort();
            }

            // Root groups have no resolved parent within the catalog.
            let mut roots: Vec<String> = group_map
                .keys()
                .filter(|name| !parent_of.contains_key(*name))
                .cloned()
                .collect();
            roots.sort();

            let group_cat_id = nodes.len();
            nodes.push(TreeNode {
                id: group_cat_id,
                label: "Groups".to_string(),
                depth: 0,
                entity: None,
                children: Vec::new(),
                is_category: true,
            });
            root_children.push(group_cat_id);

            let mut visited: HashSet<String> = HashSet::new();
            for root_name in &roots {
                if let Some(child_id) = Self::build_group_subtree(
                    &mut nodes,
                    &group_map,
                    &children_of,
                    root_name,
                    1,
                    &mut visited,
                ) {
                    nodes[group_cat_id].children.push(child_id);
                }
            }
        }

        // Ungrouped entities
        if !ungrouped.is_empty() {
            let other_cat_id = nodes.len();
            nodes.push(TreeNode {
                id: other_cat_id,
                label: "Other Entities".to_string(),
                depth: 0,
                entity: None,
                children: Vec::new(),
                is_category: true,
            });
            root_children.push(other_cat_id);

            for ews in sorted_entities(&ungrouped) {
                let ent_id = nodes.len();
                nodes.push(TreeNode {
                    id: ent_id,
                    label: format!("{}: {}", ews.entity.kind, ews.entity.display_name()),
                    depth: 1,
                    entity: Some(ews.clone()),
                    children: Vec::new(),
                    is_category: false,
                });
                nodes[other_cat_id].children.push(ent_id);
            }
        }

        EntityTree {
            nodes,
            root_children,
        }
    }

    /// Recursively build a group node and its descendants, returning the new node id.
    ///
    /// `visited` guards against cycles in malformed parent/child references.
    fn build_group_subtree(
        nodes: &mut Vec<TreeNode>,
        group_map: &HashMap<String, EntityWithSource>,
        children_of: &HashMap<String, Vec<String>>,
        name: &str,
        depth: usize,
        visited: &mut HashSet<String>,
    ) -> Option<usize> {
        if !visited.insert(name.to_string()) {
            return None;
        }
        let ews = group_map.get(name)?;

        let id = nodes.len();
        nodes.push(TreeNode {
            id,
            label: format!("{}: {}", EntityKind::Group, ews.entity.display_name()),
            depth,
            entity: Some(ews.clone()),
            children: Vec::new(),
            is_category: false,
        });

        if let Some(kids) = children_of.get(name) {
            for kid in kids {
                if let Some(child_id) = Self::build_group_subtree(
                    nodes,
                    group_map,
                    children_of,
                    kid,
                    depth + 1,
                    visited,
                ) {
                    nodes[id].children.push(child_id);
                }
            }
        }

        Some(id)
    }

    /// Get all visible nodes respecting the current expansion state.
    pub fn visible_nodes(&self, state: &TreeState) -> Vec<&TreeNode> {
        let mut visible = Vec::new();
        for &root_id in &self.root_children {
            self.collect_visible(&self.nodes[root_id], state, &mut visible);
        }
        visible
    }

    fn collect_visible<'a>(
        &'a self,
        node: &'a TreeNode,
        state: &TreeState,
        visible: &mut Vec<&'a TreeNode>,
    ) {
        visible.push(node);
        if state.is_expanded(node.id) {
            for &child_id in &node.children {
                self.collect_visible(&self.nodes[child_id], state, visible);
            }
        }
    }

    /// Get the visible nodes paired with a tree-connector prefix for each row.
    ///
    /// The prefix is built from box-drawing characters (`├─`, `└─`, `│`) so the
    /// hierarchy renders as a proper tree. Order matches [`visible_nodes`], so the
    /// rows align with selection/scroll indices. Top-level categories get no
    /// connector; their descendants are connected relative to them.
    pub fn visible_rows(&self, state: &TreeState) -> Vec<VisibleRow<'_>> {
        let mut rows = Vec::new();
        for &root_id in &self.root_children {
            self.collect_rows(&self.nodes[root_id], state, "", "", &mut rows);
        }
        rows
    }

    fn collect_rows<'a>(
        &'a self,
        node: &'a TreeNode,
        state: &TreeState,
        ancestor_prefix: &str,
        connector: &str,
        rows: &mut Vec<VisibleRow<'a>>,
    ) {
        rows.push(VisibleRow {
            node,
            prefix: format!("{ancestor_prefix}{connector}"),
        });

        if state.is_expanded(node.id) {
            // Extend the ancestor prefix: a vertical bar continues the line for
            // non-last branches, blank space closes it off for the last one.
            let child_ancestor = if connector.is_empty() {
                ancestor_prefix.to_string()
            } else if connector.starts_with('└') {
                format!("{ancestor_prefix}   ")
            } else {
                format!("{ancestor_prefix}│  ")
            };

            let last = node.children.len().saturating_sub(1);
            for (i, &child_id) in node.children.iter().enumerate() {
                let child_connector = if i == last { "└─ " } else { "├─ " };
                self.collect_rows(
                    &self.nodes[child_id],
                    state,
                    &child_ancestor,
                    child_connector,
                    rows,
                );
            }
        }
    }

    pub fn get_node(&self, id: usize) -> Option<&TreeNode> {
        self.nodes.get(id)
    }

    /// The id of the node whose children include `id`, if any.
    pub fn parent_of(&self, id: usize) -> Option<usize> {
        self.nodes
            .iter()
            .find(|n| n.children.contains(&id))
            .map(|n| n.id)
    }

    /// All node ids in display order, ignoring expansion state. Useful for
    /// ordered traversals (e.g. jumping between entities with errors).
    pub fn dfs_order(&self) -> Vec<usize> {
        let mut order = Vec::with_capacity(self.nodes.len());
        for &root_id in &self.root_children {
            self.collect_dfs(root_id, &mut order);
        }
        order
    }

    fn collect_dfs(&self, id: usize, order: &mut Vec<usize>) {
        order.push(id);
        if let Some(node) = self.nodes.get(id) {
            for &child_id in &node.children {
                self.collect_dfs(child_id, order);
            }
        }
    }

    /// Filter visible nodes by a search query.
    ///
    /// A bare query matches across the label, name, title, description, kind,
    /// owner, and tags. A `field:value` query (e.g. `owner:team-a`, `tag:web`,
    /// `kind:component`) restricts the match to that field of entity nodes.
    pub fn filter_by_search<'a>(nodes: Vec<&'a TreeNode>, search_query: &str) -> Vec<&'a TreeNode> {
        let query = search_query.trim().to_lowercase();
        if query.is_empty() {
            return nodes;
        }
        nodes
            .into_iter()
            .filter(|n| node_matches(n, &query))
            .collect()
    }
}

/// Keys of a string-keyed map, sorted, for deterministic iteration.
fn sorted_keys<V>(map: &HashMap<String, V>) -> Vec<&String> {
    let mut keys: Vec<&String> = map.keys().collect();
    keys.sort();
    keys
}

/// Entities sorted by kind then display name, for stable tree ordering.
fn sorted_entities<'a>(entities: &[&'a EntityWithSource]) -> Vec<&'a EntityWithSource> {
    let mut sorted: Vec<&'a EntityWithSource> = entities.to_vec();
    sorted.sort_by(|a, b| {
        a.entity
            .kind
            .to_string()
            .cmp(&b.entity.kind.to_string())
            .then_with(|| a.entity.display_name().cmp(&b.entity.display_name()))
    });
    sorted
}

/// Recognized `field:` scopes for search.
fn known_field(field: &str) -> bool {
    matches!(
        field,
        "name"
            | "title"
            | "kind"
            | "owner"
            | "system"
            | "domain"
            | "tag"
            | "tags"
            | "lifecycle"
            | "type"
            | "namespace"
            | "ns"
            | "desc"
            | "description"
    )
}

/// Resolve a single searchable field of an entity to a string, if present.
fn field_value(entity: &crate::entity::Entity, field: &str) -> Option<String> {
    match field {
        "name" => Some(entity.metadata.name.clone()),
        "title" => entity.metadata.title.clone(),
        "kind" => Some(entity.kind.to_string()),
        "owner" => entity.owner(),
        "system" => entity.system(),
        "domain" => entity.domain(),
        "tag" | "tags" => Some(entity.metadata.tags.join(" ")),
        "lifecycle" => entity.lifecycle(),
        "type" => entity.entity_type(),
        "namespace" | "ns" => entity.metadata.namespace.clone(),
        "desc" | "description" => entity.metadata.description.clone(),
        _ => None,
    }
}

/// Whether a node matches a (already-lowercased, non-empty) query.
fn node_matches(node: &TreeNode, query: &str) -> bool {
    // Field-scoped query: `field:value` restricted to entity nodes.
    if let Some((field, value)) = query.split_once(':') {
        if known_field(field) {
            let value = value.trim();
            return match &node.entity {
                Some(ews) => field_value(&ews.entity, field)
                    .is_some_and(|v| v.to_lowercase().contains(value)),
                None => false,
            };
        }
    }

    // Free-text query: match the label or any common entity field.
    if node.label.to_lowercase().contains(query) {
        return true;
    }
    let Some(ews) = &node.entity else {
        return false;
    };
    let e = &ews.entity;
    let haystacks = [
        Some(e.metadata.name.clone()),
        e.metadata.title.clone(),
        e.metadata.description.clone(),
        Some(e.kind.to_string()),
        e.owner(),
        Some(e.metadata.tags.join(" ")),
    ];
    haystacks
        .iter()
        .flatten()
        .any(|h| h.to_lowercase().contains(query))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::{Entity, EntityKind, Metadata};
    use std::path::PathBuf;

    fn create_test_entity(
        kind: EntityKind,
        name: &str,
        system: Option<&str>,
        domain: Option<&str>,
    ) -> EntityWithSource {
        let mut spec = serde_yaml::Value::Null;
        if let Some(sys) = system {
            let mut map = serde_yaml::Mapping::new();
            map.insert(
                serde_yaml::Value::String("system".to_string()),
                serde_yaml::Value::String(sys.to_string()),
            );
            if let Some(dom) = domain {
                map.insert(
                    serde_yaml::Value::String("domain".to_string()),
                    serde_yaml::Value::String(dom.to_string()),
                );
            }
            spec = serde_yaml::Value::Mapping(map);
        } else if let Some(dom) = domain {
            let mut map = serde_yaml::Mapping::new();
            map.insert(
                serde_yaml::Value::String("domain".to_string()),
                serde_yaml::Value::String(dom.to_string()),
            );
            spec = serde_yaml::Value::Mapping(map);
        }

        EntityWithSource {
            entity: Entity {
                api_version: "backstage.io/v1alpha1".to_string(),
                kind,
                metadata: Metadata {
                    name: name.to_string(),
                    title: Some(name.to_string()),
                    namespace: Some("default".to_string()),
                    description: None,
                    labels: HashMap::new(),
                    annotations: HashMap::new(),
                    tags: Vec::new(),
                    links: Vec::new(),
                },
                spec,
            },
            source_file: PathBuf::from("/test/catalog-info.yaml"),
            validation_errors: Vec::new(),
        }
    }

    fn create_test_group(name: &str, parent: Option<&str>) -> EntityWithSource {
        let mut spec = serde_yaml::Value::Null;
        if let Some(p) = parent {
            let mut map = serde_yaml::Mapping::new();
            map.insert(
                serde_yaml::Value::String("parent".to_string()),
                serde_yaml::Value::String(p.to_string()),
            );
            spec = serde_yaml::Value::Mapping(map);
        }

        let mut ews = create_test_entity(EntityKind::Group, name, None, None);
        ews.entity.spec = spec;
        ews
    }

    fn create_test_group_with_children(name: &str, children: &[&str]) -> EntityWithSource {
        let mut map = serde_yaml::Mapping::new();
        let seq: Vec<serde_yaml::Value> = children
            .iter()
            .map(|c| serde_yaml::Value::String(c.to_string()))
            .collect();
        map.insert(
            serde_yaml::Value::String("children".to_string()),
            serde_yaml::Value::Sequence(seq),
        );

        let mut ews = create_test_entity(EntityKind::Group, name, None, None);
        ews.entity.spec = serde_yaml::Value::Mapping(map);
        ews
    }

    #[test]
    fn test_tree_building_hierarchy() {
        // Create a complete hierarchy: Domain -> System -> Component
        let entities = vec![
            create_test_entity(EntityKind::Domain, "platform", None, None),
            create_test_entity(EntityKind::System, "auth-system", None, Some("platform")),
            create_test_entity(
                EntityKind::Component,
                "user-service",
                Some("auth-system"),
                None,
            ),
            create_test_entity(EntityKind::Api, "user-api", Some("auth-system"), None),
        ];

        let tree = EntityTree::build(&entities);

        // Verify tree structure
        assert_eq!(tree.root_children.len(), 1, "Should have 1 root category");

        // Find "Domains" category
        let domains_node = &tree.nodes[tree.root_children[0]];
        assert_eq!(domains_node.label, "Domains");
        assert_eq!(domains_node.depth, 0);
        assert!(domains_node.is_category);
        assert_eq!(domains_node.children.len(), 1, "Should have 1 domain");

        // Find platform domain
        let platform_node = &tree.nodes[domains_node.children[0]];
        assert!(platform_node.label.contains("platform"));
        assert_eq!(platform_node.depth, 1);
        assert!(!platform_node.is_category);
        assert_eq!(platform_node.children.len(), 1, "Should have 1 system");

        // Find auth-system
        let system_node = &tree.nodes[platform_node.children[0]];
        assert!(system_node.label.contains("auth-system"));
        assert_eq!(system_node.depth, 2);
        assert_eq!(
            system_node.children.len(),
            2,
            "Should have 2 children (component + api)"
        );

        // Both the component and the API are present (order is sorted by kind,
        // so the API precedes the Component — assert by lookup, not index).
        let children: Vec<&TreeNode> = system_node
            .children
            .iter()
            .map(|&id| &tree.nodes[id])
            .collect();
        let component_node = children
            .iter()
            .find(|n| n.label.contains("user-service"))
            .expect("user-service present");
        assert_eq!(component_node.depth, 3);
        assert_eq!(component_node.children.len(), 0);

        let api_node = children
            .iter()
            .find(|n| n.label.contains("user-api"))
            .expect("user-api present");
        assert_eq!(api_node.depth, 3);
    }

    #[test]
    fn test_tree_search_filtering() {
        let entities = vec![
            create_test_entity(EntityKind::Component, "user-service", None, None),
            create_test_entity(EntityKind::Component, "order-service", None, None),
            create_test_entity(EntityKind::Api, "user-api", None, None),
        ];

        let tree = EntityTree::build(&entities);
        let mut state = TreeState::new();

        // Expand the "Other Entities" category to see the actual entities
        state.expand_all(&tree);
        let visible = tree.visible_nodes(&state);

        // Filter by "user"
        let filtered = EntityTree::filter_by_search(visible.clone(), "user");
        assert_eq!(filtered.len(), 2, "Should match user-service and user-api");
        assert!(filtered.iter().any(|n| n.label.contains("user-service")));
        assert!(filtered.iter().any(|n| n.label.contains("user-api")));

        // Filter by "order"
        let filtered = EntityTree::filter_by_search(visible.clone(), "order");
        assert_eq!(filtered.len(), 1, "Should match only order-service");
        assert!(filtered[0].label.contains("order-service"));

        // Filter by "api"
        let filtered = EntityTree::filter_by_search(visible.clone(), "api");
        assert_eq!(filtered.len(), 1, "Should match only user-api");

        // Case insensitive search
        let filtered = EntityTree::filter_by_search(visible.clone(), "USER");
        assert_eq!(filtered.len(), 2, "Search should be case insensitive");

        // No matches
        let filtered = EntityTree::filter_by_search(visible.clone(), "nonexistent");
        assert_eq!(filtered.len(), 0, "Should have no matches");

        // Filter by category name
        let filtered = EntityTree::filter_by_search(visible, "Other");
        assert_eq!(filtered.len(), 1, "Should match 'Other Entities' category");
    }

    #[test]
    fn test_tree_ordering_is_sorted() {
        // Build with domains, systems and components deliberately out of order.
        let entities = vec![
            create_test_entity(EntityKind::Domain, "zeta", None, None),
            create_test_entity(EntityKind::Domain, "alpha", None, None),
            create_test_entity(EntityKind::System, "sys-b", None, Some("alpha")),
            create_test_entity(EntityKind::System, "sys-a", None, Some("alpha")),
            create_test_entity(EntityKind::Component, "comp-z", Some("sys-a"), None),
            create_test_entity(EntityKind::Api, "api-a", Some("sys-a"), None),
        ];

        let tree = EntityTree::build(&entities);

        // Domains category lists domains alphabetically.
        let domains_cat = tree
            .root_children
            .iter()
            .map(|&id| &tree.nodes[id])
            .find(|n| n.label == "Domains")
            .unwrap();
        let domain_labels: Vec<&str> = domains_cat
            .children
            .iter()
            .map(|&id| tree.nodes[id].label.as_str())
            .collect();
        assert!(domain_labels[0].contains("alpha"));
        assert!(domain_labels[1].contains("zeta"));

        // Systems under alpha are sorted.
        let alpha = &tree.nodes[domains_cat.children[0]];
        let sys_labels: Vec<&str> = alpha
            .children
            .iter()
            .map(|&id| tree.nodes[id].label.as_str())
            .collect();
        assert!(sys_labels[0].contains("sys-a"));
        assert!(sys_labels[1].contains("sys-b"));

        // Components under sys-a are sorted by kind then name: API before Component.
        let sys_a = &tree.nodes[alpha.children[0]];
        let comp_labels: Vec<&str> = sys_a
            .children
            .iter()
            .map(|&id| tree.nodes[id].label.as_str())
            .collect();
        assert!(comp_labels[0].contains("api-a"), "API sorts first");
        assert!(comp_labels[1].contains("comp-z"));
    }

    #[test]
    fn test_scoped_and_multifield_search() {
        use crate::entity::{Entity, Metadata};
        use std::path::PathBuf;

        let mut spec = serde_yaml::Mapping::new();
        spec.insert(
            serde_yaml::Value::String("owner".to_string()),
            serde_yaml::Value::String("platform-team".to_string()),
        );
        let billing = EntityWithSource::new(
            Entity {
                api_version: "backstage.io/v1alpha1".to_string(),
                kind: EntityKind::Component,
                metadata: Metadata {
                    name: "billing".to_string(),
                    title: Some("Billing Service".to_string()),
                    namespace: Some("default".to_string()),
                    description: Some("Handles invoices".to_string()),
                    labels: HashMap::new(),
                    annotations: HashMap::new(),
                    tags: vec!["payments".to_string(), "backend".to_string()],
                    links: Vec::new(),
                },
                spec: serde_yaml::Value::Mapping(spec),
            },
            PathBuf::from("c.yaml"),
        );
        let reports_api = create_test_entity(EntityKind::Api, "reports-api", None, None);

        let tree = EntityTree::build(&[billing, reports_api]);
        let mut state = TreeState::new();
        state.expand_all(&tree);
        let visible = tree.visible_nodes(&state);

        let count = |q: &str| EntityTree::filter_by_search(visible.clone(), q).len();

        // Field-scoped queries
        assert_eq!(count("owner:platform-team"), 1, "owner scope");
        assert_eq!(count("owner:other"), 0, "owner scope, no match");
        assert_eq!(count("tag:payments"), 1, "tag scope");
        assert_eq!(count("kind:api"), 1, "kind scope matches the API only");

        // Free-text across fields
        assert_eq!(count("invoices"), 1, "matches description");
        assert_eq!(count("Billing"), 1, "matches title (case-insensitive)");
        assert_eq!(count("backend"), 1, "matches a tag in free text");

        // Unknown field prefix is treated as free text (no crash, no match here)
        assert_eq!(
            count("color:red"),
            0,
            "unknown scope -> free text, no match"
        );
    }

    #[test]
    fn test_visible_nodes() {
        let entities = vec![
            create_test_entity(EntityKind::Domain, "platform", None, None),
            create_test_entity(EntityKind::System, "auth-system", None, Some("platform")),
            create_test_entity(
                EntityKind::Component,
                "user-service",
                Some("auth-system"),
                None,
            ),
        ];

        let tree = EntityTree::build(&entities);
        let mut state = TreeState::new();

        // Initially, only root category should be visible (not expanded)
        let visible = tree.visible_nodes(&state);
        assert_eq!(
            visible.len(),
            1,
            "Only Domains category should be visible initially"
        );
        assert_eq!(visible[0].label, "Domains");

        // Expand Domains category
        let domains_id = tree.root_children[0];
        state.toggle_expanded(domains_id);
        let visible = tree.visible_nodes(&state);
        assert_eq!(
            visible.len(),
            2,
            "Domains + platform domain should be visible"
        );

        // Expand platform domain
        let platform_id = tree.nodes[domains_id].children[0];
        state.toggle_expanded(platform_id);
        let visible = tree.visible_nodes(&state);
        assert_eq!(
            visible.len(),
            3,
            "Domains + platform + auth-system should be visible"
        );

        // Expand auth-system
        let system_id = tree.nodes[platform_id].children[0];
        state.toggle_expanded(system_id);
        let visible = tree.visible_nodes(&state);
        assert_eq!(
            visible.len(),
            4,
            "All nodes should be visible when fully expanded"
        );

        // Collapse Domains category - should hide all children
        state.toggle_expanded(domains_id);
        let visible = tree.visible_nodes(&state);
        assert_eq!(
            visible.len(),
            1,
            "Collapsing Domains should hide all descendants"
        );
    }

    #[test]
    fn test_tree_node_depth() {
        let entities = vec![
            create_test_entity(EntityKind::Domain, "platform", None, None),
            create_test_entity(EntityKind::System, "auth-system", None, Some("platform")),
            create_test_entity(
                EntityKind::Component,
                "user-service",
                Some("auth-system"),
                None,
            ),
            create_test_entity(EntityKind::Resource, "database", Some("auth-system"), None),
        ];

        let tree = EntityTree::build(&entities);

        // Category should be depth 0
        let domains_node = &tree.nodes[tree.root_children[0]];
        assert_eq!(domains_node.depth, 0, "Category should be depth 0");

        // Domain should be depth 1
        let platform_node = &tree.nodes[domains_node.children[0]];
        assert_eq!(platform_node.depth, 1, "Domain should be depth 1");

        // System should be depth 2
        let system_node = &tree.nodes[platform_node.children[0]];
        assert_eq!(system_node.depth, 2, "System should be depth 2");

        // Component and Resource should be depth 3
        for &child_id in &system_node.children {
            let child_node = &tree.nodes[child_id];
            assert_eq!(child_node.depth, 3, "Component/Resource should be depth 3");
        }
    }

    #[test]
    fn test_ungrouped_entities() {
        let entities = vec![
            // Entities with no system/domain
            create_test_entity(EntityKind::Component, "orphan-component", None, None),
            create_test_entity(EntityKind::User, "john-doe", None, None),
            create_test_entity(EntityKind::Group, "developers", None, None),
            create_test_entity(EntityKind::Location, "github-org", None, None),
        ];

        let tree = EntityTree::build(&entities);

        // Groups now get their own category, so we expect "Groups" + "Other Entities".
        let categories: Vec<String> = tree
            .root_children
            .iter()
            .map(|&id| tree.nodes[id].label.clone())
            .collect();
        assert!(categories.contains(&"Groups".to_string()));
        assert!(categories.contains(&"Other Entities".to_string()));

        let other_node = tree
            .root_children
            .iter()
            .map(|&id| &tree.nodes[id])
            .find(|n| n.label == "Other Entities")
            .expect("Other Entities category");
        assert_eq!(other_node.depth, 0);
        assert!(other_node.is_category);
        assert_eq!(
            other_node.children.len(),
            3,
            "Non-group ungrouped entities (component, user, location)"
        );

        // All children should be depth 1
        for &child_id in &other_node.children {
            let child_node = &tree.nodes[child_id];
            assert_eq!(child_node.depth, 1, "Ungrouped entities should be depth 1");
            assert!(!child_node.is_category);
            assert_eq!(child_node.children.len(), 0, "Should have no children");
        }

        // Non-group entities live under "Other Entities".
        let other_labels: Vec<String> = other_node
            .children
            .iter()
            .map(|&id| tree.nodes[id].label.clone())
            .collect();
        assert!(other_labels.iter().any(|l| l.contains("orphan-component")));
        assert!(other_labels.iter().any(|l| l.contains("john-doe")));
        assert!(other_labels.iter().any(|l| l.contains("github-org")));

        // The group lives under "Groups".
        let groups_node = tree
            .root_children
            .iter()
            .map(|&id| &tree.nodes[id])
            .find(|n| n.label == "Groups")
            .expect("Groups category");
        assert_eq!(groups_node.children.len(), 1, "Should contain the group");
        assert!(tree.nodes[groups_node.children[0]]
            .label
            .contains("developers"));
    }

    #[test]
    fn test_child_groups_nested() {
        // parent-team has two child groups via spec.parent.
        let entities = vec![
            create_test_group("parent-team", None),
            create_test_group("team-a", Some("parent-team")),
            create_test_group("team-b", Some("parent-team")),
        ];

        let tree = EntityTree::build(&entities);

        // Single root category: "Groups"
        assert_eq!(tree.root_children.len(), 1);
        let groups_node = &tree.nodes[tree.root_children[0]];
        assert_eq!(groups_node.label, "Groups");
        assert_eq!(
            groups_node.children.len(),
            1,
            "Only the root group should sit directly under Groups"
        );

        // parent-team at depth 1 with the two children nested under it.
        let parent_node = &tree.nodes[groups_node.children[0]];
        assert!(parent_node.label.contains("parent-team"));
        assert_eq!(parent_node.depth, 1);
        assert_eq!(parent_node.children.len(), 2, "Two child groups");

        let child_labels: Vec<String> = parent_node
            .children
            .iter()
            .map(|&id| {
                assert_eq!(tree.nodes[id].depth, 2, "Child groups should be depth 2");
                tree.nodes[id].label.clone()
            })
            .collect();
        assert!(child_labels.iter().any(|l| l.contains("team-a")));
        assert!(child_labels.iter().any(|l| l.contains("team-b")));
    }

    #[test]
    fn test_child_groups_via_spec_children() {
        // Parent declares children via spec.children instead of child's spec.parent.
        let entities = vec![
            create_test_group_with_children("parent-team", &["team-a"]),
            create_test_group("team-a", None),
        ];

        let tree = EntityTree::build(&entities);

        let groups_node = &tree.nodes[tree.root_children[0]];
        assert_eq!(groups_node.label, "Groups");
        assert_eq!(
            groups_node.children.len(),
            1,
            "Only root group at top level"
        );
        let parent_node = &tree.nodes[groups_node.children[0]];
        assert!(parent_node.label.contains("parent-team"));
        assert_eq!(parent_node.children.len(), 1);
        assert!(tree.nodes[parent_node.children[0]].label.contains("team-a"));
    }

    #[test]
    fn test_visible_rows_have_connectors() {
        let entities = vec![
            create_test_group("parent-team", None),
            create_test_group("team-a", Some("parent-team")),
            create_test_group("team-b", Some("parent-team")),
        ];

        let tree = EntityTree::build(&entities);
        let mut state = TreeState::new();
        state.expand_all(&tree);

        let rows = tree.visible_rows(&state);
        // Groups (root, no connector), parent-team, team-a, team-b
        assert_eq!(rows.len(), 4);
        assert_eq!(rows[0].prefix, "", "Root category has no connector");
        // team-a / team-b are the children of parent-team; last one uses └─.
        let team_b = rows
            .iter()
            .find(|r| r.node.label.contains("team-b"))
            .unwrap();
        assert!(team_b.prefix.contains('└'), "Last child uses └ connector");
        let team_a = rows
            .iter()
            .find(|r| r.node.label.contains("team-a"))
            .unwrap();
        assert!(
            team_a.prefix.contains('├'),
            "Non-last child uses ├ connector"
        );
    }

    #[test]
    fn test_orphan_systems() {
        // System without domain should go into "Systems" category
        let entities = vec![
            create_test_entity(EntityKind::System, "orphan-system", None, None),
            create_test_entity(
                EntityKind::Component,
                "orphan-component",
                Some("orphan-system"),
                None,
            ),
        ];

        let tree = EntityTree::build(&entities);

        // Should have one root category: "Systems"
        assert_eq!(tree.root_children.len(), 1);
        let systems_node = &tree.nodes[tree.root_children[0]];
        assert_eq!(systems_node.label, "Systems");
        assert_eq!(systems_node.children.len(), 1);

        // System should have the component as child
        let system_node = &tree.nodes[systems_node.children[0]];
        assert!(system_node.label.contains("orphan-system"));
        assert_eq!(system_node.depth, 1);
        assert_eq!(system_node.children.len(), 1, "System should have 1 child");

        let component_node = &tree.nodes[system_node.children[0]];
        assert!(component_node.label.contains("orphan-component"));
        assert_eq!(component_node.depth, 2);
    }

    #[test]
    fn test_tree_state_toggle_expanded() {
        let mut state = TreeState::new();

        assert!(!state.is_expanded(1), "Node 1 should not be expanded");

        state.toggle_expanded(1);
        assert!(state.is_expanded(1), "Node 1 should be expanded");

        state.toggle_expanded(1);
        assert!(!state.is_expanded(1), "Node 1 should be collapsed");
    }

    #[test]
    fn test_tree_state_expand_all() {
        let entities = vec![
            create_test_entity(EntityKind::Domain, "platform", None, None),
            create_test_entity(EntityKind::System, "auth-system", None, Some("platform")),
            create_test_entity(
                EntityKind::Component,
                "user-service",
                Some("auth-system"),
                None,
            ),
        ];

        let tree = EntityTree::build(&entities);
        let mut state = TreeState::new();

        state.expand_all(&tree);

        // All nodes with children should be expanded
        for node in &tree.nodes {
            if !node.children.is_empty() {
                assert!(
                    state.is_expanded(node.id),
                    "Node {} should be expanded",
                    node.label
                );
            }
        }
    }

    #[test]
    fn test_mixed_hierarchy() {
        // Mix of grouped and ungrouped entities
        let entities = vec![
            // Grouped hierarchy
            create_test_entity(EntityKind::Domain, "platform", None, None),
            create_test_entity(EntityKind::System, "auth-system", None, Some("platform")),
            create_test_entity(
                EntityKind::Component,
                "user-service",
                Some("auth-system"),
                None,
            ),
            // Orphan system
            create_test_entity(EntityKind::System, "standalone-system", None, None),
            // Completely ungrouped
            create_test_entity(EntityKind::User, "alice", None, None),
        ];

        let tree = EntityTree::build(&entities);

        // Should have 3 root categories: Domains, Systems, Other Entities
        assert_eq!(tree.root_children.len(), 3, "Should have 3 root categories");

        let categories: Vec<String> = tree
            .root_children
            .iter()
            .map(|&id| tree.nodes[id].label.clone())
            .collect();
        assert!(categories.contains(&"Domains".to_string()));
        assert!(categories.contains(&"Systems".to_string()));
        assert!(categories.contains(&"Other Entities".to_string()));
    }

    #[test]
    fn test_get_node() {
        let entities = vec![create_test_entity(
            EntityKind::Component,
            "test-component",
            None,
            None,
        )];

        let tree = EntityTree::build(&entities);

        // Valid node ID
        let node = tree.get_node(0);
        assert!(node.is_some(), "Should find node 0");

        // Invalid node ID
        let node = tree.get_node(9999);
        assert!(node.is_none(), "Should not find node 9999");
    }

    #[test]
    fn test_empty_tree() {
        let entities: Vec<EntityWithSource> = vec![];
        let tree = EntityTree::build(&entities);

        assert_eq!(tree.nodes.len(), 0, "Tree should have no nodes");
        assert_eq!(tree.root_children.len(), 0, "Tree should have no root");

        let state = TreeState::new();
        let visible = tree.visible_nodes(&state);
        assert_eq!(visible.len(), 0, "No visible nodes in empty tree");
    }

    #[test]
    fn test_system_with_nonexistent_domain() {
        // System references domain that doesn't exist
        let entities = vec![
            create_test_entity(EntityKind::System, "system1", None, Some("nonexistent")),
            create_test_entity(EntityKind::Component, "comp1", Some("system1"), None),
        ];

        let tree = EntityTree::build(&entities);

        // System should be in "Systems" category (not under "Domains")
        assert_eq!(tree.root_children.len(), 1);
        let systems_node = &tree.nodes[tree.root_children[0]];
        assert_eq!(
            systems_node.label, "Systems",
            "Should create Systems category for orphan system"
        );
    }
}
