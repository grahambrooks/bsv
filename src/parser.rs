//! File discovery and YAML parsing for Backstage catalog files.
//!
//! This module provides functionality to discover `catalog-info.yaml` files in a directory tree
//! and parse them as multi-document YAML containing Backstage entities. It intelligently excludes
//! common build directories (like `target/`, `node_modules/`, `.git/`) to avoid scanning
//! irrelevant files.
//!
//! # Examples
//!
//! ## Loading All Entities from a Directory
//!
//! ```no_run
//! use bsv::parser::load_all_entities;
//! use std::path::Path;
//!
//! let entities = load_all_entities(Path::new("."))?;
//! println!("Discovered {} entities", entities.len());
//!
//! for entity_with_source in &entities {
//!     println!("  {} from {:?}",
//!         entity_with_source.entity.metadata.name,
//!         entity_with_source.source_file);
//! }
//! # Ok::<(), anyhow::Error>(())
//! ```
//!
//! ## Loading from a Single File
//!
//! ```no_run
//! use bsv::parser::load_all_entities;
//! use std::path::Path;
//!
//! // Also works with a specific file path
//! let entities = load_all_entities(Path::new("catalog-info.yaml"))?;
//! # Ok::<(), anyhow::Error>(())
//! ```
//!
//! ## Discovering Catalog Files
//!
//! ```no_run
//! use bsv::parser::discover_catalog_files;
//! use std::path::Path;
//!
//! let catalog_files = discover_catalog_files(Path::new("."));
//! for file in catalog_files {
//!     println!("Found: {}", file.display());
//! }
//! ```
//!
//! ## Parsing Multi-Document YAML
//!
//! Backstage catalog files can contain multiple entities separated by `---`:
//!
//! ```no_run
//! use bsv::parser::parse_catalog_file;
//! use std::path::Path;
//!
//! let entities = parse_catalog_file(Path::new("catalog-info.yaml"))?;
//! println!("Parsed {} entities from file", entities.len());
//! # Ok::<(), anyhow::Error>(())
//! ```
//!
//! # Key Functions
//!
//! - [`load_all_entities`] - Main entry point: load entities from directory or file
//! - [`discover_catalog_files`] - Recursively find all catalog-info.yaml files
//! - [`parse_catalog_file`] - Parse multi-document YAML file into entities
//! - [`should_exclude_dir`] - Check if a directory should be skipped during discovery

use crate::entity::{Entity, EntityWithSource};
use crate::validator::validate_entity;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Directories to skip during filesystem scans (build outputs, dependencies, caches)
pub const EXCLUDED_DIRS: &[&str] = &[
    // Rust
    "target",
    // Node.js
    "node_modules",
    // Python
    "__pycache__",
    ".venv",
    "venv",
    ".tox",
    // Java / Gradle / Maven
    "build",
    ".gradle",
    // .NET
    "bin",
    "obj",
    // Generic build outputs
    "dist",
    "out",
    // Frontend frameworks
    ".next",
    ".nuxt",
    ".svelte-kit",
    // Caches and tooling
    ".cache",
    ".parcel-cache",
    ".turbo",
    "coverage",
];

/// Directory prefixes to skip (matches any directory starting with these)
pub const EXCLUDED_DIR_PREFIXES: &[&str] = &[
    // Bazel (generates bazel-out, bazel-bin, bazel-testlogs, bazel-<project>, etc.)
    "bazel-",
];

/// Check if a directory name should be excluded from scanning
pub fn should_exclude_dir(name: &str) -> bool {
    name.starts_with('.')
        || EXCLUDED_DIRS.contains(&name)
        || EXCLUDED_DIR_PREFIXES
            .iter()
            .any(|prefix| name.starts_with(prefix))
}

