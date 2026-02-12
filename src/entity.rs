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
    /// Format: [<kind>:][<namespace>/]<name>
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
    pub fn build(entities: &[EntityWithSource]) -> Self {
        let keys = entities.iter().map(|e| e.entity.ref_key()).collect();
        Self { keys }
    }

    pub fn contains(&self, entity_ref: &EntityRef) -> bool {
        self.keys.contains(&entity_ref.canonical())
    }
}
