# ADR 0003: Entity Reference Validation

## Status

Accepted

## Context

Backstage entity references use the format:

```
[<kind>:][<namespace>/]<name>
```

Examples:
- `component:default/my-service` (fully qualified)
- `my-service` (name only, kind and namespace inferred)
- `group:platform-team` (kind specified, namespace inferred)

References can be invalid if:
1. The referenced entity doesn't exist
2. The kind is not a recognized Backstage type
3. The format is malformed

## Decision

Implement reference validation with visual feedback:

1. **Green** - Reference exists in the catalog
2. **Yellow** - Reference not found (external or missing)
3. **Red** - Unknown entity kind

Display inferred parts in `[brackets]` with dim styling to distinguish from explicit values.

## Consequences

### Positive

- Users can quickly identify broken references
- Clear distinction between explicit and inferred values
- Helps maintain catalog hygiene

### Negative

- Requires building an index of all entities
- May show false positives for external references
- Additional complexity in UI rendering
