# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2026-02-12] - 2026-02-12

### Added
- **JSON Schema Validation**: Automatically validates entities against official Backstage schema during parsing
- **Group Hierarchy Display**: Comprehensive visualization of group parent/child relationships and member lists
- **Test Suite**: 46 new tests across entity, parser, graph, and tree modules (2 → 48 tests, ~75% coverage)
- **Module Documentation**: Comprehensive docs with 18 code examples for all public modules
- **UI Submodules**: Split large ui.rs into 7 focused modules (tree, details, graph, docs, theme, help)

### Changed
- **Refactored Graph Module**: Reduced long functions by 71% (290 → 84 lines) using helper functions
- **Refactored Event Handling**: Reduced main.rs event loop by 74% (107 → 28 lines) with InputMode enum
- **Code Quality**: Applied 27 clippy pedantic fixes for more idiomatic Rust patterns
- **Module Organization**: Better separation of concerns with focused, maintainable modules

### Fixed
- JSON Schema API compatibility with jsonschema 0.41
- Build warnings by eliminating code duplication in search functionality
- All clippy pedantic warnings (40 → 0)

### Improved
- **Module Health**: Grade improved from B+ to A
- **Test Coverage**: Increased from ~5% to ~75%
- **Maintainability**: Eliminated all 5 functions over 100 lines
- **Documentation**: Zero doc warnings, all examples compile
- **Code Quality**: Zero build/clippy/doc warnings

## [Unreleased]

### Added

- Links display in entity details panel
- Annotations display with special highlighting for documentation-related annotations
- Documentation browser for TechDocs and ADR markdown files
  - Support for `backstage.io/techdocs-ref` annotation
  - Support for `backstage.io/adr-location` annotation
  - Basic markdown syntax highlighting
  - Scrollable document viewing
- Press `d` to open documentation browser when docs annotations are present

## [0.1.0] - 2024-01-01

### Added

- Initial release
- Tree view for browsing entities organized by Domain → System → Component
- Details panel showing entity metadata, ownership, and source file
- Relationship graph view showing entity connections
- Entity reference validation with visual feedback for missing references
- Search functionality to filter entities by name
- Support for all standard Backstage entity types:
  - Component, API, Resource, System, Domain, Group, User, Location
- Multi-document YAML parsing for catalog files
- Recursive directory scanning for catalog-info.yaml files
- Keyboard navigation with vim-style bindings
- Live reload capability
