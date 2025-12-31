# Contributing to bsv

Thank you for your interest in contributing to bsv!

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/yourusername/bsv.git`
3. Create a branch: `git checkout -b feature/your-feature`
4. Make your changes
5. Run tests and linting: `cargo test && cargo clippy && cargo fmt --check`
6. Commit your changes: `git commit -m "Add your feature"`
7. Push to your fork: `git push origin feature/your-feature`
8. Open a Pull Request

## Development Setup

### Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))

### Building

```bash
cargo build
```

### Running

```bash
# Run with current directory
cargo run

# Run with test data
cargo run -- testdata/large-catalog.yaml
```

### Testing

```bash
cargo test
```

### Linting

```bash
# Check for issues
cargo clippy

# Auto-format code
cargo fmt
```

## Code Style

- Follow standard Rust conventions
- Run `cargo fmt` before committing
- Ensure `cargo clippy` passes without warnings
- Add documentation for public APIs

## Pull Request Guidelines

- Keep PRs focused on a single change
- Include a clear description of what the PR does
- Update documentation if needed
- Add tests for new functionality
- Ensure all CI checks pass

## Reporting Issues

When reporting issues, please include:

- Rust version (`rustc --version`)
- Operating system
- Steps to reproduce
- Expected vs actual behavior
- Sample catalog-info.yaml if relevant

## Feature Requests

Feature requests are welcome! Please open an issue describing:

- The problem you're trying to solve
- Your proposed solution
- Any alternatives you've considered

## Code of Conduct

Please be respectful and constructive in all interactions.
