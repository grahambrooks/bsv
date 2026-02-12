//! Entity models, reference parsing, and validation for Backstage entities.
//!
//! This module provides the core data structures for representing Backstage catalog entities,
//! including Components, APIs, Resources, Systems, Domains, Groups, Users, and Locations.
//! It also handles parsing of entity references in the Backstage format and provides an
//! index for fast reference validation.
//!
//! # Examples
//!
//! ## Parsing Entity References
//!
//! Entity references in Backstage follow the format `[<kind>:][<namespace>/]<name>`:
//!
//! ```
//! use bsv::entity::EntityRef;
//!
//! // Parse a full reference
//! let ref1 = EntityRef::parse("component:default/my-service", "component");
//! assert_eq!(ref1.kind, "component");
//! assert_eq!(ref1.namespace, "default");
//! assert_eq!(ref1.name, "my-service");
//!
//! // Parse with defaults (kind and namespace inferred)
//! let ref2 = EntityRef::parse("my-service", "component");
//! assert_eq!(ref2.canonical(), "component:default/my-service");
//! ```
//!
//! ## Building an Entity Index
//!
//! The entity index provides O(1) lookup for reference validation:
//!
//! ```
//! # use bsv::entity::{Entity, EntityKind, EntityIndex, EntityWithSource, EntityRef, Metadata};
//! # use std::path::PathBuf;
//! # use std::collections::HashMap;
//! # let entity = Entity {
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
//! #     spec: serde_yaml::Value::Null,
//! # };
//! let entities = vec![
//!     EntityWithSource::new(entity, PathBuf::from("catalog.yaml")),
//! ];
//! let index = EntityIndex::build(&entities);
//!
//! // Validate references
//! let valid_ref = EntityRef::parse("component:default/my-service", "component");
//! assert!(index.contains(&valid_ref));
//!
//! let invalid_ref = EntityRef::parse("nonexistent", "component");
//! assert!(!index.contains(&invalid_ref));
//! ```
//!
//! # Key Types
//!
//! - [`Entity`] - Core Backstage entity with metadata and spec
//! - [`EntityKind`] - Enumeration of all supported entity types
//! - [`EntityRef`] - Parsed entity reference with kind, namespace, and name
//! - [`EntityIndex`] - Fast lookup index for reference validation
//! - [`EntityWithSource`] - Entity wrapper tracking source file and validation errors
//! - [`ValidationError`] - Structured validation error from JSON Schema

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Validation error from JSON Schema validation
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub path: String,
    pub message: String,
}

/// Parsed entity reference with resolved kind and namespace
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EntityRef {
    pub kind: String,
    pub namespace: String,
    pub name: String,
    pub kind_inferred: bool,
    pub namespace_inferred: bool,
}

impl EntityRef {
    /// Parse an entity reference string with a default kind for the context
    /// 
    /// Format: `[kind:]` `[namespace/]` `name`
    pub fn parse(reference: &str, default_kind: &str) -> Self {
        let (kind, rest, kind_inferred) = if let Some(idx) = reference.find(':') {
            (
                reference[..idx].to_lowercase(),
                &reference[idx + 1..],
                false,
            )
        } else {
            (default_kind.to_lowercase(), reference, true)
        };

        let (namespace, name, namespace_inferred) = if let Some(idx) = rest.find('/') {
            (rest[..idx].to_string(), rest[idx + 1..].to_string(), false)
        } else {
            ("default".to_string(), rest.to_string(), true)
        };

        EntityRef {
            kind,
            namespace,
            name,
            kind_inferred,
            namespace_inferred,
        }
    }

    /// Get the canonical reference string
    pub fn canonical(&self) -> String {
        format!("{}:{}/{}", self.kind, self.namespace, self.name)
    }

    /// Check if the kind is a known Backstage kind
    pub fn is_known_kind(&self) -> bool {
        matches!(
            self.kind.as_str(),
            "component" | "api" | "resource" | "system" | "domain" | "group" | "user" | "location"
        )
    }
}

impl std::fmt::Display for EntityRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}/{}", self.kind, self.namespace, self.name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityKind {
    Component,
    #[serde(alias = "API")]
    Api,
    Resource,
    System,
    Domain,
    Group,
    User,
    Location,
    #[serde(other)]
    Unknown,
}

