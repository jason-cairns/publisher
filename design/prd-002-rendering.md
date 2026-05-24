# PRD: Milestone 2 — Render the API sketch example to PDF and HTML

## Source snapshot

This PRD is based on the root Rust checkout at commit `2c4f7a47fb79311b2cfbb5d4d99e6900a84700da`.

Read inputs:

- `design/prd-001-core-entities-inspect.md`
- `docs/api-sketch.typ`
- `docs/model.typ`
- `docs/implementation-notes.typ`
- `examples/api_sketch_site/`
- `src/model.rs`
- `src/parser.rs`
- `src/inspect.rs`

This PRD is intended to be self-contained for implementation. The files above explain provenance, but the implementation contract below restates the current model, fixture shape, API entrypoints, dependencies, scope, and acceptance criteria needed for the milestone.

## Problem

Milestone 1 can parse the API sketch fixture, build the core publication entities, validate the model, and inspect the result. It deliberately stops before rendering.

The next milestone should prove that the parsed model can drive a visible edition. The rendered output does not need final query semantics yet. It only needs to turn the API example into reviewable PDF and HTML files, with projection placeholders that make the parsed publication detail visible.

## Goal

Add a small rendering path that takes the parsed `examples/api_sketch_site/index.typ` publication and writes both:

- `build/api-sketch/api-sketch.pdf`
- one HTML file per reachable source node under `build/api-sketch/`, such as `index.html`, `writing.html`, `writing/blog-1.html`, and `thesis/intro.html`

The renderer should render from typed publication data and the authored Typst source content for reachable nodes. The PDF may remain a single combined proof artifact, but HTML should approximate future routed site pages: each output file corresponds to one source node and contains rendered Typst body content from that node. It should not require the production CLI yet, should not call the Typst CLI, and should not implement the real query engine.

## Current implementation context

The current crate is `publisher`, Rust edition `2024`.

Existing dependencies:

```toml
typst = "0.14.2"
typst-kit = { version = "0.14.2", default-features = false, features = ["embed-fonts"] }
typst-syntax = "0.14.2"
```

Existing development dependency:

```toml
insta = "1.43.1"
```

The renderer should add `typst-pdf` at the same Typst crate version. If HTML export requires a direct dependency on the matching exporter crate, add it at the same version too. Keep exporter versions aligned with `typst = "0.14.2"` unless the implementation deliberately updates the whole Typst stack in one atomic change.

The current public Rust API already exposes:

```rust
parse_publication(root_path) -> Result<Publication, ParseError>
Publication::validate() -> ValidationReport
inspect_publication(&Publication, &ValidationReport, &InspectOptions) -> String
```

The current example entrypoint is `examples/api_sketch.rs`. It parses `examples/api_sketch_site/index.typ`, validates the publication, and prints the inspect output. This milestone extends that example so it also writes render artifacts.

The current library re-exports its modules from `src/lib.rs`. Add the rendering API there in the same simple style.

Do not use `v1/` as implementation context. It is old implementation history and out of scope for this milestone.

## Current model summary

The parsed publication model already exists. The renderer should consume it for publication structure, validation, and projection metadata. If the current model does not yet retain enough source syntax or content to preserve authored node bodies, extend the parser/model narrowly to carry that render input rather than deriving publication semantics a second time inside the renderer.

Important entity shapes:

- `Publication` has an explicit `root_node`, ordered reachable `nodes`, `scopes`, `properties`, `queries`, `projections`, and `parse_warnings`.
- `Node` represents one reachable Typst source file. It has `id`, `source_path`, optional `parent`, ordered `children`, owned `properties`, declared explicit scopes, and attached `projections`.
- The publication root is explicit: for the API sketch fixture it is `index.typ`.
- The global scope is implicit and rooted at the explicit root node.
- Each node has an implicit current-page scope that does not flow to children.
- Explicit scopes cover their root node and descendants unless constrained.
- A node spine is derived from the publication model and ordered from global scope to nearest scope.
- Properties belong to nodes. Scope is interpretation context, not property storage.
- Projections belong to origin nodes and reference queries.
- Navigation suppression belongs to the relevant navigation projection as `ProjectionSuppression`, not to node properties or scope attributes.

The existing parser builds these concepts from publisher markers in the fixture. The renderer should not add new parser behavior unless a rendering test exposes a narrow missing fact that is necessary for this milestone.

## API sketch fixture

The implementation target is the current fixture rooted at:

```text
examples/api_sketch_site/index.typ
```

Reachable Typst source nodes should be:

