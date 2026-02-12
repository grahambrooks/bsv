//! Relationship graph extraction for Backstage entities.
//!
//! This module builds relationship graphs showing how entities relate to each other through
//! ownership, system membership, domain membership, dependencies, API provision/consumption,
//! and group membership. The graphs distinguish between outgoing relationships (references
//! this entity makes) and incoming relationships (references other entities make to this one).
//!
//! # Examples
//!
//! ## Building a Relationship Graph
//!
//! ```
//! use bsv::entity::{Entity, EntityKind, EntityWithSource, Metadata};
//! use bsv::graph::RelationshipGraph;
//! use std::path::PathBuf;
//! use std::collections::HashMap;
//!
//! # let mut spec = serde_yaml::Mapping::new();
//! # spec.insert(
//! #     serde_yaml::Value::String("owner".to_string()),
//! #     serde_yaml::Value::String("team-a".to_string()),
//! # );
//! # spec.insert(
//! #     serde_yaml::Value::String("system".to_string()),
//! #     serde_yaml::Value::String("platform".to_string()),
//! # );
//! # let component = Entity {
//! #     api_version: "backstage.io/v1alpha1".to_string(),
//! #     kind: EntityKind::Component,
//! #     metadata: Metadata {
//! #         name: "my-service".to_string(),
//! #         title: None,
//! #         namespace: Some("default".to_string()),
//! #         description: None,
//! #         labels: HashMap::new(),
//! #         annotations: HashMap::new(),
//! #         tags: Vec::new(),
//! #         links: Vec::new(),
//! #     },
//! #     spec: serde_yaml::Value::Mapping(spec),
//! # };
//! let entity = EntityWithSource::new(component, PathBuf::from("catalog.yaml"));
//! let all_entities = vec![entity.clone()];
//!
//! let graph = RelationshipGraph::build(&entity, &all_entities);
//!
//! println!("Center: {}", graph.center.display_name);
//! for (rel_type, node) in &graph.outgoing {
//!     println!("  {} {}", rel_type.label(), node.display_name);
//!     if !node.exists {
//!         println!("    (entity not found)");
//!     }
//! }
//! ```
//!
//! ## Checking Bidirectional Relationships
//!
//! ```
//! # use bsv::entity::{Entity, EntityKind, EntityWithSource, Metadata};
//! # use bsv::graph::{RelationshipGraph, RelationType};
//! # use std::path::PathBuf;
//! # use std::collections::HashMap;
//! # fn create_entity(kind: EntityKind, name: &str, spec: serde_yaml::Value) -> EntityWithSource {
//! #     EntityWithSource::new(
//! #         Entity {
//! #             api_version: "backstage.io/v1alpha1".to_string(),
//! #             kind,
//! #             metadata: Metadata {
//! #                 name: name.to_string(),
//! #                 title: None,
//! #                 namespace: Some("default".to_string()),
//! #                 description: None,
//! #                 labels: HashMap::new(),
//! #                 annotations: HashMap::new(),
//! #                 tags: Vec::new(),
//! #                 links: Vec::new(),
//! #             },
//! #             spec,
//! #         },
//! #         PathBuf::from("catalog.yaml"),
//! #     )
//! # }
//! // Component depends on another component
//! let frontend_spec = serde_yaml::from_str("dependsOn: [backend]").unwrap();
//! let frontend = create_entity(EntityKind::Component, "frontend", frontend_spec);
//! let backend = create_entity(EntityKind::Component, "backend", serde_yaml::Value::Null);
//!
//! let all = vec![frontend.clone(), backend.clone()];
//!
//! // Frontend has outgoing DependsOn
//! let frontend_graph = RelationshipGraph::build(&frontend, &all);
//! let has_depends_on = frontend_graph.outgoing.iter()
//!     .any(|(t, _)| *t == RelationType::DependsOn);
//! assert!(has_depends_on);
//!
//! // Backend has incoming DependencyOf
//! let backend_graph = RelationshipGraph::build(&backend, &all);
//! let has_dependency_of = backend_graph.incoming.iter()
//!     .any(|(t, _)| *t == RelationType::DependencyOf);
//! assert!(has_dependency_of);
//! ```
//!
//! # Key Types
//!
//! - [`RelationshipGraph`] - Complete relationship graph for an entity
//! - [`RelationType`] - Type of relationship (Owner, System, DependsOn, etc.)
//! - [`EntityNode`] - Node in the graph representing an entity reference

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
    /// Build a complete relationship graph for an entity.
    ///
    /// Extracts both outgoing relationships (references this entity makes)
    /// and incoming relationships (references other entities make to this one).
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

    fn add_single_ref_relationship(
        ref_str: &str,
        default_kind: &str,
        rel_type: RelationType,
        entity_map: &HashMap<String, &EntityWithSource>,
        outgoing: &mut Vec<(RelationType, EntityNode)>,
    ) {
        let parsed = EntityRef::parse(ref_str, default_kind);
        let exists = entity_map.contains_key(&parsed.canonical());
        outgoing.push((
            rel_type,
            EntityNode {
                display_name: parsed.name.clone(),
                kind: parsed.kind.clone(),
                exists,
            },
        ));
    }

    fn add_array_ref_relationships(
        field_value: &serde_yaml::Value,
        default_kind: &str,
        rel_type: RelationType,
        entity_map: &HashMap<String, &EntityWithSource>,
        outgoing: &mut Vec<(RelationType, EntityNode)>,
    ) {
        if let Some(arr) = field_value.as_sequence() {
            for item in arr {
                if let Some(item_str) = item.as_str() {
                    Self::add_single_ref_relationship(item_str, default_kind, rel_type.clone(), entity_map, outgoing);
                }
            }
        }
    }

    fn extract_outgoing_relationships(
        entity: &EntityWithSource,
        entity_map: &HashMap<String, &EntityWithSource>,
        outgoing: &mut Vec<(RelationType, EntityNode)>,
    ) {
        if let Some(owner_ref) = entity.entity.owner() {
            Self::add_single_ref_relationship(&owner_ref, "group", RelationType::Owner, entity_map, outgoing);
        }

        if let Some(system_ref) = entity.entity.system() {
            Self::add_single_ref_relationship(&system_ref, "system", RelationType::System, entity_map, outgoing);
        }

        if let Some(domain_ref) = entity.entity.domain() {
            Self::add_single_ref_relationship(&domain_ref, "domain", RelationType::Domain, entity_map, outgoing);
        }

        if let Some(parent) = entity.entity.get_spec_string("parent") {
            Self::add_single_ref_relationship(&parent, "group", RelationType::Parent, entity_map, outgoing);
        }

        if let Some(children) = entity.entity.spec.get("children") {
            Self::add_array_ref_relationships(children, "group", RelationType::Child, entity_map, outgoing);
        }

        if let Some(deps) = entity.entity.spec.get("dependsOn") {
            Self::add_array_ref_relationships(deps, "component", RelationType::DependsOn, entity_map, outgoing);
        }

        if let Some(apis) = entity.entity.spec.get("providesApis") {
            Self::add_array_ref_relationships(apis, "api", RelationType::ProvidesApi, entity_map, outgoing);
        }

        if let Some(apis) = entity.entity.spec.get("consumesApis") {
            Self::add_array_ref_relationships(apis, "api", RelationType::ConsumesApi, entity_map, outgoing);
        }

        if let Some(groups) = entity.entity.spec.get("memberOf") {
            Self::add_array_ref_relationships(groups, "group", RelationType::MemberOf, entity_map, outgoing);
        }
    }

    fn check_single_ref_incoming(
        ref_str: &str,
        default_kind: &str,
        center_ref: &str,
        rel_type: RelationType,
        entity: &EntityWithSource,
        entity_map: &HashMap<String, &EntityWithSource>,
        incoming: &mut Vec<(RelationType, EntityNode)>,
    ) {
        let parsed = EntityRef::parse(ref_str, default_kind);
        if parsed.canonical() == center_ref {
            incoming.push((
                rel_type,
                Self::node_from_entity(entity, entity_map),
            ));
        }
    }

    fn check_array_ref_incoming(
        field_value: &serde_yaml::Value,
        default_kind: &str,
        center_ref: &str,
        rel_type: RelationType,
        entity: &EntityWithSource,
        entity_map: &HashMap<String, &EntityWithSource>,
        incoming: &mut Vec<(RelationType, EntityNode)>,
    ) -> bool {
        if let Some(arr) = field_value.as_sequence() {
            for item in arr {
                if let Some(item_str) = item.as_str() {
                    let parsed = EntityRef::parse(item_str, default_kind);
                    if parsed.canonical() == center_ref {
                        incoming.push((
                            rel_type.clone(),
                            Self::node_from_entity(entity, entity_map),
                        ));
                        return true;
                    }
                }
            }
        }
        false
    }

    fn extract_incoming_relationships(
        center_ref: &str,
        all_entities: &[EntityWithSource],
        entity_map: &HashMap<String, &EntityWithSource>,
        incoming: &mut Vec<(RelationType, EntityNode)>,
    ) {
        for other in all_entities {
            if other.entity.ref_key() == center_ref {
                continue;
            }

            if let Some(owner) = other.entity.owner() {
                Self::check_single_ref_incoming(&owner, "group", center_ref, RelationType::Owner, other, entity_map, incoming);
            }

            if let Some(system) = other.entity.system() {
                Self::check_single_ref_incoming(&system, "system", center_ref, RelationType::System, other, entity_map, incoming);
            }

            if let Some(domain) = other.entity.domain() {
                Self::check_single_ref_incoming(&domain, "domain", center_ref, RelationType::Domain, other, entity_map, incoming);
            }

            if let Some(parent) = other.entity.get_spec_string("parent") {
                Self::check_single_ref_incoming(&parent, "group", center_ref, RelationType::Child, other, entity_map, incoming);
            }

            if let Some(deps) = other.entity.spec.get("dependsOn") {
                Self::check_array_ref_incoming(deps, "component", center_ref, RelationType::DependencyOf, other, entity_map, incoming);
            }

            if let Some(apis) = other.entity.spec.get("consumesApis") {
                Self::check_array_ref_incoming(apis, "api", center_ref, RelationType::ConsumedBy, other, entity_map, incoming);
            }

            if let Some(apis) = other.entity.spec.get("providesApis") {
                Self::check_array_ref_incoming(apis, "api", center_ref, RelationType::ProvidedBy, other, entity_map, incoming);
            }

            if let Some(groups) = other.entity.spec.get("memberOf") {
                Self::check_array_ref_incoming(groups, "group", center_ref, RelationType::HasMember, other, entity_map, incoming);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::{Entity, EntityKind, Metadata};
    use serde_yaml::Value;
    use std::path::PathBuf;

    fn create_entity(
        kind: EntityKind,
        name: &str,
        spec: Value,
    ) -> EntityWithSource {
        EntityWithSource::new(
            Entity {
                api_version: "backstage.io/v1alpha1".to_string(),
                kind,
                metadata: Metadata {
                    name: name.to_string(),
                    title: None,
                    namespace: Some("default".to_string()),
                    description: None,
                    labels: HashMap::new(),
                    annotations: HashMap::new(),
                    tags: Vec::new(),
                    links: Vec::new(),
                },
                spec,
            },
            PathBuf::from("test.yaml"),
        )
    }

    fn create_component(name: &str, spec: Value) -> EntityWithSource {
        create_entity(EntityKind::Component, name, spec)
    }

    fn create_api(name: &str, spec: Value) -> EntityWithSource {
        create_entity(EntityKind::Api, name, spec)
    }

    fn create_system(name: &str, spec: Value) -> EntityWithSource {
        create_entity(EntityKind::System, name, spec)
    }

    fn create_domain(name: &str, spec: Value) -> EntityWithSource {
        create_entity(EntityKind::Domain, name, spec)
    }

    fn create_group(name: &str, spec: Value) -> EntityWithSource {
        create_entity(EntityKind::Group, name, spec)
    }

    fn create_user(name: &str, spec: Value) -> EntityWithSource {
        create_entity(EntityKind::User, name, spec)
    }

    #[test]
    fn test_relationship_graph_building() {
        // Create a component with multiple relationships
        let component_spec = serde_yaml::from_str(
            r#"
            owner: team-a
            system: payment-system
            domain: finance
            dependsOn:
              - component:default/auth-service
            providesApis:
              - payment-api
            consumesApis:
              - api:default/auth-api
            "#,
        )
        .unwrap();

        let component = create_component("payment-service", component_spec);

        // Create related entities
        let team = create_group("team-a", Value::Null);
        let system = create_system("payment-system", Value::Null);
        let domain = create_domain("finance", Value::Null);
        let auth_service = create_component("auth-service", Value::Null);
        let payment_api = create_api("payment-api", Value::Null);
        let auth_api = create_api("auth-api", Value::Null);

        let all_entities = vec![
            component.clone(),
            team,
            system,
            domain,
            auth_service,
            payment_api,
            auth_api,
        ];

        let graph = RelationshipGraph::build(&component, &all_entities);

        // Verify center node
        assert_eq!(graph.center.display_name, "payment-service");
        assert_eq!(graph.center.kind, "Component");
        assert!(graph.center.exists);

        // Verify outgoing relationships exist
        assert_eq!(graph.outgoing.len(), 6);

        // Check relationship types are present
        let outgoing_types: Vec<RelationType> =
            graph.outgoing.iter().map(|(t, _)| t.clone()).collect();
        assert!(outgoing_types.contains(&RelationType::Owner));
        assert!(outgoing_types.contains(&RelationType::System));
        assert!(outgoing_types.contains(&RelationType::Domain));
        assert!(outgoing_types.contains(&RelationType::DependsOn));
        assert!(outgoing_types.contains(&RelationType::ProvidesApi));
        assert!(outgoing_types.contains(&RelationType::ConsumesApi));
    }

    #[test]
    fn test_outgoing_relationships() {
        let component_spec = serde_yaml::from_str(
            r#"
            owner: engineering-team
            system: platform
            domain: infrastructure
            "#,
        )
        .unwrap();

        let component = create_component("core-service", component_spec);
        let team = create_group("engineering-team", Value::Null);
        let system = create_system("platform", Value::Null);
        let domain = create_domain("infrastructure", Value::Null);

        let all_entities = vec![component.clone(), team, system, domain];
        let graph = RelationshipGraph::build(&component, &all_entities);

        // Find owner relationship
        let owner_rel = graph
            .outgoing
            .iter()
            .find(|(t, _)| *t == RelationType::Owner)
            .expect("Owner relationship should exist");
        assert_eq!(owner_rel.1.display_name, "engineering-team");
        assert_eq!(owner_rel.1.kind, "group");
        assert!(owner_rel.1.exists);

        // Find system relationship
        let system_rel = graph
            .outgoing
            .iter()
            .find(|(t, _)| *t == RelationType::System)
            .expect("System relationship should exist");
        assert_eq!(system_rel.1.display_name, "platform");
        assert_eq!(system_rel.1.kind, "system");
        assert!(system_rel.1.exists);

        // Find domain relationship
        let domain_rel = graph
            .outgoing
            .iter()
            .find(|(t, _)| *t == RelationType::Domain)
            .expect("Domain relationship should exist");
        assert_eq!(domain_rel.1.display_name, "infrastructure");
        assert_eq!(domain_rel.1.kind, "domain");
        assert!(domain_rel.1.exists);
    }

    #[test]
    fn test_incoming_relationships() {
        // Create a system that other components are part of
        let system = create_system("payment-system", Value::Null);

        // Create components that reference this system
        let comp1_spec = serde_yaml::from_str(
            r#"
            system: payment-system
            owner: team-a
            "#,
        )
        .unwrap();

        let comp2_spec = serde_yaml::from_str(
            r#"
            system: payment-system
            owner: team-b
            "#,
        )
        .unwrap();

        let comp1 = create_component("payment-frontend", comp1_spec);
        let comp2 = create_component("payment-backend", comp2_spec);

        let all_entities = vec![system.clone(), comp1, comp2];
        let graph = RelationshipGraph::build(&system, &all_entities);

        // Should have 2 incoming System relationships
        let system_incoming: Vec<_> = graph
            .incoming
            .iter()
            .filter(|(t, _)| *t == RelationType::System)
            .collect();
        assert_eq!(system_incoming.len(), 2);

        let names: Vec<&str> = system_incoming
            .iter()
            .map(|(_, n)| n.display_name.as_str())
            .collect();
        assert!(names.contains(&"payment-frontend"));
        assert!(names.contains(&"payment-backend"));
    }

    #[test]
    fn test_member_relationships() {
        // Test User -> Group (MemberOf) and Group -> User (HasMember)
        let user_spec = serde_yaml::from_str(
            r#"
            memberOf:
              - engineering
              - platform-team
            "#,
        )
        .unwrap();

        let user = create_user("john-doe", user_spec);
        let group1 = create_group("engineering", Value::Null);
        let group2 = create_group("platform-team", Value::Null);

        let all_entities = vec![user.clone(), group1.clone(), group2.clone()];

        // Check user's outgoing relationships
        let user_graph = RelationshipGraph::build(&user, &all_entities);
        let member_of: Vec<_> = user_graph
            .outgoing
            .iter()
            .filter(|(t, _)| *t == RelationType::MemberOf)
            .collect();
        assert_eq!(member_of.len(), 2);

        let group_names: Vec<&str> = member_of
            .iter()
            .map(|(_, n)| n.display_name.as_str())
            .collect();
        assert!(group_names.contains(&"engineering"));
        assert!(group_names.contains(&"platform-team"));

        // Check group's incoming relationships
        let group_graph = RelationshipGraph::build(&group1, &all_entities);
        let has_member: Vec<_> = group_graph
            .incoming
            .iter()
            .filter(|(t, _)| *t == RelationType::HasMember)
            .collect();
        assert_eq!(has_member.len(), 1);
        assert_eq!(has_member[0].1.display_name, "john-doe");
    }

    #[test]
    fn test_bidirectional_relationships() {
        // Test component dependency and API consumption/provision
        let frontend_spec = serde_yaml::from_str(
            r#"
            dependsOn:
              - backend-service
            consumesApis:
              - user-api
            "#,
        )
        .unwrap();

        let backend_spec = serde_yaml::from_str(
            r#"
            providesApis:
              - user-api
            "#,
        )
        .unwrap();

        let frontend = create_component("frontend-app", frontend_spec);
        let backend = create_component("backend-service", backend_spec);
        let api = create_api("user-api", Value::Null);

        let all_entities = vec![frontend.clone(), backend.clone(), api.clone()];

        // Frontend should have DependsOn outgoing to backend
        let frontend_graph = RelationshipGraph::build(&frontend, &all_entities);
        let depends_on = frontend_graph
            .outgoing
            .iter()
            .find(|(t, _)| *t == RelationType::DependsOn)
            .expect("DependsOn relationship should exist");
        assert_eq!(depends_on.1.display_name, "backend-service");

        // Backend should have DependencyOf incoming from frontend
        let backend_graph = RelationshipGraph::build(&backend, &all_entities);
        let dependency_of = backend_graph
            .incoming
            .iter()
            .find(|(t, _)| *t == RelationType::DependencyOf)
            .expect("DependencyOf relationship should exist");
        assert_eq!(dependency_of.1.display_name, "frontend-app");

        // API should have ProvidedBy incoming from backend
        let api_graph = RelationshipGraph::build(&api, &all_entities);
        let provided_by = api_graph
            .incoming
            .iter()
            .find(|(t, _)| *t == RelationType::ProvidedBy)
            .expect("ProvidedBy relationship should exist");
        assert_eq!(provided_by.1.display_name, "backend-service");

        // API should have ConsumedBy incoming from frontend
        let consumed_by = api_graph
            .incoming
            .iter()
            .find(|(t, _)| *t == RelationType::ConsumedBy)
            .expect("ConsumedBy relationship should exist");
        assert_eq!(consumed_by.1.display_name, "frontend-app");
    }

    #[test]
    fn test_missing_entity_references() {
        // Create component referencing non-existent entities
        let component_spec = serde_yaml::from_str(
            r#"
            owner: nonexistent-team
            system: nonexistent-system
            domain: nonexistent-domain
            dependsOn:
              - nonexistent-service
            providesApis:
              - nonexistent-api
            "#,
        )
        .unwrap();

        let component = create_component("isolated-service", component_spec);

        // Build graph with only the component itself
        let all_entities = vec![component.clone()];
        let graph = RelationshipGraph::build(&component, &all_entities);

        // All outgoing relationships should exist but with exists=false
        assert_eq!(graph.outgoing.len(), 5);

        for (_, node) in &graph.outgoing {
            assert!(
                !node.exists,
                "Referenced entity '{}' should not exist",
                node.display_name
            );
        }

        // Verify specific non-existent references
        let owner = graph
            .outgoing
            .iter()
            .find(|(t, _)| *t == RelationType::Owner)
            .unwrap();
        assert_eq!(owner.1.display_name, "nonexistent-team");
        assert!(!owner.1.exists);

        let system = graph
            .outgoing
            .iter()
            .find(|(t, _)| *t == RelationType::System)
            .unwrap();
        assert_eq!(system.1.display_name, "nonexistent-system");
        assert!(!system.1.exists);
    }

    #[test]
    fn test_parent_child_group_relationships() {
        // Test Group parent-child relationships
        let child_spec = serde_yaml::from_str(
            r#"
            parent: engineering
            "#,
        )
        .unwrap();

        let parent_spec = serde_yaml::from_str(
            r#"
            children:
              - platform-team
              - infrastructure-team
            "#,
        )
        .unwrap();

        let parent = create_group("engineering", parent_spec);
        let child1 = create_group("platform-team", child_spec);
        let child2 = create_group("infrastructure-team", Value::Null);

        let all_entities = vec![parent.clone(), child1.clone(), child2];

        // Child should have Parent outgoing relationship
        let child_graph = RelationshipGraph::build(&child1, &all_entities);
        let parent_rel = child_graph
            .outgoing
            .iter()
            .find(|(t, _)| *t == RelationType::Parent)
            .expect("Parent relationship should exist");
        assert_eq!(parent_rel.1.display_name, "engineering");

        // Parent should have Child incoming relationship
        let parent_graph = RelationshipGraph::build(&parent, &all_entities);
        let child_incoming: Vec<_> = parent_graph
            .incoming
            .iter()
            .filter(|(t, _)| *t == RelationType::Child)
            .collect();
        assert_eq!(child_incoming.len(), 1);
        assert_eq!(child_incoming[0].1.display_name, "platform-team");

        // Parent should have Child outgoing relationships
        let child_outgoing: Vec<_> = parent_graph
            .outgoing
            .iter()
            .filter(|(t, _)| *t == RelationType::Child)
            .collect();
        assert_eq!(child_outgoing.len(), 2);

        let child_names: Vec<&str> = child_outgoing
            .iter()
            .map(|(_, n)| n.display_name.as_str())
            .collect();
        assert!(child_names.contains(&"platform-team"));
        assert!(child_names.contains(&"infrastructure-team"));
    }

    #[test]
    fn test_empty_relationships() {
        // Component with no relationships
        let component = create_component("standalone-service", Value::Null);
        let all_entities = vec![component.clone()];

        let graph = RelationshipGraph::build(&component, &all_entities);

        assert_eq!(graph.outgoing.len(), 0);
        assert_eq!(graph.incoming.len(), 0);
        assert_eq!(graph.center.display_name, "standalone-service");
    }

    #[test]
    fn test_relation_type_labels() {
        assert_eq!(RelationType::Owner.label(), "owned by");
        assert_eq!(RelationType::System.label(), "part of");
        assert_eq!(RelationType::Domain.label(), "in domain");
        assert_eq!(RelationType::Parent.label(), "parent");
        assert_eq!(RelationType::Child.label(), "child");
        assert_eq!(RelationType::DependsOn.label(), "depends on");
        assert_eq!(RelationType::DependencyOf.label(), "dependency of");
        assert_eq!(RelationType::ProvidesApi.label(), "provides");
        assert_eq!(RelationType::ConsumesApi.label(), "consumes");
        assert_eq!(RelationType::ProvidedBy.label(), "provided by");
        assert_eq!(RelationType::ConsumedBy.label(), "consumed by");
        assert_eq!(RelationType::MemberOf.label(), "member of");
        assert_eq!(RelationType::HasMember.label(), "has member");
    }

    #[test]
    fn test_multiple_dependencies() {
        // Component with multiple dependencies
        let component_spec = serde_yaml::from_str(
            r#"
            dependsOn:
              - auth-service
              - database-service
              - cache-service
            "#,
        )
        .unwrap();

        let component = create_component("api-gateway", component_spec);
        let auth = create_component("auth-service", Value::Null);
        let db = create_component("database-service", Value::Null);
        let cache = create_component("cache-service", Value::Null);

        let all_entities = vec![component.clone(), auth, db, cache];
        let graph = RelationshipGraph::build(&component, &all_entities);

        // Should have 3 DependsOn relationships
        let deps: Vec<_> = graph
            .outgoing
            .iter()
            .filter(|(t, _)| *t == RelationType::DependsOn)
            .collect();
        assert_eq!(deps.len(), 3);

        let dep_names: Vec<&str> = deps.iter().map(|(_, n)| n.display_name.as_str()).collect();
        assert!(dep_names.contains(&"auth-service"));
        assert!(dep_names.contains(&"database-service"));
        assert!(dep_names.contains(&"cache-service"));
    }

    #[test]
    fn test_api_provider_consumer_relationships() {
        // Test complete API relationship chain
        let provider_spec = serde_yaml::from_str(
            r#"
            providesApis:
              - payments-api
              - users-api
            "#,
        )
        .unwrap();

        let consumer_spec = serde_yaml::from_str(
            r#"
            consumesApis:
              - payments-api
              - users-api
            "#,
        )
        .unwrap();

        let provider = create_component("backend", provider_spec);
        let consumer = create_component("frontend", consumer_spec);
        let api1 = create_api("payments-api", Value::Null);
        let api2 = create_api("users-api", Value::Null);

        let all_entities = vec![provider.clone(), consumer.clone(), api1.clone(), api2.clone()];

        // Provider should have ProvidesApi outgoing
        let provider_graph = RelationshipGraph::build(&provider, &all_entities);
        let provides: Vec<_> = provider_graph
            .outgoing
            .iter()
            .filter(|(t, _)| *t == RelationType::ProvidesApi)
            .collect();
        assert_eq!(provides.len(), 2);

        // Consumer should have ConsumesApi outgoing
        let consumer_graph = RelationshipGraph::build(&consumer, &all_entities);
        let consumes: Vec<_> = consumer_graph
            .outgoing
            .iter()
            .filter(|(t, _)| *t == RelationType::ConsumesApi)
            .collect();
        assert_eq!(consumes.len(), 2);

        // API should have both ProvidedBy and ConsumedBy incoming
        let api_graph = RelationshipGraph::build(&api1, &all_entities);
        assert_eq!(
            api_graph
                .incoming
                .iter()
                .filter(|(t, _)| *t == RelationType::ProvidedBy)
                .count(),
            1
        );
        assert_eq!(
            api_graph
                .incoming
                .iter()
                .filter(|(t, _)| *t == RelationType::ConsumedBy)
                .count(),
            1
        );
    }
}
