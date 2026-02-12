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
//! let tree = EntityTree::build(entities);
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
//! let tree = EntityTree::build(entities);
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
//! let tree = EntityTree::build(entities);
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

use crate::entity::{EntityKind, EntityWithSource};
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
    pub fn build(entities: Vec<EntityWithSource>) -> Self {
        let mut nodes: Vec<TreeNode> = Vec::new();
        let mut root_children: Vec<usize> = Vec::new();

        // Group entities by kind, then by domain/system relationships
        let mut domains: HashMap<String, Vec<&EntityWithSource>> = HashMap::new();
        let mut systems: HashMap<String, Vec<&EntityWithSource>> = HashMap::new();
        let mut system_to_domain: HashMap<String, String> = HashMap::new();
        let mut components_by_system: HashMap<String, Vec<&EntityWithSource>> = HashMap::new();
        let mut ungrouped: Vec<&EntityWithSource> = Vec::new();

        // First pass: collect domains and systems
        for ews in &entities {
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
        for ews in &entities {
            match ews.entity.kind {
                EntityKind::Domain | EntityKind::System => {}
                EntityKind::Component | EntityKind::Api | EntityKind::Resource => {
                    if let Some(system) = ews.entity.system() {
                        components_by_system.entry(system).or_default().push(ews);
                    } else {
                        ungrouped.push(ews);
                    }
                }
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

            for (domain_name, domain_entities) in &domains {
                for ews in domain_entities {
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

                    // Add systems belonging to this domain
                    for (sys_name, sys_entities) in &systems {
                        if system_to_domain.get(sys_name) == Some(domain_name) {
                            for sys_ews in sys_entities {
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

                                // Add components of this system
                                if let Some(comps) = components_by_system.get(sys_name) {
                                    for comp_ews in comps {
                                        let comp_id = nodes.len();
                                        nodes.push(TreeNode {
                                            id: comp_id,
                                            label: format!(
                                                "{}: {}",
                                                comp_ews.entity.kind,
                                                comp_ews.entity.display_name()
                                            ),
                                            depth: 3,
                                            entity: Some((*comp_ews).clone()),
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
        }

        // Systems without domains (or with non-existent domain references)
        let orphan_systems: Vec<_> = systems
            .iter()
            .filter(|(name, _)| {
                match system_to_domain.get(*name) {
                    None => true,                                            // No domain reference
                    Some(domain_name) => !domains.contains_key(domain_name), // Domain doesn't exist
                }
            })
            .collect();

        if !orphan_systems.is_empty() {
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

            for (sys_name, sys_entities) in orphan_systems {
                for ews in sys_entities {
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

                    // Add components of this system
                    if let Some(comps) = components_by_system.get(sys_name) {
                        for comp_ews in comps {
                            let comp_id = nodes.len();
                            nodes.push(TreeNode {
                                id: comp_id,
                                label: format!(
                                    "{}: {}",
                                    comp_ews.entity.kind,
                                    comp_ews.entity.display_name()
                                ),
                                depth: 2,
                                entity: Some((*comp_ews).clone()),
                                children: Vec::new(),
                                is_category: false,
                            });
                            nodes[sys_id].children.push(comp_id);
                        }
                    }
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

            for ews in ungrouped {
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

    pub fn get_node(&self, id: usize) -> Option<&TreeNode> {
        self.nodes.get(id)
    }

    /// Filter visible nodes by search query
    pub fn filter_by_search<'a>(nodes: Vec<&'a TreeNode>, search_query: &str) -> Vec<&'a TreeNode> {
        let query = search_query.to_lowercase();
        nodes
            .into_iter()
            .filter(|n| n.label.to_lowercase().contains(&query))
            .collect()
    }
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

        let tree = EntityTree::build(entities);

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

        // Find user-service component
        let component_node = &tree.nodes[system_node.children[0]];
        assert!(component_node.label.contains("user-service"));
        assert_eq!(component_node.depth, 3);
        assert_eq!(component_node.children.len(), 0);

        // Find user-api
        let api_node = &tree.nodes[system_node.children[1]];
        assert!(api_node.label.contains("user-api"));
        assert_eq!(api_node.depth, 3);
    }

    #[test]
    fn test_tree_search_filtering() {
        let entities = vec![
            create_test_entity(EntityKind::Component, "user-service", None, None),
            create_test_entity(EntityKind::Component, "order-service", None, None),
            create_test_entity(EntityKind::Api, "user-api", None, None),
        ];

        let tree = EntityTree::build(entities);
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

        let tree = EntityTree::build(entities);
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

        let tree = EntityTree::build(entities);

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

        let tree = EntityTree::build(entities);

        // Should have exactly one root category: "Other Entities"
        assert_eq!(tree.root_children.len(), 1, "Should have 1 root category");

        let other_node = &tree.nodes[tree.root_children[0]];
        assert_eq!(
            other_node.label, "Other Entities",
            "Should be 'Other Entities' category"
        );
        assert_eq!(other_node.depth, 0);
        assert!(other_node.is_category);
        assert_eq!(
            other_node.children.len(),
            4,
            "Should contain all 4 entities"
        );

        // All children should be depth 1
        for &child_id in &other_node.children {
            let child_node = &tree.nodes[child_id];
            assert_eq!(child_node.depth, 1, "Ungrouped entities should be depth 1");
            assert!(!child_node.is_category);
            assert_eq!(child_node.children.len(), 0, "Should have no children");
        }

        // Verify all entity types are present
        let labels: Vec<String> = other_node
            .children
            .iter()
            .map(|&id| tree.nodes[id].label.clone())
            .collect();
        assert!(labels.iter().any(|l| l.contains("orphan-component")));
        assert!(labels.iter().any(|l| l.contains("john-doe")));
        assert!(labels.iter().any(|l| l.contains("developers")));
        assert!(labels.iter().any(|l| l.contains("github-org")));
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

        let tree = EntityTree::build(entities);

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

        let tree = EntityTree::build(entities);
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

        let tree = EntityTree::build(entities);

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

        let tree = EntityTree::build(entities);

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
        let tree = EntityTree::build(entities);

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

        let tree = EntityTree::build(entities);

        // System should be in "Systems" category (not under "Domains")
        assert_eq!(tree.root_children.len(), 1);
        let systems_node = &tree.nodes[tree.root_children[0]];
        assert_eq!(
            systems_node.label, "Systems",
            "Should create Systems category for orphan system"
        );
    }
}
