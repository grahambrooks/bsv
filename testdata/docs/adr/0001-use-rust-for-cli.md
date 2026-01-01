# ADR 0001: Use Rust for CLI Development

## Status

Accepted

## Context

We need to build a terminal UI application for visualizing Backstage catalog entities. The application needs to:

- Parse YAML files efficiently
- Render a responsive terminal interface
- Handle keyboard navigation smoothly
- Work cross-platform (Linux, macOS, Windows)

## Decision

We will use **Rust** as the primary language with the following libraries:

- `ratatui` for terminal UI rendering
- `crossterm` for cross-platform terminal handling
- `serde` + `serde_yaml` for YAML parsing
- `walkdir` for file discovery

## Consequences

### Positive

- Excellent performance for file parsing and UI rendering
- Strong type system catches errors at compile time
- Cross-platform support out of the box
- Memory safety without garbage collection
- Rich ecosystem for CLI tools

### Negative

- Steeper learning curve compared to Python or JavaScript
- Longer compile times during development
- Smaller talent pool for maintenance

## References

- [Ratatui Documentation](https://ratatui.rs/)
- [The Rust Programming Language](https://doc.rust-lang.org/book/)
