# ADR 0006 — Ownership Tree Defines The Rendered Set

## Status

Accepted. Supersedes one consequence of ADR 0005 (see below).

## Context

ADR 0005 made Typst-authored ownership the publication structure, with the
positive consequence:

> unreachable non-helper Typst files become build errors

In practice, the underscore-prefix convention is just a discovery hint. A
file that lives in `src/` without a leading `_` is not necessarily a
publication — it might be an unfinished draft, a sketch, a reference
fragment, or a file the author hasn't yet wired into the ownership tree.
Forcing those into build errors creates friction without protecting any
real invariant: the ownership graph alone decides what is published.

The system needs:

- one rule for publication membership, not two
- forgiving treatment of files that haven't been wired up yet
- continued strict checking of ownership edges that *are* in the graph

## Decision

The ownership tree alone defines the **rendered set** `R ⊆ C`, where `C`
is the set of discovered publication candidates. `R` is the set of
candidates reachable from the root publication via ownership edges
restricted to candidates.

Files in `C \ R` are **dropped silently**. They are not published, do not
appear in routes, do not contribute to the unified PDF, and produce no
build error.

The validation rules from ADR 0005 — acyclicity, single-owner for
non-root, root unowned, and reference closure — apply to the rendered set
and to edges originating in it, not to the full discovered candidate set.

Concretely:

- ownership edges with owner ∈ R must have target ∈ C (else: build error)
- reference edges with source ∈ R must have target ∈ R (else: build
  error)
- label references with source ∈ R must target labels defined in R (else:
  build error)
- ownership edges with owner ∉ R are ignored
- reference edges with source ∉ R are ignored
- labels and label references with source ∉ R are ignored

The underscore prefix (`_*.typ`) remains a discovery convention used by
the build to avoid compiling obvious helper files. It is not load-bearing
for publication semantics.

## Consequences

Positive:

- one rule for "is this published": ownership reachability
- drafts and sketches can sit in `src/` without forcing build errors
- typos in ownership edges (`#publish` or `#entry` to a missing target)
  still error cleanly when the author is in the rendered set
- labels in unpublished drafts cannot satisfy references from the rendered
  set or force those drafts into the rendered set
- the underscore convention can be relaxed or extended without changing
  publication semantics

Negative:

- a typo in a discovery-time filter (e.g. an underscore the author didn't
  intend) silently demotes a file from publishing — the author sees no
  page on the site rather than a build error

## Notes

ADR 0005's other consequences — explicit Typst-authored ownership,
single-owner per published page, publication index inheritance via the
ownership path — remain in force.
