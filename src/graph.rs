use crate::entity::{EntityRef, EntityWithSource};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationType {
    Owner,
    System,
    Domain,
    Parent,
    Child,
    DependsOn,
    DependencyOf,
    ProvidesApi,
    ConsumesApi,
    ProvidedBy,
    ConsumedBy,
    MemberOf,
    HasMember,
}

impl RelationType {
    pub fn label(&self) -> &'static str {
        match self {
            RelationType::Owner => "owned by",
            RelationType::System => "part of",
            RelationType::Domain => "in domain",
            RelationType::Parent => "parent",
            RelationType::Child => "child",
            RelationType::DependsOn => "depends on",
            RelationType::DependencyOf => "dependency of",
            RelationType::ProvidesApi => "provides",
            RelationType::ConsumesApi => "consumes",
            RelationType::ProvidedBy => "provided by",
            RelationType::ConsumedBy => "consumed by",
            RelationType::MemberOf => "member of",
            RelationType::HasMember => "has member",
        }
    }
}

#[derive(Debug, Clone)]
pub struct EntityNode {
    pub display_name: String,
    pub kind: String,
    pub exists: bool,
}

#[derive(Debug, Clone)]
pub struct RelationshipGraph {
    pub center: EntityNode,
    pub outgoing: Vec<(RelationType, EntityNode)>,
    pub incoming: Vec<(RelationType, EntityNode)>,
}

impl RelationshipGraph {
    pub fn build(entity: &EntityWithSource, all_entities: &[EntityWithSource]) -> Self {
        let center_ref = entity.entity.ref_key();
        let entity_map: HashMap<String, &EntityWithSource> = all_entities
            .iter()
            .map(|e| (e.entity.ref_key(), e))
            .collect();

        let center = EntityNode {
            display_name: entity.entity.display_name(),
            kind: entity.entity.kind.to_string(),
            exists: true,
        };

        let mut outgoing = Vec::new();
        let mut incoming = Vec::new();

        // Extract outgoing relationships from this entity
        Self::extract_outgoing_relationships(entity, &entity_map, &mut outgoing);

        // Find incoming relationships (other entities pointing to this one)
        Self::extract_incoming_relationships(&center_ref, all_entities, &entity_map, &mut incoming);

        RelationshipGraph {
            center,
            outgoing,
            incoming,
        }
    }

    fn extract_outgoing_relationships(
        entity: &EntityWithSource,
        entity_map: &HashMap<String, &EntityWithSource>,
        outgoing: &mut Vec<(RelationType, EntityNode)>,
    ) {
        // Owner relationship
        if let Some(owner_ref) = entity.entity.owner() {
            let parsed = EntityRef::parse(&owner_ref, "group");
            let exists = entity_map.contains_key(&parsed.canonical());
            outgoing.push((
                RelationType::Owner,
                EntityNode {
                    display_name: parsed.name.clone(),
                    kind: parsed.kind.clone(),
                    exists,
                },
            ));
        }

        // System relationship
        if let Some(system_ref) = entity.entity.system() {
            let parsed = EntityRef::parse(&system_ref, "system");
            let exists = entity_map.contains_key(&parsed.canonical());
            outgoing.push((
                RelationType::System,
                EntityNode {
                    display_name: parsed.name.clone(),
                    kind: parsed.kind.clone(),
                    exists,
                },
            ));
        }

        // Domain relationship
        if let Some(domain_ref) = entity.entity.domain() {
            let parsed = EntityRef::parse(&domain_ref, "domain");
            let exists = entity_map.contains_key(&parsed.canonical());
            outgoing.push((
                RelationType::Domain,
                EntityNode {
                    display_name: parsed.name.clone(),
                    kind: parsed.kind.clone(),
                    exists,
                },
            ));
        }

        // Parent relationship (for groups)
        if let Some(parent) = entity.entity.get_spec_string("parent") {
            let parsed = EntityRef::parse(&parent, "group");
            let exists = entity_map.contains_key(&parsed.canonical());
            outgoing.push((
                RelationType::Parent,
                EntityNode {
                    display_name: parsed.name.clone(),
                    kind: parsed.kind.clone(),
                    exists,
                },
            ));
        }

        // Children relationships (for groups)
        if let Some(children) = entity.entity.spec.get("children") {
            if let Some(children_arr) = children.as_sequence() {
                for child in children_arr {
                    if let Some(child_str) = child.as_str() {
                        let parsed = EntityRef::parse(child_str, "group");
                        let exists = entity_map.contains_key(&parsed.canonical());
                        outgoing.push((
                            RelationType::Child,
                            EntityNode {
                                display_name: parsed.name.clone(),
                                kind: parsed.kind.clone(),
                                exists,
                            },
                        ));
                    }
                }
            }
        }

        // DependsOn relationships
        if let Some(deps) = entity.entity.spec.get("dependsOn") {
            if let Some(deps_arr) = deps.as_sequence() {
                for dep in deps_arr {
                    if let Some(dep_str) = dep.as_str() {
                        let parsed = EntityRef::parse(dep_str, "component");
                        let exists = entity_map.contains_key(&parsed.canonical());
                        outgoing.push((
                            RelationType::DependsOn,
                            EntityNode {
                                display_name: parsed.name.clone(),
                                kind: parsed.kind.clone(),
                                exists,
                            },
                        ));
                    }
                }
            }
        }

        // ProvidesApis relationships
        if let Some(apis) = entity.entity.spec.get("providesApis") {
            if let Some(apis_arr) = apis.as_sequence() {
                for api in apis_arr {
                    if let Some(api_str) = api.as_str() {
                        let parsed = EntityRef::parse(api_str, "api");
                        let exists = entity_map.contains_key(&parsed.canonical());
                        outgoing.push((
                            RelationType::ProvidesApi,
                            EntityNode {
                                display_name: parsed.name.clone(),
                                kind: parsed.kind.clone(),
                                exists,
                            },
                        ));
                    }
                }
            }
        }

        // ConsumesApis relationships
        if let Some(apis) = entity.entity.spec.get("consumesApis") {
            if let Some(apis_arr) = apis.as_sequence() {
                for api in apis_arr {
                    if let Some(api_str) = api.as_str() {
                        let parsed = EntityRef::parse(api_str, "api");
                        let exists = entity_map.contains_key(&parsed.canonical());
                        outgoing.push((
                            RelationType::ConsumesApi,
                            EntityNode {
                                display_name: parsed.name.clone(),
                                kind: parsed.kind.clone(),
                                exists,
                            },
                        ));
                    }
                }
            }
        }

        // MemberOf relationships (for users)
        if let Some(groups) = entity.entity.spec.get("memberOf") {
            if let Some(groups_arr) = groups.as_sequence() {
                for group in groups_arr {
                    if let Some(group_str) = group.as_str() {
                        let parsed = EntityRef::parse(group_str, "group");
                        let exists = entity_map.contains_key(&parsed.canonical());
                        outgoing.push((
                            RelationType::MemberOf,
                            EntityNode {
                                display_name: parsed.name.clone(),
                                kind: parsed.kind.clone(),
                                exists,
                            },
                        ));
                    }
                }
            }
        }
    }