impl std::fmt::Display for EntityKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntityKind::Component => write!(f, "Component"),
            EntityKind::Api => write!(f, "API"),
            EntityKind::Resource => write!(f, "Resource"),
            EntityKind::System => write!(f, "System"),
            EntityKind::Domain => write!(f, "Domain"),
            EntityKind::Group => write!(f, "Group"),
            EntityKind::User => write!(f, "User"),
            EntityKind::Location => write!(f, "Location"),
            EntityKind::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub labels: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub annotations: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<Link>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entity {
    pub api_version: String,
    pub kind: EntityKind,
    pub metadata: Metadata,
    #[serde(default)]
    pub spec: serde_yaml::Value,
}

#[derive(Debug, Clone)]
pub struct EntityWithSource {
    pub entity: Entity,
    pub source_file: PathBuf,
    pub validation_errors: Vec<ValidationError>,
}

impl EntityWithSource {
    pub fn new(entity: Entity, source_file: PathBuf) -> Self {
        Self {
            entity,
            source_file,
            validation_errors: Vec::new(),
        }
    }

    pub fn with_validation_errors(mut self, errors: Vec<ValidationError>) -> Self {
        self.validation_errors = errors;
        self
    }
}

impl Entity {
    /// Get display name, preferring title over name if available.
    pub fn display_name(&self) -> String {
        self.metadata
            .title
            .clone()
            .unwrap_or_else(|| self.metadata.name.clone())
    }

    pub fn get_spec_string(&self, key: &str) -> Option<String> {
        self.spec
            .get(key)
            .and_then(|v| v.as_str())
            .map(String::from)
    }

    pub fn system(&self) -> Option<String> {
        self.get_spec_string("system")
    }

    pub fn domain(&self) -> Option<String> {
        self.get_spec_string("domain")
    }

    pub fn owner(&self) -> Option<String> {
        self.get_spec_string("owner")
    }

    pub fn lifecycle(&self) -> Option<String> {
        self.get_spec_string("lifecycle")
    }

    pub fn entity_type(&self) -> Option<String> {
        self.get_spec_string("type")
    }

    /// Get the canonical reference key for this entity
    pub fn ref_key(&self) -> String {
        let kind = self.kind.to_string().to_lowercase();
        let namespace = self.metadata.namespace.as_deref().unwrap_or("default");
        format!("{}:{}/{}", kind, namespace, self.metadata.name)
    }
}

/// Index of all loaded entities for reference validation
#[derive(Debug, Clone, Default)]
pub struct EntityIndex {
    keys: HashSet<String>,
}

impl EntityIndex {
    /// Build an index from a list of entities for O(1) reference validation.
    pub fn build(entities: &[EntityWithSource]) -> Self {
        let keys = entities.iter().map(|e| e.entity.ref_key()).collect();
        Self { keys }
    }

