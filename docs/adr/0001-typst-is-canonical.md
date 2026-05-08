# ADR 0001 — Typst Is Canonical

## Status

Accepted

## Context

The site requires:
- semantic HTML
- unified PDF output
- minimal architecture
- plain-text authoring
- strong document semantics

There are several possible approaches:
- custom markdown pipeline
- HTML-first generation
- Typst-first generation
- AST-driven transformation systems

## Decision

Typst is the canonical source of truth.

All authored content exists primarily as `.typ` files.

Typst owns:
- headings
- anchors
- imports
- PDF rendering
- document semantics
- HTML generation

The site generator does not parse Typst source.

## Consequences

Positive:
- one canonical source format
- PDF generation remains first-class
- semantic document structure comes from Typst directly
- reduced metadata/config duplication

Negative:
- Typst HTML limitations constrain downstream processing
- HTML generation depends on Typst capabilities
- some advanced behaviors may require Typst macro work
