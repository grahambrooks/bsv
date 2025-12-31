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

            for (domain_name, domain_entities) in domains.iter() {
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
                    for (sys_name, sys_entities) in systems.iter() {
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
}