```text
index.typ
writing.typ
writing/blog-1.typ
writing/blog-2.typ
thesis/intro.typ
thesis/bib.typ
thesis/ch-1.typ
cv.typ
```

Expected child relationships:

```text
index.typ
  writing.typ
    writing/blog-1.typ
    writing/blog-2.typ
  thesis/intro.typ
    thesis/bib.typ
    thesis/ch-1.typ
  cv.typ
```

Expected title-like properties:

```text
index.typ -> My Publication
writing.typ -> Writing
writing/blog-1.typ -> Blog 1
writing/blog-2.typ -> Blog 2
thesis/intro.typ -> Thesis
```

Expected projection kinds present in the fixture:

- outline
- navigation
- bibliography
- reference

Expected parse warning:

```text
drafts/unreachable.typ is intentionally unreachable
```

## Milestone summary

By the end of this milestone:

1. A rendering module exists in the Rust library.
2. The renderer can generate per-node Typst render sources from a `Publication`.
3. Projection output is visible as explicit placeholders for outline, navigation, bibliography, and reference projections.
4. The placeholders include useful inspect-level detail such as projection kind, origin node, suppression state, rendering attributes, and query shape.
5. The example command `cargo run --example api_sketch` parses, validates, inspects, and writes the PDF and HTML files.
6. The PDF and HTML files are produced through the Typst Rust crate stack from generated `.typ` files, not virtual in-memory fixtures.
7. Tests verify that per-node HTML contains rendered source content, inherited projection placeholders, and no inspect-style source/model metadata dump.
8. The implementation keeps query evaluation as a non-goal.

## Non-goals

This milestone does not need the production CLI.

It does not need to evaluate arbitrary queries, resolve references, filter bibliographies, build final navigation, or produce final tables of contents.

It does not need final production layout or final projection output, but it should preserve authored content from the reachable source `.typ` files. Projection output may remain placeholder content in this milestone. Projection calls should remain in their authored positions where practical, rendering as explicit placeholders until final query/projection semantics exist.

It does not need a styling system, template registry, plugin system, or routing layer.

## Rendering semantics

The renderer should treat the parsed `Publication` as the source of truth.

The renderer must be strictly structural. Do not implement this milestone by stitching Typst strings together. That approach is too brittle for authored Typst content because it forces the renderer to reason about escaping, markup/code mode boundaries, comments, labels, references, nested publisher calls, and imports as raw text.

Use one of these structural approaches instead:

- operate on Typst syntax/AST nodes and source spans where the Typst crate stack supports it;
- introduce a narrow typed render IR, such as `AssemblyDocument`, if direct Typst syntax rewriting is awkward;
- serialize to `.typ` only at the boundary after the structured representation has been built.

The persisted `.typ` files remain useful as review and debugging artifacts, but they are not the renderer's internal programming model. Treat generated `.typ` files as serialized output from a structural render source, not as a pile of concatenated strings.

For each reachable node, the generated page source should include:

- the node title when one exists
- the authored body content for that node
- each projection attached to that node as visible placeholder content

Projection placeholders should be ordinary Typst content. They should be intentionally obvious, but not pretend to be final output.

A suppressed projection should still be visible in the rendered artifact as a suppressed placeholder, so review can confirm that suppression state was parsed and attached to the projection rather than stored as a node property. Inherited projections that have no authored call site yet, such as inherited navigation, should still render as page-level placeholders so the output approximates the future routed page.

The renderer may normalize or re-emit Typst syntax when serializing render sources, but it should not drop non-publisher authored content from reachable nodes. Publisher calls whose final output depends on unimplemented query/projection semantics should be represented by structural placeholders that preserve the parsed projection detail.

## Export semantics

The renderer may write intermediate Typst render sources under `build/api-sketch/typst/`.

Final export should be delegated to Typst libraries for both formats:

- PDF export uses standard paged Typst compilation plus `typst-pdf`, with the `typst-pdf` version aligned to the existing Typst crate version.
- HTML export uses Typst's experimental HTML document path plus the matching HTML exporter crate, compiling each generated reachable-node `.typ` source to a separate HTML file.

The implementation must not invoke `typst` as a subprocess. A later CLI milestone can decide how production rendering is configured and exposed to users.

The expected dependency shape is direct Rust dependencies on the exporter crates needed for library-level output, including `typst-pdf` for PDF export.

## Rendering API

Add a small rendering API rather than a CLI.

Suggested shape:

```rust
pub struct RenderOptions {
    pub source_root: PathBuf,
    pub output_dir: PathBuf,
    pub artifact_name: String,
}

pub struct RenderedArtifacts {
    pub typst_path: PathBuf,
    pub pdf_path: PathBuf,
    pub html_paths: Vec<PathBuf>,
}

pub fn render_publication(
    publication: &Publication,
    options: &RenderOptions,
) -> Result<RenderedArtifacts, RenderError>
```

The exact names may vary if the implementation has a clearer local fit, but keep the surface small and library-first.

`render_publication` should:

1. Validate or require a validation-clean publication before export.
2. Build generated per-node Typst render sources structurally from the typed publication model and reachable source content.
3. Write per-node `.typ` files under the output directory and write `api-sketch.typ` as a single combined PDF source that includes those page sources.
4. Compile `api-sketch.typ` to a paged Typst document in-process.
5. Export PDF in-process through `typst-pdf`.
6. Compile each generated reachable-node `.typ` file to HTML in-process through the matching Typst HTML path, writing deterministic route-like output paths.
7. Return the paths written.

Errors should report which stage failed: generated Typst source write, paged compilation, PDF export, HTML compilation/export, or output write.

## Generated render source content

The generated page sources should be intentionally simple Typst. They should prioritize approximating the eventual routed pages over final site design.

The generated `.typ` files are useful even if the implementation works with Typst syntax/AST nodes internally:

- they give reviewers stable intermediate artifacts to inspect when PDF or HTML output is wrong;
- they keep PDF and HTML export on shared page sources instead of separate render semantics;
- it makes this milestone a rendering proof from the parsed publication model, not a commitment to final routed-page generation.

Direct Typst syntax/AST manipulation is preferred where it is the clearer implementation path, especially for preserving authored body content. A small typed render IR is also acceptable if the Typst syntax API is not ergonomic enough. The milestone contract is structural render source first, serialized `.typ` artifacts second, and final PDF/HTML export delegated to the Typst crate stack.

Minimum visible structure:

```typ
= API Sketch Publication

== My Publication
Source: `index.typ`
Children: writing.typ, thesis/intro.typ, cv.typ

Projection placeholder:
- kind: outline
- origin: index.typ
- rendering: depth=1
- query: selection=nodes, search=nearest-scope
```

The exact formatting can differ, but the generated PDF and HTML must visibly answer:

- Which nodes rendered?
- What rendered authored content belongs to each node?
- Which projections are visible on each node, including inherited navigation?
- Which projection placeholders are suppressed?
- What query/rendering details came from inspect-level model data?

The renderer should include authored content from the source `.typ` files for this milestone. Where final projection semantics are not implemented, it should render placeholders in place of, or alongside, the corresponding projection output. It should not silently omit parsed projection placeholders.

## Suggested implementation slices

Keep the work atomic. A good split is:

1. PRD/docs only.
2. Add `typst-pdf` and any matching HTML exporter dependency, plus a minimal render module that writes generated per-node Typst render sources and tests their structure.
3. Add in-process PDF/HTML export and tests around artifact creation from those generated `.typ` files.
4. Extend `examples/api_sketch.rs` to call the renderer and print artifact paths.

Do not combine CLI work, query-engine work, routed-page output, or fixture redesign into these commits.

## Acceptance criteria

Running this command should succeed:

```sh
cargo run --example api_sketch
```

It should write:

```text
build/api-sketch/api-sketch.typ
build/api-sketch/api-sketch.pdf
build/api-sketch/typst/index.typ
build/api-sketch/typst/writing.typ
build/api-sketch/index.html
build/api-sketch/writing.html
build/api-sketch/writing/blog-1.html
build/api-sketch/writing/blog-2.html
build/api-sketch/thesis/intro.html
build/api-sketch/thesis/bib.html
build/api-sketch/thesis/ch-1.html
build/api-sketch/cv.html
```

The generated PDF should remain a useful combined proof artifact. The generated HTML pages should visibly include each node's rendered authored content and in-place projection placeholders; they should not be a single inspect-style source/model metadata dump.

The test suite should include focused evidence for the renderer and continue to validate the existing parser/model/inspect behavior.

Suggested verification commands:

```sh
cargo test
cargo run --example api_sketch
test -s build/api-sketch/api-sketch.typ
test -s build/api-sketch/api-sketch.pdf
test -s build/api-sketch/index.html
test -s build/api-sketch/writing/blog-1.html
test -s build/api-sketch/thesis/intro.html
```

The example should print the artifact paths it wrote so a reviewer can open them manually.