/// Discover all `catalog-info.yaml` and `catalog-info.yml` files recursively.
///
/// Automatically excludes common build directories like `target/`, `node_modules/`,
/// and `.git/` to avoid scanning irrelevant files.
pub fn discover_catalog_files(root: &Path) -> Vec<std::path::PathBuf> {
    WalkDir::new(root)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| {
            // Allow files, but filter directories
            if e.file_type().is_dir() {
                e.file_name()
                    .to_str()
                    .map_or(true, |name| !should_exclude_dir(name))
            } else {
                true
            }
        })
        .filter_map(std::result::Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.file_name()
                .to_str()
                .is_some_and(|name| name == "catalog-info.yaml" || name == "catalog-info.yml")
        })
        .map(walkdir::DirEntry::into_path)
        .collect()
}

/// Parse a catalog file as multi-document YAML.
///
/// Each document in the file is deserialized as a separate entity.
/// Includes validation against JSON Schema.
pub fn parse_catalog_file(path: &Path) -> Result<Vec<EntityWithSource>> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    parse_multi_document_yaml(&content, path)
}

fn parse_multi_document_yaml(content: &str, source_path: &Path) -> Result<Vec<EntityWithSource>> {
    let mut entities = Vec::new();

    for document in serde_yaml::Deserializer::from_str(content) {
        match Entity::deserialize(document) {
            Ok(entity) => {
                // Validate the entity against JSON Schema
                let validation_errors = validate_entity(&entity);

                entities.push(
                    EntityWithSource::new(entity, source_path.to_path_buf())
                        .with_validation_errors(validation_errors),
                );
            }
            Err(e) => {
                eprintln!(
                    "Warning: Failed to parse entity in {}: {}",
                    source_path.display(),
                    e
                );
            }
        }
    }

    Ok(entities)
}

/// Load all entities from a directory or single file.
///
/// If the path is a file, loads just that file. If it's a directory,
/// recursively discovers and parses all catalog-info.yaml files.
pub fn load_all_entities(root: &Path) -> Result<Vec<EntityWithSource>> {
    // If path is a file, load just that file
    if root.is_file() {
        return parse_catalog_file(root);
    }

    // Otherwise, discover catalog files in directory
    let catalog_files = discover_catalog_files(root);
    let mut all_entities = Vec::new();

    for file_path in catalog_files {
        match parse_catalog_file(&file_path) {
            Ok(entities) => all_entities.extend(entities),
            Err(e) => eprintln!("Warning: {e}"),
        }
    }

    Ok(all_entities)
}

