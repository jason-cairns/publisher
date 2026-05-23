= PR 57 fix 01

This plan addresses the review comments on PR #57 without implementing the
follow-up work in the plan commit itself.

== Review goals

- Preserve file-level parse warnings so unsupported `publisher.*` calls are
  visible in validation and inspect output.
- Keep the parser strict about the publisher namespace for this milestone.
- Move nav suppression from node-owned property state to navigation projection
  state.
- Represent inherited nav behavior as projections on covered child nodes.
- Move publisher call and projection-building logic out of `parser.rs`.
- Add a named scope example to the API sketch fixture.
- Replace the stale root Python CI workflow with Rust checks.
- Capture new model concepts in docs instead of letting implementation names
  quietly become domain language.

== Model decisions

=== Authored values

`Authored<T>` is a funny new concept in the current implementation. It should
not silently become publication-domain terminology.

The concept it is trying to represent is source provenance for model fields:
some values come from explicit Typst source, and some values are parser or model
defaults. That distinction is useful for inspect and debug output, but the word
`authored` can be confused with publication authorship.

The implementation should either rename this toward provenance language or
document it explicitly as "source-provided versus defaulted field provenance".
`docs/model-map.typ` should not present `Authored<T>` as a core model entity
without that discussion.

=== Navigation scopes and projections

Navigation is a projection, not a node property.

Declaring a nav scope means the scope root and all covered child nodes inherit a
navigation projection connected to that nav scope root. A child covered by
multiple nav scopes may therefore have multiple nav projections, matching the
stacked-navigation model.

`publisher.nav.suppress()` suppresses the nav projection on the page where it
appears. Suppression should be represented on the affected navigation projection,
not as a `nav.suppressed` property owned by the node.

This keeps navigation behavior close to the projection model:

- scopes describe context over nodes;
- queries describe what a projection selects;
- projections describe generated output and output-specific state;
- nodes do not carry nav-output state as general-purpose properties.

== Implementation slices

=== Preserve parse warnings

Copy each reachable `ParsedFile.warnings` entry into
`Publication.parse_warnings` during publication assembly. Preserve deterministic
ordering by adding reachable-file warnings in publication traversal order before
unreachable-file warnings.

Add regression coverage for an unsupported `publisher.*` call.

=== Parser modularity

Keep filesystem traversal, reachability, Typst AST walking, and publication
assembly in `parser.rs`.

Move call-building and projection-building logic for outline, bibliography,
reference, nav suppression, and inherited nav projections into one or more
separate parser module files. Use `pub(super)` boundaries where needed; avoid
turning parser internals into public API.

=== Nav projection inheritance

When an explicit nav scope is declared, create navigation projections for the
scope root and covered child nodes. Each projection should stay connected to the
nav scope root through its query or rendering metadata, so inspect output makes
the inherited source clear.

If a node is covered by multiple nav scopes, preserve stacked navigation by
representing multiple navigation projections.

If `publisher.nav.suppress()` appears on a page, suppress the navigation
projection for that page only. Do not add a node-owned `nav.suppressed`
property.

=== Publisher namespace restriction

Only `publisher.nav.suppress()` should trigger nav suppression in this
milestone. Bare `nav.suppress()` should not create properties, projections, or
warnings.

=== Named scopes

Support `publisher.scope(kind: "outline", name: "...")`.

Add a named outline scope to one writing source fixture. The scope id should
remain based on kind and root node; the authored name is metadata used for
queries and inspect output.

=== Docs and CI

Update `docs/model.typ` for projection-owned nav suppression and inherited nav
projections.

Update `docs/model-map.typ` so the authored/defaulted field distinction is
explained as implementation provenance rather than a new unexplained model
entity.

Update `docs/api-sketch.typ` so examples do not imply bare `nav.suppress()` is
supported by this milestone parser.

Replace root Python CI with Rust CI:

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo fmt --check
      - run: cargo test --locked
      - run: cargo run --example api_sketch
```

== Test plan

- Unsupported `publisher.*` calls reach validation warnings.
- Bare `nav.suppress()` is ignored.
- `publisher.nav.suppress()` suppresses a navigation projection only.
- Nav scopes create inherited navigation projections for covered children.
- Multiple nav scopes create stacked navigation projections.
- Named scopes preserve authored names without changing stable ids.
- Inspect snapshots no longer show node-owned `nav.suppressed` properties.

Run:

```sh
cargo fmt --check
cargo test --locked
cargo run --example api_sketch
git diff --check
```

If inspect snapshots change:

```sh
INSTA_UPDATE=always cargo test --test inspect_api_sketch
cargo test --test inspect_api_sketch
cargo test --locked
```
