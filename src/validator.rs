//! JSON Schema validation for Backstage entities.
//!
//! This module validates Backstage entities against the official catalog JSON Schema.
//! The schema is embedded at compile time and validates entity structure, required fields,
//! and field types according to Backstage specifications.
//!
//! # Examples
//!
//! ## Validating a Component
//!
//! ```
//! use bsv::entity::{Entity, EntityKind, Metadata};
//! use bsv::validator::validate_entity;
//! use std::collections::HashMap;
//!
//! // Create a valid component
//! let mut spec = serde_yaml::Mapping::new();
//! spec.insert(
//!     serde_yaml::Value::String("type".to_string()),
//!     serde_yaml::Value::String("service".to_string()),
//! );
//! spec.insert(
//!     serde_yaml::Value::String("lifecycle".to_string()),
//!     serde_yaml::Value::String("production".to_string()),
//! );
//! spec.insert(
//!     serde_yaml::Value::String("owner".to_string()),
//!     serde_yaml::Value::String("team-a".to_string()),
//! );
//!
//! let entity = Entity {
//!     api_version: "backstage.io/v1alpha1".to_string(),
//!     kind: EntityKind::Component,
//!     metadata: Metadata {
//!         name: "my-service".to_string(),
//!         title: None,
//!         namespace: None,
//!         description: None,
//!         labels: HashMap::new(),
//!         annotations: HashMap::new(),
//!         tags: Vec::new(),
//!         links: Vec::new(),
//!     },
//!     spec: serde_yaml::Value::Mapping(spec),
//! };
//!
//! // Validate
//! let errors = validate_entity(&entity);
//! assert!(errors.is_empty(), "Valid entity should have no errors");
//! ```
//!
//! ## Handling Validation Errors
//!
//! ```
//! use bsv::entity::{Entity, EntityKind, Metadata};
//! use bsv::validator::validate_entity;
//! use std::collections::HashMap;
//!
//! // Create an invalid component (missing required fields)
//! let entity = Entity {
//!     api_version: "backstage.io/v1alpha1".to_string(),
//!     kind: EntityKind::Component,
//!     metadata: Metadata {
//!         name: "invalid-service".to_string(),
//!         title: None,
//!         namespace: None,
//!         description: None,
//!         labels: HashMap::new(),
//!         annotations: HashMap::new(),
//!         tags: Vec::new(),
//!         links: Vec::new(),
//!     },
//!     spec: serde_yaml::Value::Mapping(serde_yaml::Mapping::new()),
//! };
//!
//! let errors = validate_entity(&entity);
//! assert!(!errors.is_empty(), "Invalid entity should have errors");
//!
//! for error in errors {
//!     println!("Validation error at {}: {}", error.path, error.message);
//! }
//! ```
//!
//! # Key Types and Functions
//!
//! - [`validate_entity`] - Validate an entity against the JSON Schema
//! - Schema is automatically loaded and compiled on first use

use crate::entity::{Entity, ValidationError};
use jsonschema::Validator;
use once_cell::sync::Lazy;
use serde_json::Value as JsonValue;

/// Embedded Backstage catalog JSON Schema
static SCHEMA_STR: &str = include_str!("../schema/catalog-info.json");

/// Compiled JSON Schema validator (initialized once)
static SCHEMA: Lazy<Validator> = Lazy::new(|| {
    let schema_json: JsonValue =
        serde_json::from_str(SCHEMA_STR).expect("Failed to parse embedded JSON schema");
    jsonschema::validator_for(&schema_json).expect("Failed to compile JSON schema")
});

/// Validate an entity against the Backstage catalog JSON Schema
pub fn validate_entity(entity: &Entity) -> Vec<ValidationError> {
    // Convert entity to JSON for validation
    let entity_json = match serde_json::to_value(entity) {
        Ok(json) => json,
        Err(e) => {
            return vec![ValidationError {
                path: "/".to_string(),
                message: format!("Failed to serialize entity to JSON: {e}"),
            }];
        }
    };

    // Validate against schema and collect errors
    SCHEMA
        .iter_errors(&entity_json)
        .map(|error| {
            let path = error.instance_path().to_string();
            let path = if path.is_empty() {
                "/".to_string()
            } else {
                path
            };
            ValidationError {
                path,
                message: error.to_string(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::{Entity, EntityKind, Metadata};
    use std::collections::HashMap;

    #[test]
    fn test_valid_component() {
        let mut spec = serde_yaml::Mapping::new();
        spec.insert(
            serde_yaml::Value::String("type".to_string()),
            serde_yaml::Value::String("service".to_string()),
        );
        spec.insert(
            serde_yaml::Value::String("lifecycle".to_string()),
            serde_yaml::Value::String("production".to_string()),
        );
        spec.insert(
            serde_yaml::Value::String("owner".to_string()),
            serde_yaml::Value::String("team-a".to_string()),
        );

        let entity = Entity {
            api_version: "backstage.io/v1alpha1".to_string(),
            kind: EntityKind::Component,
            metadata: Metadata {
                name: "my-service".to_string(),
                title: None,
                namespace: None,
                description: None,
                labels: HashMap::new(),
                annotations: HashMap::new(),
                tags: Vec::new(),
                links: Vec::new(),
            },
            spec: serde_yaml::Value::Mapping(spec),
        };

        let errors = validate_entity(&entity);
        if !errors.is_empty() {
            eprintln!("Validation errors found:");
            for error in &errors {
                eprintln!("  Path: {}", error.path);
                eprintln!("  Message: {}", error.message);
            }
        }
        assert!(
            errors.is_empty(),
            "Valid component should have no errors: found {} errors",
            errors.len()
        );
    }

    #[test]
    fn test_missing_required_field() {
        let spec = serde_yaml::Mapping::new(); // Empty spec - missing required fields

        let entity = Entity {
            api_version: "backstage.io/v1alpha1".to_string(),
            kind: EntityKind::Component,
            metadata: Metadata {
                name: "my-service".to_string(),
                title: None,
                namespace: None,
                description: None,
                labels: HashMap::new(),
                annotations: HashMap::new(),
                tags: Vec::new(),
                links: Vec::new(),
            },
            spec: serde_yaml::Value::Mapping(spec),
        };

        let errors = validate_entity(&entity);
        assert!(!errors.is_empty(), "Invalid component should have errors");
    }
}
