# ADR 0002: Entity Tree Structure

## Status

Accepted

## Context

Backstage entities have complex relationships:

- Domains contain Systems
- Systems contain Components and APIs
- Components can depend on other Components
- Components can provide/consume APIs
- Groups own entities
- Users belong to Groups

We need to display these in a navigable tree structure.

## Decision

Organize the tree as follows:

```
Root
├── Domains
│   └── [domain-name]
│       └── Systems in this domain
├── Systems (ungrouped)
│   └── [system-name]
│       ├── Components
│       └── APIs
├── Components (orphaned)
├── APIs (orphaned)
├── Resources
├── Groups
│   └── [group-name]
│       └── Child groups
└── Users
```

## Consequences

### Positive

- Intuitive hierarchy matching Backstage concepts
- Easy to find entities by domain/system
- Orphaned entities are still visible

### Negative

- Some entities appear in multiple logical places
- Deep nesting can be hard to navigate
- Requires expand/collapse state management
