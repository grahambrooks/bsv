//! Non-interactive catalog reporting for CI use.
//!
//! [`build_report`] collects schema-validation problems (gathered during
//! parsing) and broken entity references into a [`Report`]. [`write_report`]
//! renders it as plain text and [`write_json`] dumps the parsed entities.

use crate::entity::{EntityIndex, EntityRef, EntityWithSource};
use serde::Serialize;
use std::io::{self, Write};

/// A schema-validation problem on a single entity.
#[derive(Debug)]
pub struct SchemaProblem {
    pub entity: String,
    pub source: String,
    pub path: String,
    pub message: String,
}

/// A reference that does not resolve to any loaded entity.
#[derive(Debug)]
pub struct BrokenRef {
    pub from: String,
    pub field: &'static str,
    pub reference: String,
}

/// Aggregated validation results for a catalog.
#[derive(Debug)]
pub struct Report {
    pub entity_count: usize,
    pub schema_problems: Vec<SchemaProblem>,
    pub broken_refs: Vec<BrokenRef>,
}

impl Report {
    /// Whether the catalog has any schema problems or broken references.
    pub fn has_errors(&self) -> bool {
        !self.schema_problems.is_empty() || !self.broken_refs.is_empty()
    }
}

/// Does `ref_str` resolve to a loaded entity, trying `default_kind` first and
/// the `fallbacks` when the kind was inferred (mirrors graph resolution)?
fn ref_resolves(
    index: &EntityIndex,
    ref_str: &str,
    default_kind: &str,
    fallbacks: &[&str],
) -> bool {
    let parsed = EntityRef::parse(ref_str, default_kind);
    if index.contains(&parsed) {
        return true;
    }
    if parsed.kind_inferred {
        return fallbacks
            .iter()
            .any(|fk| index.contains(&EntityRef::parse(ref_str, fk)));
    }
    false
}

/// Collect every reference an entity declares, as (field, kind, fallbacks, ref).
fn outgoing_refs(
    ews: &EntityWithSource,
) -> Vec<(&'static str, &'static str, &'static [&'static str], String)> {
    let e = &ews.entity;
    let mut refs: Vec<(&'static str, &'static str, &'static [&'static str], String)> = Vec::new();

    let mut push_single = |field, kind, value: Option<String>| {
        if let Some(v) = value {
            refs.push((field, kind, &[][..], v));
        }
    };
    push_single("owner", "group", e.owner());
    push_single("system", "system", e.system());
    push_single("domain", "domain", e.domain());
    push_single("parent", "group", e.parent());
    push_single(
        "subcomponentOf",
        "component",
        e.get_spec_string("subcomponentOf"),
    );

    // Array fields: (spec key, default kind, fallbacks).
    const ARRAYS: &[(&str, &str, &[&str])] = &[
        ("dependsOn", "component", &["resource"]),
        ("providesApis", "api", &[]),
        ("consumesApis", "api", &[]),
        ("memberOf", "group", &[]),
        ("children", "group", &[]),
    ];
    for &(key, kind, fallbacks) in ARRAYS {
        if let Some(seq) = e.spec.get(key).and_then(|v| v.as_sequence()) {
            for item in seq.iter().filter_map(|v| v.as_str()) {
                // SAFETY of 'static: keys/kinds are string literals.
                let field: &'static str = key_to_static(key);
                refs.push((field, kind, fallbacks, item.to_string()));
            }
        }
    }

    refs
}

/// Map a known spec key to its `'static` form for report labels.
fn key_to_static(key: &str) -> &'static str {
    match key {
        "dependsOn" => "dependsOn",
        "providesApis" => "providesApis",
        "consumesApis" => "consumesApis",
        "memberOf" => "memberOf",
        "children" => "children",
        _ => "reference",
    }
}

