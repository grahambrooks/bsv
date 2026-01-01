# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
