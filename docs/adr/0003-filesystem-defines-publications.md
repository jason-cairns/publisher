# ADR 0003 — Filesystem Defines Publications And Routes

## Status

Accepted

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

Top-level publications form the main navigation.

Subdirectory publications do not automatically appear in global navigation.

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