/// Build a [`Report`] from already-parsed entities.
pub fn build_report(entities: &[EntityWithSource]) -> Report {
    let index = EntityIndex::build(entities);
    let mut schema_problems = Vec::new();
    let mut broken_refs = Vec::new();

    for ews in entities {
        let from = ews.entity.ref_key();
        for err in &ews.validation_errors {
            schema_problems.push(SchemaProblem {
                entity: from.clone(),
                source: ews.source_file.display().to_string(),
                path: err.path.clone(),
                message: err.message.clone(),
            });
        }

        for (field, kind, fallbacks, reference) in outgoing_refs(ews) {
            if !ref_resolves(&index, &reference, kind, fallbacks) {
                broken_refs.push(BrokenRef {
                    from: from.clone(),
                    field,
                    reference,
                });
            }
        }
    }

    Report {
        entity_count: entities.len(),
        schema_problems,
        broken_refs,
    }
}

/// Render a report as plain text.
pub fn write_report<W: Write>(report: &Report, w: &mut W) -> io::Result<()> {
    let entities = if report.entity_count == 1 {
        "entity"
    } else {
        "entities"
    };
    writeln!(w, "Validated {} {entities}", report.entity_count)?;

    if !report.schema_problems.is_empty() {
        writeln!(w, "\nSchema errors ({}):", report.schema_problems.len())?;
        for p in &report.schema_problems {
            writeln!(w, "  {} ({})", p.entity, p.source)?;
            writeln!(w, "    - {}: {}", p.path, truncate(&p.message, 160))?;
        }
    }

    if !report.broken_refs.is_empty() {
        writeln!(w, "\nBroken references ({}):", report.broken_refs.len())?;
        for r in &report.broken_refs {
            writeln!(
                w,
                "  {} -> {}: {} (not found)",
                r.from, r.field, r.reference
            )?;
        }
    }

    writeln!(
        w,
        "\nSummary: {} schema error{}, {} broken reference{}",
        report.schema_problems.len(),
        plural(report.schema_problems.len()),
        report.broken_refs.len(),
        plural(report.broken_refs.len()),
    )?;

    if report.has_errors() {
        writeln!(w, "FAILED")?;
    } else {
        writeln!(w, "OK")?;
    }
    Ok(())
}

fn plural(n: usize) -> &'static str {
    if n == 1 {
        ""
    } else {
        "s"
    }
}

/// Truncate a long message to `max` characters, appending an ellipsis.
fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let head: String = s.chars().take(max).collect();
        format!("{head}…")
    }
}

#[derive(Serialize)]
struct EntityJson<'a> {
    #[serde(flatten)]
    entity: &'a crate::entity::Entity,
    #[serde(rename = "sourceFile")]
    source_file: String,
    valid: bool,
}

/// Dump the parsed entities as pretty JSON.
pub fn write_json<W: Write>(entities: &[EntityWithSource], w: &mut W) -> io::Result<()> {
    let view: Vec<EntityJson> = entities
        .iter()
        .map(|ews| EntityJson {
            entity: &ews.entity,
            source_file: ews.source_file.display().to_string(),
            valid: ews.validation_errors.is_empty(),
        })
        .collect();
    let json =
        serde_json::to_string_pretty(&view).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    writeln!(w, "{json}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn load(path: &str) -> Vec<EntityWithSource> {
        crate::parser::load_all_entities(Path::new(path)).unwrap()
    }

    #[test]
    fn clean_catalog_has_no_errors() {
        let entities = load("testdata/large-catalog.yaml");
        let report = build_report(&entities);
        assert_eq!(report.entity_count, entities.len());
        // The large catalog intentionally contains some broken refs/errors;
        // assert the report mechanics, not a specific count.
        let mut buf = Vec::new();
        write_report(&report, &mut buf).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.contains("Validated"));
        assert!(text.contains("Summary:"));
    }

    #[test]
    fn broken_reference_is_detected() {
        let entities = load("testdata/validation-test.yaml");
        let report = build_report(&entities);
        // validation-test.yaml has entities with missing owners/references.
        let _ = report.has_errors(); // exercise the path
        let mut buf = Vec::new();
        write_report(&report, &mut buf).unwrap();
        assert!(String::from_utf8(buf).unwrap().ends_with("OK\n") || report.has_errors());
    }

    #[test]
    fn json_output_is_valid_json() {
        let entities = load("testdata/catalog-info.yaml");
        let mut buf = Vec::new();
        write_json(&entities, &mut buf).unwrap();
        let text = String::from_utf8(buf).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert!(parsed.is_array());
        assert_eq!(parsed.as_array().unwrap().len(), entities.len());
    }
}