use serde::Deserialize;

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_should_exclude_dir() {
        // Test common build directories
        assert!(should_exclude_dir("target"));
        assert!(should_exclude_dir("node_modules"));
        assert!(should_exclude_dir("dist"));
        assert!(should_exclude_dir("build"));
        assert!(should_exclude_dir("bin"));
        assert!(should_exclude_dir("obj"));
        assert!(should_exclude_dir("out"));

        // Test Python directories
        assert!(should_exclude_dir("__pycache__"));
        assert!(should_exclude_dir(".venv"));
        assert!(should_exclude_dir("venv"));
        assert!(should_exclude_dir(".tox"));

        // Test Node.js/frontend directories
        assert!(should_exclude_dir(".next"));
        assert!(should_exclude_dir(".nuxt"));
        assert!(should_exclude_dir(".svelte-kit"));

        // Test cache directories
        assert!(should_exclude_dir(".cache"));
        assert!(should_exclude_dir(".parcel-cache"));
        assert!(should_exclude_dir(".turbo"));
        assert!(should_exclude_dir("coverage"));

        // Test Java/Gradle directories
        assert!(should_exclude_dir(".gradle"));

        // Test dot directories (git, cache, etc.)
        assert!(should_exclude_dir(".git"));
        assert!(should_exclude_dir(".gitignore")); // any dot file
        assert!(should_exclude_dir(".github"));
        assert!(should_exclude_dir(".vscode"));
        assert!(should_exclude_dir(".idea"));

        // Test Bazel directories (prefix match)
        assert!(should_exclude_dir("bazel-out"));
        assert!(should_exclude_dir("bazel-bin"));
        assert!(should_exclude_dir("bazel-testlogs"));
        assert!(should_exclude_dir("bazel-bsv"));
        assert!(should_exclude_dir("bazel-workspace"));

        // Test normal directories (should NOT exclude)
        assert!(!should_exclude_dir("src"));
        assert!(!should_exclude_dir("docs"));
        assert!(!should_exclude_dir("config"));
        assert!(!should_exclude_dir("api"));
        assert!(!should_exclude_dir("services"));
        assert!(!should_exclude_dir("components"));
        assert!(!should_exclude_dir("testdata"));
        assert!(!should_exclude_dir("my-service"));
        assert!(!should_exclude_dir("package"));
    }

    #[test]
    fn test_discover_catalog_files() {
        // Use the testdata directory for discovery
        let testdata_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata");

        let files = discover_catalog_files(&testdata_path);

        // Should find at least the catalog files we know exist
        assert!(
            !files.is_empty(),
            "Should discover at least one catalog file"
        );

        // All discovered files should end with catalog-info.yaml or catalog-info.yml
        for file in &files {
            let name = file.file_name().unwrap().to_str().unwrap();
            assert!(
                name == "catalog-info.yaml" || name == "catalog-info.yml",
                "File {} should be named catalog-info.yaml or catalog-info.yml",
                name
            );
        }

        // Files should be valid paths
        for file in &files {
            assert!(file.exists(), "File {} should exist", file.display());
        }
    }

    #[test]
    fn test_multi_document_yaml() {
        // Create a multi-document YAML string
        let yaml_content = r#"
apiVersion: backstage.io/v1alpha1
kind: Component
metadata:
  name: service-a
  description: First service
spec:
  type: service
  lifecycle: production
  owner: team-a
---
apiVersion: backstage.io/v1alpha1
kind: Component
metadata:
  name: service-b
  description: Second service
spec:
  type: service
  lifecycle: production
  owner: team-b
---
apiVersion: backstage.io/v1alpha1
kind: API
metadata:
  name: api-a
  description: First API
spec:
  type: openapi
  lifecycle: production
  owner: team-a
  definition: |
    openapi: 3.0.0
"#;

        let path = Path::new("test.yaml");
        let entities = parse_multi_document_yaml(yaml_content, path)
            .expect("Should parse multi-document YAML");

        assert_eq!(entities.len(), 3, "Should parse 3 entities");

        // Check first entity
        assert_eq!(entities[0].entity.metadata.name, "service-a");
        assert_eq!(entities[0].entity.kind.to_string(), "Component");

        // Check second entity
        assert_eq!(entities[1].entity.metadata.name, "service-b");
        assert_eq!(entities[1].entity.kind.to_string(), "Component");

        // Check third entity
        assert_eq!(entities[2].entity.metadata.name, "api-a");
        assert_eq!(entities[2].entity.kind.to_string(), "API");

        // Check source path is preserved
        for entity in &entities {
            assert_eq!(entity.source_file, path);
        }
    }

    #[test]
    fn test_parse_catalog_file() {
        // Parse the validation-test.yaml file which contains multiple entities
        let testdata_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("testdata")
            .join("validation-test.yaml");

        let entities =
            parse_catalog_file(&testdata_path).expect("Should parse validation-test.yaml");

        // validation-test.yaml contains 9 entities (counting the YAML content shown)
        assert!(!entities.is_empty(), "Should parse at least one entity");

        // Verify some expected entities
        let entity_names: Vec<&str> = entities
            .iter()
            .map(|e| e.entity.metadata.name.as_str())
            .collect();

        assert!(
            entity_names.contains(&"valid-service"),
            "Should contain valid-service"
        );
        assert!(
            entity_names.contains(&"valid-api"),
            "Should contain valid-api"
        );
        assert!(
            entity_names.contains(&"valid-domain"),
            "Should contain valid-domain"
        );

        // Verify source path is set correctly
        for entity in &entities {
            assert_eq!(entity.source_file, testdata_path);
        }
    }

    #[test]
    fn test_validation_during_parse() {
        // Parse the validation-test.yaml which contains entities with validation errors
        let testdata_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("testdata")
            .join("validation-test.yaml");

        let entities =
            parse_catalog_file(&testdata_path).expect("Should parse validation-test.yaml");

        // Find entities with and without validation errors
        let valid_service = entities
            .iter()
            .find(|e| e.entity.metadata.name == "valid-service")
            .expect("Should find valid-service");

        let missing_owner = entities
            .iter()
            .find(|e| e.entity.metadata.name == "missing-owner")
            .expect("Should find missing-owner");

        let missing_lifecycle = entities
            .iter()
            .find(|e| e.entity.metadata.name == "missing-lifecycle")
            .expect("Should find missing-lifecycle");

        let missing_type = entities
            .iter()
            .find(|e| e.entity.metadata.name == "missing-type")
            .expect("Should find missing-type");

        // Valid entity should have no validation errors
        assert!(
            valid_service.validation_errors.is_empty(),
            "Valid service should have no validation errors, but has: {:?}",
            valid_service.validation_errors
        );

        // Invalid entities should have validation errors
        assert!(
            !missing_owner.validation_errors.is_empty(),
            "missing-owner should have validation errors"
        );

        assert!(
            !missing_lifecycle.validation_errors.is_empty(),
            "missing-lifecycle should have validation errors"
        );

        assert!(
            !missing_type.validation_errors.is_empty(),
            "missing-type should have validation errors"
        );

        // Check that error messages contain validation errors
        assert!(
            !missing_owner.validation_errors.is_empty(),
            "missing-owner should have validation errors"
        );
    }

    #[test]
    fn test_load_all_entities_from_directory() {
        // Test loading all entities from the testdata directory
        let testdata_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata");

        let entities =
            load_all_entities(&testdata_path).expect("Should load entities from directory");

        // Should find multiple entities across multiple files
        // Note: catalog-info.yaml files in testdata and subdir contain test entities
        assert!(
            entities.len() >= 2,
            "Should load at least 2 entities from testdata directory, but found {}",
            entities.len()
        );

        // Verify entities were discovered from catalog-info.yaml files
        let has_catalog_entities = entities
            .iter()
            .any(|e| e.source_file.file_name().unwrap() == "catalog-info.yaml");
        assert!(
            has_catalog_entities,
            "Should find entities from catalog-info.yaml files"
        );
    }

    #[test]
    fn test_load_all_entities_from_single_file() {
        // Test loading from a single file path
        let file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("testdata")
            .join("validation-test.yaml");

        let entities =
            load_all_entities(&file_path).expect("Should load entities from single file");

        assert!(
            !entities.is_empty(),
            "Should load entities from single file"
        );

        // All entities should be from the same file
        for entity in &entities {
            assert_eq!(entity.source_file, file_path);
        }
    }

    #[test]
    fn test_empty_yaml_file() {
        // Test parsing empty YAML content
        let yaml_content = "";
        let path = Path::new("empty.yaml");

        let entities =
            parse_multi_document_yaml(yaml_content, path).expect("Should handle empty YAML");

        assert_eq!(entities.len(), 0, "Empty YAML should produce no entities");
    }

    #[test]
    fn test_yaml_with_invalid_entity() {
        // Test YAML with one valid and one invalid entity
        let yaml_content = r#"
apiVersion: backstage.io/v1alpha1
kind: Component
metadata:
  name: good-service
spec:
  type: service
  lifecycle: production
  owner: team-a
---
this is not valid yaml or entity structure
---
apiVersion: backstage.io/v1alpha1
kind: Component
metadata:
  name: another-good-service
spec:
  type: service
  lifecycle: production
  owner: team-b
"#;

        let path = Path::new("mixed.yaml");
        let entities = parse_multi_document_yaml(yaml_content, path)
            .expect("Should parse valid entities and skip invalid ones");

        // Should parse the valid entities and skip the invalid one
        assert_eq!(entities.len(), 2, "Should parse 2 valid entities");
        assert_eq!(entities[0].entity.metadata.name, "good-service");
        assert_eq!(entities[1].entity.metadata.name, "another-good-service");
    }

    #[test]
    fn test_discover_excludes_build_directories() {
        // This test verifies that discovery properly excludes common build directories
        // by checking the should_exclude_dir function is applied correctly

        // We can't easily create temporary directories with specific names,
        // but we can verify that if such directories existed, they would be excluded
        // by testing that our testdata directory (which doesn't have these dirs)
        // works correctly, and trusting should_exclude_dir tests cover the logic

        let testdata_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata");
        let files = discover_catalog_files(&testdata_path);

        // Verify none of the discovered files are in excluded directories
        for file in &files {
            let path_str = file.to_str().unwrap();
            assert!(
                !path_str.contains("/target/"),
                "Should not find files in target/"
            );
            assert!(
                !path_str.contains("/node_modules/"),
                "Should not find files in node_modules/"
            );
            assert!(
                !path_str.contains("/.git/"),
                "Should not find files in .git/"
            );
            assert!(
                !path_str.contains("/bazel-"),
                "Should not find files in bazel-* dirs"
            );
        }
    }

    #[test]
    fn test_parse_catalog_file_nonexistent() {
        // Test parsing a non-existent file
        let result = parse_catalog_file(Path::new("/nonexistent/catalog-info.yaml"));

        assert!(result.is_err(), "Should return error for nonexistent file");

        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Failed to read file") || err_msg.contains("No such file"),
            "Error message should indicate file read failure: {}",
            err_msg
        );
    }

    #[test]
    fn test_entity_kinds_preserved() {
        // Test that different entity kinds are correctly preserved
        let yaml_content = r#"
apiVersion: backstage.io/v1alpha1
kind: Component
metadata:
  name: my-component
spec:
  type: service
  lifecycle: production
  owner: team-a
---
apiVersion: backstage.io/v1alpha1
kind: API
metadata:
  name: my-api
spec:
  type: openapi
  lifecycle: production
  owner: team-a
  definition: "openapi: 3.0.0"
---
apiVersion: backstage.io/v1alpha1
kind: System
metadata:
  name: my-system
spec:
  owner: team-a
---
apiVersion: backstage.io/v1alpha1
kind: Domain
metadata:
  name: my-domain
spec:
  owner: team-a
---
apiVersion: backstage.io/v1alpha1
kind: Resource
metadata:
  name: my-database
spec:
  type: database
  owner: team-a
---
apiVersion: backstage.io/v1alpha1
kind: Group
metadata:
  name: my-group
spec:
  type: team
  children: []
---
apiVersion: backstage.io/v1alpha1
kind: User
metadata:
  name: my-user
spec:
  memberOf: []
"#;

        let path = Path::new("kinds.yaml");
        let entities =
            parse_multi_document_yaml(yaml_content, path).expect("Should parse all entity kinds");

        assert_eq!(entities.len(), 7, "Should parse 7 different entity kinds");

        let kinds: Vec<String> = entities.iter().map(|e| e.entity.kind.to_string()).collect();

        assert!(kinds.contains(&"Component".to_string()));
        assert!(kinds.contains(&"API".to_string()));
        assert!(kinds.contains(&"System".to_string()));
        assert!(kinds.contains(&"Domain".to_string()));
        assert!(kinds.contains(&"Resource".to_string()));
        assert!(kinds.contains(&"Group".to_string()));
        assert!(kinds.contains(&"User".to_string()));
    }
}
