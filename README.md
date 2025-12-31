# bsv - Backstage Entity Visualizer

A terminal UI application for exploring and visualizing [Backstage](https://backstage.io/) software catalog entities.

![bsv demo](https://github.com/user-attachments/assets/placeholder.png)

## Features

- **Tree View**: Hierarchical visualization of entities organized by Domain → System → Component
- **Entity Details**: View metadata, ownership, lifecycle, tags, and source file information
- **Relationship Graph**: Visualize how entities relate to each other (dependencies, APIs, ownership)
- **Reference Validation**: Highlights missing or invalid entity references
- **Search**: Filter entities by name with `/` search
- **Live Reload**: Refresh catalog data without restarting

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/yourusername/bsv.git
cd bsv

# Build and install
cargo install --path .
```

### Requirements

- Rust 1.70 or later

## Usage

```bash
# Scan current directory for catalog-info.yaml files
bsv

# Scan a specific directory
bsv /path/to/backstage/catalog

# Load a specific catalog file
bsv /path/to/catalog-info.yaml
```

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `↑` / `k` | Move up |
| `↓` / `j` | Move down |
| `←` / `h` | Collapse node |
| `→` / `l` / `Enter` | Expand node |
| `e` | Expand all nodes |
| `/` | Start search |
| `Esc` | Clear search / Cancel |
| `g` | Toggle graph view |
| `r` | Reload catalog |
| `q` | Quit |

## Supported Entity Types

bsv supports all standard Backstage entity types:

- **Component** - Individual software components (services, websites, libraries)
- **API** - API definitions (REST, GraphQL, gRPC)
- **Resource** - Infrastructure resources (databases, storage, queues)
- **System** - Collection of related components and APIs
- **Domain** - Business domain grouping systems
- **Group** - Teams and organizational units
- **User** - Individual team members
- **Location** - Catalog file locations

## Entity Reference Format

Backstage uses a specific format for entity references:

```
[<kind>:][<namespace>/]<name>
```

Examples:
- `component:default/my-service` - Fully qualified
- `my-service` - Name only (kind and namespace inferred from context)
- `group:platform-team` - Kind specified, namespace inferred

bsv displays inferred parts in `[brackets]` with dim styling to distinguish them from explicitly specified values.

## Reference Validation

bsv validates entity references and provides visual feedback:

- **Green**: Reference exists in catalog
- **Yellow**: Reference not found (might be external or missing)
- **Red**: Unknown entity kind

## Catalog File Format

bsv reads standard Backstage `catalog-info.yaml` files:

```yaml
apiVersion: backstage.io/v1alpha1
kind: Component
metadata:
  name: my-service
  title: My Service
  description: A microservice that does things
  tags:
    - python
    - backend
spec:
  type: service
  lifecycle: production
  owner: group:platform-team
  system: my-system
  dependsOn:
    - component:other-service
  providesApis:
    - my-api
```

Multiple entities can be defined in a single file using YAML document separators (`---`).

## Development

```bash
# Run in development mode
cargo run

# Run with a test catalog
cargo run -- testdata/large-catalog.yaml

# Run tests
cargo test

# Check for linting issues
cargo clippy

# Format code
cargo fmt
```

## Project Structure

```
src/
├── main.rs      # Entry point and event loop
├── app.rs       # Application state management
├── entity.rs    # Entity types and reference parsing
├── parser.rs    # YAML file discovery and parsing
├── tree.rs      # Tree data structure
├── graph.rs     # Relationship graph extraction
└── ui.rs        # Terminal UI rendering
```

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.
