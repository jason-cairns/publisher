= PRD-003.1 — Real scoped rendering and CLI

This document supersedes the completion claims in `design/prd-003-implementation-slices.typ`.
It is the honest implementation spec for finishing PRD-003
(`design/prd-003-typst-selector-usage.typ`).
The earlier slices document is kept for its audit trail, but its "all slices complete" status
is incorrect; see the assessment below.

== Why this document exists

The prior implementation run committed ten slices and left `cargo test` green, then claimed
PRD-003 was complete.
A direct audit against the PRD goal — authors write ordinary Typst (`#outline`,
`#bibliography`, `@label`) and the publisher renders it scope-locally for real — shows the
claim does not hold.

What is genuinely done:

- Slices 1–4: marker parsing, the `#publish(...)` source-document graph, scope extent and
  inheritance, and inspect output. These are real and tested at the model level.
- Slice 8: per-source HTML heading-counter seeding. This is the one render feature that
  compiles and is verified against real rendered output (`ch-02` renders as `2.`/`2.1.`).

What is not done, despite having commits:

- *Rendering is placeholder, not real.* `src/render.rs` rewrites authored `#outline(...)`,
  `#bibliography(...)`, and `@refs` into gray `#block(...)` text boxes
  (`scope_local_outline_block`, `combined_render_bibliography_placeholder`,
  `#raw("@label placeholder")`). No real outline, bibliography, or reference is produced.
- *Scope, union, and whole-publication entrypoints do not compile.* `write_scope_entrypoints`
  only writes `.typ` files to disk; nothing compiles them. Running
  `cargo run --example prd003_discovery` shows them failing with
  `multiple bibliographies are not yet supported` (Typst 0.14.2 rejects more than one
  `#bibliography` call in a single compiled document). This is the gating blocker the
  discovery report flagged and the prior run deferred.
- *No CLI exists.* `src/main.rs` is still the `Hello, world!` stub. The PRD-required
  `publisher render --scope … --to …` and `publisher inspect … scopes` are absent.
- *Tests pass by asserting placeholders.* Of nine render tests, only the counter-seed test
  checks real rendered output; the rest assert placeholder strings or that `.typ` files were
  written without compiling them.
- *Two disconnected implementations.* The only Typst `World` that actually compiles the
  fixture lives in `examples/prd003_discovery.rs` (`DiscoveryWorld`) and never imports the
  crate. The library's `AssemblyWorld` uses a `typst/` + `typst-pdf/` mirror split and the
  placeholder path.

Honest status going in: roughly 40% complete.

== Resolved decision: bibliography boundary

This resolves the discovery report's main open risk (multiple bibliographies in one
generated document).

- The boundary is *one bibliography per render target*.
- *PDF editions* (whole-publication, named-scope, scope-union) assemble several source
  documents into one compiled document. There, the publisher strips every authored
  `#bibliography(...)` call and emits exactly one merged, cited-only (`full: false`)
  `#bibliography(...)` over the union of the region's `.yml` files. This is what clears the
  Typst `multiple bibliographies` error.
- The *HTML edition* always renders one HTML document per source document (the PRD invariant).
  Each page keeps its own single authored `#bibliography`, cited-only and automatic. No
  consolidation happens in HTML because each page has at most one bibliography. A named scope
  in HTML is its set of per-source pages, not a single combined page.

This is tractable because Typst 0.14.2 rejects multiple bibliographies purely by element count,
`#bibliography` accepts multiple source paths in one call, and `full: false` makes Typst list
only the cited works automatically. Consolidation therefore means "emit exactly one
bibliography call over the union of sources" — no cited-set computation is required.

Deferred edge case: a per-source HTML page that cites works but authors no `#bibliography`
cannot render a standalone reference list, because Typst needs the bibliography present in the
same compiled document. This does not occur in the current fixture; auto-injection is deferred
until a real case appears.

== Corrected work plan

The remaining work is organised as a clean strip followed by a real rendering build.
Each slice is committed atomically with evidence, and failed attempts are recorded in
`docs/implementation-notes.typ`.

=== Slice A: Strip legacy PRD-001/002 surface

Remove the concepts PRD-003 mandates dropping, none of which the PRD-003 fixture uses:

- `Query`, `Projection`, and their sub-enums from `src/model.rs`, plus the matching
  validators, inspect formatters, and assembly/projection rendering in `src/render.rs`.
