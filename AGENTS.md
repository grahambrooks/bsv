# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

bsv (Backstage Entity Visualizer) is a Rust terminal UI application that discovers and visualizes Backstage entities from `catalog-info.yaml` files. It recursively scans directories, parses multi-document YAML files, and displays entities in an interactive hierarchical tree view.

## Build Commands

```bash
cargo build           # Build debug
cargo build --release # Build release
cargo run             # Run with current directory
cargo run /path       # Run with specific directory
cargo check           # Type check without building
cargo clippy          # Lint
cargo fmt             # Format code
```

## Architecture

The application follows a standard TUI architecture with separation between data, state, and presentation:

- **entity.rs** - Backstage entity model (`Entity`, `EntityKind`, `Metadata`). Supports all 8 standard Backstage types: Component, API, Resource, System, Domain, Group, User, Location.

- **parser.rs** - File discovery using `walkdir` to find `catalog-info.yaml`/`.yml` files, and multi-document YAML parsing with `serde_yaml::Deserializer`.

- **tree.rs** - Builds hierarchical `EntityTree` from flat entity list. Groups entities by Domain → System → Components/APIs. `TreeState` tracks selection and expanded nodes.

- **app.rs** - Application state (`App`) combining tree, tree state, and quit flag. Handles navigation commands (up/down/expand/collapse).

- **ui.rs** - Ratatui rendering with two-panel layout: entity tree (left) and details panel (right).

- **main.rs** - Terminal setup/teardown with crossterm, event loop processing keyboard input.

## Key Patterns

- Entities are wrapped in `EntityWithSource` to track their source file path
- Tree nodes use numeric IDs with parent-child relationships stored as `Vec<usize>`
- Navigation works on visible nodes only (respects collapsed state)
- Entity relationships are inferred from `spec.system` and `spec.domain` fields
