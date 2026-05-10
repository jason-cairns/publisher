# ADR 0003 — Filesystem Defines Publications And Routes

## Status

Superseded by ADR 0005.

## Context

The system requires:
- minimal configuration
- stable routing
- semantic publishing
- low cognitive overhead

Several models were considered:
- explicit route declarations
- metadata frontmatter
- filesystem-derived routing

## Decision

Filesystem structure defines publication structure and routing.

Rules:
- every non-underscore `.typ` file is a publication
- `_*.typ` files are helpers/import-only
- routes derive directly from paths

Examples:

```text
about.typ      -> /about/
thesis/ch1.typ -> /thesis/ch1/
```

In this superseded model, top-level publications formed the main navigation.

Subdirectory publications did not automatically appear in global navigation.

## Consequences

Positive:
- no route metadata layer
- no frontmatter
- trivial publication discovery
- deterministic routing
- tiny implementation

Negative:
- routes tied to filesystem layout
- moving files changes URLs
