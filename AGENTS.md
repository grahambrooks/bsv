# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

bsv (Backstage Entity Visualizer) is a Rust terminal UI application that discovers and visualizes Backstage entities from `catalog-info.yaml` files. It recursively scans directories, parses multi-document YAML files, and displays entities in an interactive hierarchical tree view with relationship visualization.

## Build Commands

```bash
cargo build           # Build debug
cargo build --release # Build release
cargo run             # Run with current directory
cargo run /path       # Run with specific directory or file
cargo test            # Run tests
cargo clippy          # Lint
cargo fmt             # Format code
```

## Architecture

The application follows a standard TUI architecture with separation between data, state, and presentation:

- **entity.rs** - Backstage entity model (`Entity`, `EntityKind`, `Metadata`), entity reference parsing (`EntityRef`), and reference validation (`EntityIndex`). Supports all 8 standard Backstage types: Component, API, Resource, System, Domain, Group, User, Location.

- **parser.rs** - File discovery using `walkdir` to find `catalog-info.yaml`/`.yml` files, and multi-document YAML parsing with `serde_yaml::Deserializer`. Supports both directory scanning and single file loading.

- **tree.rs** - Builds hierarchical `EntityTree` from flat entity list. Groups entities by Domain → System → Components/APIs. `TreeState` tracks selection and expanded nodes.

- **graph.rs** - Extracts relationship graphs for entities. `RelationshipGraph` contains outgoing relationships (owner, system, domain, dependencies, APIs) and incoming relationships (entities that reference this one).

- **docs.rs** - Documentation browser for TechDocs and ADR markdown files. Parses `backstage.io/techdocs-ref` and `backstage.io/adr-location` annotations. Provides file discovery, markdown rendering, and scrollable viewing.

- **app.rs** - Application state (`App`) combining tree, tree state, entity index, docs browser, and UI mode. Handles navigation, search, graph toggle, docs browsing, and reload.

- **ui.rs** - Ratatui rendering with two-panel layout: entity tree with search bar (left) and details/graph panel (right). Includes reference validation visualization.

- **main.rs** - Terminal setup/teardown with crossterm, event loop processing keyboard input for both normal and search modes.

## Key Patterns

- Entities are wrapped in `EntityWithSource` to track their source file path
- Tree nodes use numeric IDs with parent-child relationships stored as `Vec<usize>`
- Navigation works on visible nodes only (respects collapsed state and search filter)
- Entity relationships are inferred from spec fields (system, domain, owner, dependsOn, providesApis, consumesApis, memberOf)
- `EntityRef` parses Backstage reference format `[<kind>:][<namespace>/]<name>` with context-aware defaults
- `EntityIndex` provides O(1) lookup for reference validation

## UI Modes

- **Normal mode**: Tree navigation, expand/collapse, toggle views
- **Search mode**: Filter entities by name (activated with `/`)
- **Graph view**: Show relationships instead of details (toggle with `g`)
- **Docs browser**: Browse TechDocs/ADR markdown files (activated with `d` when annotations present)
