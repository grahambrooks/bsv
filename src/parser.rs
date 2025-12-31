use crate::entity::{Entity, EntityWithSource};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

pub fn discover_catalog_files(root: &Path) -> Vec<std::path::PathBuf> {
    WalkDir::new(root)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.file_name()
                .to_str()
                .map(|name| name == "catalog-info.yaml" || name == "catalog-info.yml")
                .unwrap_or(false)
        })
        .map(|e| e.into_path())
        .collect()
}

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
                entities.push(EntityWithSource {
                    entity,
                    source_file: source_path.to_path_buf(),
                });
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

pub fn load_all_entities(root: &Path) -> Result<Vec<EntityWithSource>> {
    let catalog_files = discover_catalog_files(root);
    let mut all_entities = Vec::new();

    for file_path in catalog_files {
        match parse_catalog_file(&file_path) {
            Ok(entities) => all_entities.extend(entities),
            Err(e) => eprintln!("Warning: {}", e),
        }
    }

    Ok(all_entities)
}

use serde::Deserialize;
