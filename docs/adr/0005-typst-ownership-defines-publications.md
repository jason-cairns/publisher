# ADR 0005 - Typst Ownership Defines Publications

## Status

Accepted. The "unreachable non-helper Typst files become build errors"
consequence below is superseded by ADR 0006: unreachable candidates are
dropped silently.

## Context

ADR 0003 used filesystem structure to discover publications, derive routes, and build navigation.

That kept the first implementation small, but it made publication ownership implicit. A file linked from a section could accidentally look like a child of that section, and moving files could change publishing semantics.

The system needs:
- explicit publication ownership
- Typst as the semantic source of truth
- no YAML, frontmatter, or route registry
- no Typst source parsing in Prolog
- deterministic navigation inheritance

## Decision

Typst-authored ownership edges define the publication structure.

The site has a root publication, normally `src/index.typ`. The published set is the rooted ordered ownership tree reachable from that root.

Typst exposes three publication relationships:

- `#nav(target)[Label]` creates an ownership edge and a visible ordered navigation item.
- `#publish(target)` creates an ownership edge without adding a visible navigation item.
- `#publication-link(target)[Label]` creates a reference edge only.

The ownership graph has these constraints:

- it must be acyclic
- every published page except the root has exactly one owner
- sibling navigation order follows source order
- ordinary reference links may be cyclic
- reference links must point to publications already in the ownership tree

Navigation context is derived from the unique ownership path from the root to a publication.

Routes still derive from source paths. Filesystem paths map owned publications to URLs, but filesystem structure does not decide whether a file is published.

Prolog operates only on Typst-generated HTML. Typst helpers emit semantic HTML markers for ownership and reference edges; Prolog reads those markers from intermediate HTML and never parses Typst source.

## Consequences

Positive:
- publication structure is explicit and authored in Typst
- navigation inheritance has a single mathematical source: the ownership path
- ordinary links do not imply ownership
- directories can organize source files without owning publication semantics
- unreachable non-helper Typst files become build errors

Negative:
- authors must declare publication ownership explicitly
- builds need graph validation
- a page cannot be owned by two different sections unless rendered as two distinct publications later