    fn extract_incoming_relationships(
        center_ref: &str,
        all_entities: &[EntityWithSource],
        entity_map: &HashMap<String, &EntityWithSource>,
        incoming: &mut Vec<(RelationType, EntityNode)>,
    ) {
        for other in all_entities {
            let other_ref = other.entity.ref_key();
            if other_ref == center_ref {
                continue;
            }

            // Check if this entity owns the center
            if let Some(owner) = other.entity.owner() {
                let parsed = EntityRef::parse(&owner, "group");
                if parsed.canonical() == center_ref {
                    incoming.push((
                        RelationType::Owner,
                        Self::node_from_entity(other, entity_map),
                    ));
                }
            }

            // Check if this entity is part of center system
            if let Some(system) = other.entity.system() {
                let parsed = EntityRef::parse(&system, "system");
                if parsed.canonical() == center_ref {
                    incoming.push((
                        RelationType::System,
                        Self::node_from_entity(other, entity_map),
                    ));
                }
            }

            // Check if this entity is in center domain
            if let Some(domain) = other.entity.domain() {
                let parsed = EntityRef::parse(&domain, "domain");
                if parsed.canonical() == center_ref {
                    incoming.push((
                        RelationType::Domain,
                        Self::node_from_entity(other, entity_map),
                    ));
                }
            }

            // Check if this entity has center as parent
            if let Some(parent) = other.entity.get_spec_string("parent") {
                let parsed = EntityRef::parse(&parent, "group");
                if parsed.canonical() == center_ref {
                    incoming.push((
                        RelationType::Child,
                        Self::node_from_entity(other, entity_map),
                    ));
                }
            }

            // Check if this entity depends on center
            if let Some(deps) = other.entity.spec.get("dependsOn") {
                if let Some(deps_arr) = deps.as_sequence() {
                    for dep in deps_arr {
                        if let Some(dep_str) = dep.as_str() {
                            let parsed = EntityRef::parse(dep_str, "component");
                            if parsed.canonical() == center_ref {
                                incoming.push((
                                    RelationType::DependencyOf,
                                    Self::node_from_entity(other, entity_map),
                                ));
                                break;
                            }
                        }
                    }
                }
            }

            // Check if this entity consumes API provided by center
            if let Some(apis) = other.entity.spec.get("consumesApis") {
                if let Some(apis_arr) = apis.as_sequence() {
                    for api in apis_arr {
                        if let Some(api_str) = api.as_str() {
                            let parsed = EntityRef::parse(api_str, "api");
                            if parsed.canonical() == center_ref {
                                incoming.push((
                                    RelationType::ConsumedBy,
                                    Self::node_from_entity(other, entity_map),
                                ));
                                break;
                            }
                        }
                    }
                }
            }

            // Check if this entity provides the center API
            if let Some(apis) = other.entity.spec.get("providesApis") {
                if let Some(apis_arr) = apis.as_sequence() {
                    for api in apis_arr {
                        if let Some(api_str) = api.as_str() {
                            let parsed = EntityRef::parse(api_str, "api");
                            if parsed.canonical() == center_ref {
                                incoming.push((
                                    RelationType::ProvidedBy,
                                    Self::node_from_entity(other, entity_map),
                                ));
                                break;
                            }
                        }
                    }
                }
            }

            // Check if user is member of center group
            if let Some(groups) = other.entity.spec.get("memberOf") {
                if let Some(groups_arr) = groups.as_sequence() {
                    for group in groups_arr {
                        if let Some(group_str) = group.as_str() {
                            let parsed = EntityRef::parse(group_str, "group");
                            if parsed.canonical() == center_ref {
                                incoming.push((
                                    RelationType::HasMember,
                                    Self::node_from_entity(other, entity_map),
                                ));
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    fn node_from_entity(
        entity: &EntityWithSource,
        _entity_map: &HashMap<String, &EntityWithSource>,
    ) -> EntityNode {
        EntityNode {
            display_name: entity.entity.display_name(),
            kind: entity.entity.kind.to_string(),
            exists: true,
        }
    }
}