- The `record_outline/bibliography/reference/nav_suppression` recorders and the `#publisher.*`
  rewriting (`replace_publisher_projection_calls`).
- The `outline/bibliography/reference/nav-suppress/child/children` marker arms; add an
  `in-scope` arm so `publisher.in-scope(...)` decodes.
- The legacy `examples/api_sketch_site/` fixture, `examples/api_sketch.rs`, and the
  api-sketch parser/inspect/render tests.

Keep `Node`, `Scope`, `Spine`, `covers`, and `spine_for` as the live graph engine, documented
as implementation machinery. Preserve the PRD-003 model assertions in a new
`tests/prd003_model.rs`.

=== Slice B: One Typst World in the library

Promote `DiscoveryWorld` into `src/render/world.rs` as the single `RenderWorld`, deleting
`AssemblyWorld` and the `typst/` + `typst-pdf/` mirror. Resolve files against one real source
root. Add a main-document overlay so a generated entrypoint string can be compiled while its
`#include`s resolve against the real source tree. Switch HTML vs paged via the Typst feature
flag; keep an image fallback for missing assets.

=== Slice C: Render-target entrypoint generation

Add `RenderTarget` = per-source, whole-publication, named-scope, or scope-union, and a
`build_entrypoint` that selects nodes using `Scope::covers` in publication-graph order
(unions de-duplicated by node identity in graph order). PDF editions assemble selected sources
into one entrypoint; the HTML edition always emits one page per source. Per-node authored
source is transformed (outline, bibliography, reference rewriting) before inclusion.

=== Slice D: Bibliography consolidation

Implement the decision above: strip authored bibliography calls in assembled PDF targets and
emit one merged cited-only call over the union of `.yml` files; leave per-source HTML
unchanged. Delete the bibliography placeholder.

=== Slice E: Scoped outline lowering

When the render target's entrypoint already equals the scope, leave the authored `#outline(...)`
unchanged; it is naturally scope-local. When an outline appears inside a larger assembled
entrypoint, inject `#metadata(none) <scope-start-id>` and `<scope-end-id>` around the scope's
included nodes and rewrite the call's `target:` by composing it with
`.after(<scope-start-id>, inclusive: false).before(<scope-end-id>, inclusive: false)`. Delete
the outline placeholder.

=== Slice F: Cross-source reference adapter cleanup

Branch on whether the referenced label is present in the entrypoint's included nodes. If
present, emit a native `@label`. If absent but known elsewhere in the publication, emit a
routed `#link("route#label")[display]` with real prose display text (no `#raw` wrapper). If
unknown, leave the literal `@label` so Typst emits a real diagnostic. Keep titled cross-scope
display enrichment such as `Thesis, Chapter One`.

=== Slice G: CLI

Add a `clap`-based `src/main.rs` exposing:

```sh
publisher render --root index.typ --to html
publisher render --root index.typ --to pdf
publisher render --root index.typ --scope thesis --to pdf
publisher inspect --root index.typ scopes
```

No scope plus HTML renders per-source HTML for every node; no scope plus PDF renders the whole
publication; `--scope` plus PDF renders that named scope. `inspect … scopes` reuses
`src/inspect.rs` to print scope ids, titles, tags, covered sources, active scope stacks, graph
order, routes, and diagnostics.

=== Slice H: Tests against real output

Rewrite the render tests to compile generated entrypoints and assert real rendered structure:
the named-scope and whole-publication PDFs compile to non-empty bytes; assembled HTML has
exactly one bibliography section listing only cited works; the whole-publication outline over a
scope lists only that scope's headings; a per-source page links to a cross-source label with
real display text; one HTML document is emitted per source. Assert structural facts via
substring and count, never exact bytes. Add one end-to-end test over the fixture across all
targets, retiring the example evidence harness into a real test.

== Acceptance

PRD-003 is complete when the commands in Slice G all succeed against
`examples/prd003_discovery_site/index.typ`:

- per-source HTML emits one document per source, with cross-source links resolving;
- the whole-publication and `thesis` PDFs compile with exactly one bibliography and
  scope-local outlines;
- `inspect … scopes` prints the full scope and graph diagnostic table;

and the legacy `Query`, `Projection`, user-facing `Node`, and `publisher.outline/bibliography/
ref/child` surfaces no longer exist in the crate.