    /// Check if the given entity reference exists in the index.
    pub fn contains(&self, entity_ref: &EntityRef) -> bool {
        self.keys.contains(&entity_ref.canonical())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_ref_parsing_variations() {
        // Full format: "component:default/my-service"
        let ref1 = EntityRef::parse("component:default/my-service", "component");
        assert_eq!(ref1.kind, "component");
        assert_eq!(ref1.namespace, "default");
        assert_eq!(ref1.name, "my-service");
        assert!(!ref1.kind_inferred);
        assert!(!ref1.namespace_inferred);

        // Just name (infer kind and namespace): "my-service"
        let ref2 = EntityRef::parse("my-service", "component");
        assert_eq!(ref2.kind, "component");
        assert_eq!(ref2.namespace, "default");
        assert_eq!(ref2.name, "my-service");
        assert!(ref2.kind_inferred);
        assert!(ref2.namespace_inferred);

        // Kind and name: "api:my-api"
        let ref3 = EntityRef::parse("api:my-api", "component");
        assert_eq!(ref3.kind, "api");
        assert_eq!(ref3.namespace, "default");
        assert_eq!(ref3.name, "my-api");
        assert!(!ref3.kind_inferred);
        assert!(ref3.namespace_inferred);

        // Namespace and name: "default/my-service"
        let ref4 = EntityRef::parse("default/my-service", "system");
        assert_eq!(ref4.kind, "system");
        assert_eq!(ref4.namespace, "default");
        assert_eq!(ref4.name, "my-service");
        assert!(ref4.kind_inferred);
        assert!(!ref4.namespace_inferred);

        // Custom namespace
        let ref5 = EntityRef::parse("component:production/my-service", "component");
        assert_eq!(ref5.kind, "component");
        assert_eq!(ref5.namespace, "production");
        assert_eq!(ref5.name, "my-service");
        assert!(!ref5.kind_inferred);
        assert!(!ref5.namespace_inferred);

        // Edge case: empty string (becomes default namespace and empty name)
        let ref6 = EntityRef::parse("", "component");
        assert_eq!(ref6.kind, "component");
        assert_eq!(ref6.namespace, "default");
        assert_eq!(ref6.name, "");
        assert!(ref6.kind_inferred);
        assert!(ref6.namespace_inferred);

        // Edge case: special characters in name
        let ref7 = EntityRef::parse("my-service-123", "component");
        assert_eq!(ref7.name, "my-service-123");

        // Edge case: uppercase kind is lowercased
        let ref8 = EntityRef::parse("Component:my-service", "component");
        assert_eq!(ref8.kind, "component");

        // Edge case: just kind delimiter
        let ref9 = EntityRef::parse("api:", "component");
        assert_eq!(ref9.kind, "api");
        assert_eq!(ref9.namespace, "default");
        assert_eq!(ref9.name, "");

        // Edge case: just namespace delimiter
        let ref10 = EntityRef::parse("production/", "component");
        assert_eq!(ref10.kind, "component");
        assert_eq!(ref10.namespace, "production");
        assert_eq!(ref10.name, "");
    }

    #[test]
    fn test_entity_ref_canonical_format() {
        // Test that canonical() produces consistent output
        let ref1 = EntityRef::parse("component:default/my-service", "component");
        assert_eq!(ref1.canonical(), "component:default/my-service");

        let ref2 = EntityRef::parse("my-service", "component");
        assert_eq!(ref2.canonical(), "component:default/my-service");

        let ref3 = EntityRef::parse("api:production/my-api", "component");
        assert_eq!(ref3.canonical(), "api:production/my-api");

        // Test Display trait matches canonical
        assert_eq!(ref1.to_string(), ref1.canonical());
        assert_eq!(ref2.to_string(), ref2.canonical());
        assert_eq!(ref3.to_string(), ref3.canonical());
    }

    #[test]
    fn test_entity_ref_known_kinds() {
        // Test all known Backstage kinds
        assert!(EntityRef::parse("component:default/test", "component").is_known_kind());
        assert!(EntityRef::parse("api:default/test", "api").is_known_kind());
        assert!(EntityRef::parse("resource:default/test", "resource").is_known_kind());
        assert!(EntityRef::parse("system:default/test", "system").is_known_kind());
        assert!(EntityRef::parse("domain:default/test", "domain").is_known_kind());
        assert!(EntityRef::parse("group:default/test", "group").is_known_kind());
        assert!(EntityRef::parse("user:default/test", "user").is_known_kind());
        assert!(EntityRef::parse("location:default/test", "location").is_known_kind());

        // Test unknown kind
        assert!(!EntityRef::parse("custom:default/test", "custom").is_known_kind());
        assert!(!EntityRef::parse("widget:default/test", "widget").is_known_kind());

        // Case insensitive (kinds are lowercased in parse)
        assert!(EntityRef::parse("Component:default/test", "component").is_known_kind());
        assert!(EntityRef::parse("API:default/test", "api").is_known_kind());
    }

    #[test]
    fn test_entity_index_operations() {
        // Create test entities
        let entity1 = Entity {
            api_version: "backstage.io/v1alpha1".to_string(),
            kind: EntityKind::Component,
            metadata: Metadata {
                name: "service-a".to_string(),
                title: None,
                namespace: Some("default".to_string()),
                description: None,
                labels: HashMap::new(),
                annotations: HashMap::new(),
                tags: Vec::new(),
                links: Vec::new(),
            },
            spec: serde_yaml::Value::Null,
        };

        let entity2 = Entity {
            api_version: "backstage.io/v1alpha1".to_string(),
            kind: EntityKind::Api,
            metadata: Metadata {
                name: "api-b".to_string(),
                title: None,
                namespace: Some("production".to_string()),
                description: None,
                labels: HashMap::new(),
                annotations: HashMap::new(),
                tags: Vec::new(),
                links: Vec::new(),
            },
            spec: serde_yaml::Value::Null,
        };

        let entity3 = Entity {
            api_version: "backstage.io/v1alpha1".to_string(),
            kind: EntityKind::System,
            metadata: Metadata {
                name: "system-c".to_string(),
                title: None,
                namespace: None, // Should default to "default"
                description: None,
                labels: HashMap::new(),
                annotations: HashMap::new(),
                tags: Vec::new(),
                links: Vec::new(),
            },
            spec: serde_yaml::Value::Null,
        };

        let entities = vec![
            EntityWithSource::new(entity1, PathBuf::from("test1.yaml")),
            EntityWithSource::new(entity2, PathBuf::from("test2.yaml")),
            EntityWithSource::new(entity3, PathBuf::from("test3.yaml")),
        ];

        let index = EntityIndex::build(&entities);

        // Test contains() for existing entities
        assert!(index.contains(&EntityRef::parse("component:default/service-a", "component")));
        assert!(index.contains(&EntityRef::parse("service-a", "component"))); // Inferred
        assert!(index.contains(&EntityRef::parse("api:production/api-b", "api")));
        assert!(index.contains(&EntityRef::parse("system:default/system-c", "system")));

        // Test contains() for non-existing entities
        assert!(!index.contains(&EntityRef::parse("component:default/nonexistent", "component")));
        assert!(!index.contains(&EntityRef::parse("api:default/api-b", "api"))); // Wrong namespace
        assert!(!index.contains(&EntityRef::parse("component:production/service-a", "component"))); // Wrong namespace
    }

    #[test]
    fn test_entity_display_name() {
        // Test with title
        let entity_with_title = Entity {
            api_version: "backstage.io/v1alpha1".to_string(),
            kind: EntityKind::Component,
            metadata: Metadata {
                name: "service-a".to_string(),
                title: Some("Service A (Production)".to_string()),
                namespace: None,
                description: None,
                labels: HashMap::new(),
                annotations: HashMap::new(),
                tags: Vec::new(),
                links: Vec::new(),
            },
            spec: serde_yaml::Value::Null,
        };
        assert_eq!(entity_with_title.display_name(), "Service A (Production)");

        // Test without title (should return name)
        let entity_without_title = Entity {
            api_version: "backstage.io/v1alpha1".to_string(),
            kind: EntityKind::Component,
            metadata: Metadata {
                name: "service-b".to_string(),
                title: None,
                namespace: None,
                description: None,
                labels: HashMap::new(),
                annotations: HashMap::new(),
                tags: Vec::new(),
                links: Vec::new(),
            },
            spec: serde_yaml::Value::Null,
        };
        assert_eq!(entity_without_title.display_name(), "service-b");
    }

    #[test]
    fn test_entity_ref_key() {
        // Test with explicit namespace
        let entity1 = Entity {
            api_version: "backstage.io/v1alpha1".to_string(),
            kind: EntityKind::Component,
            metadata: Metadata {
                name: "my-service".to_string(),
                title: None,
                namespace: Some("production".to_string()),
                description: None,
                labels: HashMap::new(),
                annotations: HashMap::new(),
                tags: Vec::new(),
                links: Vec::new(),
            },
            spec: serde_yaml::Value::Null,
        };
        assert_eq!(entity1.ref_key(), "component:production/my-service");

        // Test with default namespace (None)
        let entity2 = Entity {
            api_version: "backstage.io/v1alpha1".to_string(),
            kind: EntityKind::Api,
            metadata: Metadata {
                name: "my-api".to_string(),
                title: None,
                namespace: None,
                description: None,
                labels: HashMap::new(),
                annotations: HashMap::new(),
                tags: Vec::new(),
                links: Vec::new(),
            },
            spec: serde_yaml::Value::Null,
        };
        assert_eq!(entity2.ref_key(), "api:default/my-api");

        // Test different kinds
        let entity3 = Entity {
            api_version: "backstage.io/v1alpha1".to_string(),
            kind: EntityKind::System,
            metadata: Metadata {
                name: "platform".to_string(),
                title: None,
                namespace: Some("default".to_string()),
                description: None,
                labels: HashMap::new(),
                annotations: HashMap::new(),
                tags: Vec::new(),
                links: Vec::new(),
            },
            spec: serde_yaml::Value::Null,
        };
        assert_eq!(entity3.ref_key(), "system:default/platform");
    }

    #[test]
    fn test_entity_kind_display() {
        assert_eq!(EntityKind::Component.to_string(), "Component");
        assert_eq!(EntityKind::Api.to_string(), "API");
        assert_eq!(EntityKind::Resource.to_string(), "Resource");
        assert_eq!(EntityKind::System.to_string(), "System");
        assert_eq!(EntityKind::Domain.to_string(), "Domain");
        assert_eq!(EntityKind::Group.to_string(), "Group");
        assert_eq!(EntityKind::User.to_string(), "User");
        assert_eq!(EntityKind::Location.to_string(), "Location");
        assert_eq!(EntityKind::Unknown.to_string(), "Unknown");
    }

    #[test]
    fn test_entity_spec_accessors() {
        let mut spec_map = serde_yaml::Mapping::new();
        spec_map.insert(
            serde_yaml::Value::String("system".to_string()),
            serde_yaml::Value::String("platform".to_string()),
        );
        spec_map.insert(
            serde_yaml::Value::String("domain".to_string()),
            serde_yaml::Value::String("payments".to_string()),
        );
        spec_map.insert(
            serde_yaml::Value::String("owner".to_string()),
            serde_yaml::Value::String("team-a".to_string()),
        );
        spec_map.insert(
            serde_yaml::Value::String("lifecycle".to_string()),
            serde_yaml::Value::String("production".to_string()),
        );
        spec_map.insert(
            serde_yaml::Value::String("type".to_string()),
            serde_yaml::Value::String("service".to_string()),
        );

        let entity = Entity {
            api_version: "backstage.io/v1alpha1".to_string(),
            kind: EntityKind::Component,
            metadata: Metadata {
                name: "test-service".to_string(),
                title: None,
                namespace: None,
                description: None,
                labels: HashMap::new(),
                annotations: HashMap::new(),
                tags: Vec::new(),
                links: Vec::new(),
            },
            spec: serde_yaml::Value::Mapping(spec_map),
        };

        assert_eq!(entity.system(), Some("platform".to_string()));
        assert_eq!(entity.domain(), Some("payments".to_string()));
        assert_eq!(entity.owner(), Some("team-a".to_string()));
        assert_eq!(entity.lifecycle(), Some("production".to_string()));
        assert_eq!(entity.entity_type(), Some("service".to_string()));

        // Test with empty spec
        let empty_entity = Entity {
            api_version: "backstage.io/v1alpha1".to_string(),
            kind: EntityKind::Component,
            metadata: Metadata {
                name: "test".to_string(),
                title: None,
                namespace: None,
                description: None,
                labels: HashMap::new(),
                annotations: HashMap::new(),
                tags: Vec::new(),
                links: Vec::new(),
            },
            spec: serde_yaml::Value::Null,
        };

        assert_eq!(empty_entity.system(), None);
        assert_eq!(empty_entity.domain(), None);
        assert_eq!(empty_entity.owner(), None);
        assert_eq!(empty_entity.lifecycle(), None);
        assert_eq!(empty_entity.entity_type(), None);
    }

    #[test]
    fn test_entity_with_source() {
        let entity = Entity {
            api_version: "backstage.io/v1alpha1".to_string(),
            kind: EntityKind::Component,
            metadata: Metadata {
                name: "test".to_string(),
                title: None,
                namespace: None,
                description: None,
                labels: HashMap::new(),
                annotations: HashMap::new(),
                tags: Vec::new(),
                links: Vec::new(),
            },
            spec: serde_yaml::Value::Null,
        };

        let source_path = PathBuf::from("/path/to/catalog-info.yaml");
        let entity_with_source = EntityWithSource::new(entity.clone(), source_path.clone());

        assert_eq!(entity_with_source.source_file, source_path);
        assert!(entity_with_source.validation_errors.is_empty());

        // Test with validation errors
        let errors = vec![
            ValidationError {
                path: "spec.owner".to_string(),
                message: "Required field missing".to_string(),
            },
            ValidationError {
                path: "metadata.name".to_string(),
                message: "Invalid format".to_string(),
            },
        ];

        let entity_with_errors = EntityWithSource::new(entity, source_path)
            .with_validation_errors(errors.clone());

        assert_eq!(entity_with_errors.validation_errors.len(), 2);
        assert_eq!(entity_with_errors.validation_errors[0].path, "spec.owner");
        assert_eq!(
            entity_with_errors.validation_errors[0].message,
            "Required field missing"
        );
    }

    #[test]
    fn test_entity_ref_hash_and_eq() {
        let ref1 = EntityRef::parse("component:default/my-service", "component");
        let ref2 = EntityRef::parse("component:default/my-service", "component");
        let ref3 = EntityRef::parse("component:default/other-service", "component");

        // Same reference should be equal
        assert_eq!(ref1, ref2);

        // Different references should not be equal
        assert_ne!(ref1, ref3);

        // Test HashSet usage
        let mut set = HashSet::new();
        set.insert(ref1.clone());
        assert!(set.contains(&ref2)); // Should find by value
        assert!(!set.contains(&ref3));

        // Test that inferred flags affect equality
        let ref4 = EntityRef::parse("my-service", "component");
        assert_ne!(ref1, ref4); // Different because of inferred flags

        // But canonical strings are the same
        assert_eq!(ref1.canonical(), ref4.canonical());
    }

    #[test]
    fn test_validation_error() {
        let error = ValidationError {
            path: "spec.type".to_string(),
            message: "Unknown type specified".to_string(),
        };

        assert_eq!(error.path, "spec.type");
        assert_eq!(error.message, "Unknown type specified");

        // Test clone
        let cloned = error.clone();
        assert_eq!(cloned.path, error.path);
        assert_eq!(cloned.message, error.message);
    }
}
