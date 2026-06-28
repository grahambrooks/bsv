//! bsv — Backstage Entity Visualizer.
//!
//! A library and terminal UI for parsing, validating, and exploring
//! [Backstage](https://backstage.io/) software catalog entities. The binary in
//! `main.rs` is a thin shell over these modules; exposing them as a library also
//! lets the documentation examples and interaction logic be unit-tested.
//!
//! # Modules
//!
//! - [`entity`] — entity models, reference parsing, and the lookup index
//! - [`parser`] — load and validate `catalog-info.yaml` files from disk
//! - [`validator`] — JSON Schema validation of entities
//! - [`tree`] — hierarchical tree built from a flat entity list
//! - [`graph`] — relationship graph between entities
//! - [`docs`] — TechDocs / ADR documentation discovery and browsing
//! - [`app`] — application state and interaction logic
//! - [`ui`] — ratatui rendering

pub mod app;
pub mod cli;
pub mod docs;
pub mod entity;
pub mod graph;
pub mod parser;
pub mod report;
pub mod tree;
pub mod ui;
pub mod validator;
