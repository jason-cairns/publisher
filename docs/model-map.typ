= Implementation model map

This maps the current source-document model to current Rust implementation pieces.
Some type names are legacy internal names from PRD-001 and PRD-002; they are not author-facing API language.

== Core implementation records

- `Publication`: `src/model.rs` `Publication`, built by `src/parser.rs` `parse_publication`, validated by `Publication::validate`.
- Source document record: `src/model.rs` `Node`; one reachable Typst source document with source path, route, parent, ordered published children, properties, and declared scopes.
- `Scope`: `src/model.rs` `Scope`, `ScopeKind`, and `ScopeExtent`; explicit publication scopes use `ScopeKind::Publication`.
- Source-local boundary: `ScopeKind::CurrentPage`, still named from an earlier milestone, is the implicit source-document scope.
- `Spine`: `src/model.rs` `Spine`, derived by `Publication::spine_for`, stores active scope ids from global to nearest.
- `Property`: `src/model.rs` `Property` and `PropertySource`; properties are owned by source documents and interpreted through active scopes.

== Removed legacy internals

Earlier experimental surfaces were removed:

- `Query` and `Projection` structures: deleted. Rendering lowers ordinary Typst calls at the generated entrypoint instead of recording projection placeholders.
- `publisher.child`, `publisher.children`, `publisher.outline`, `publisher.bibliography`, and `publisher.ref`: removed along with the legacy API sketch fixture. Authors use ordinary `#publish`, `#scope`, `#outline`, `#bibliography`, and `@label`.

== Parser

- Typst parsing: `src/parser.rs` uses `typst-syntax` to walk syntax nodes for headings, references, and ordinary bibliography calls.
- Marker evaluation: `src/parser/world.rs` embeds Typst with a root-confined parser world, and `src/parser/markers.rs` reads `metadata(...) <publisher-marker>` values emitted by the local publisher library.
- Marker dispatch: `src/parser/calls.rs` converts decoded `publish`, `scope`, and `in-scope` markers into the internal model; `src/parser/scopes.rs` records publication scopes.
- Fixture root: `examples/discovery_site/index.typ`.
- Reachability: `#publish(path)` markers produce publication edges. Typst `#include(path)` does not.
- Warnings: unreachable Typst source files are reported as `ParseWarningKind::UnreachableTypFile`.

== Rendering

- `src/render.rs` exposes `render_target(publication, target, edition, options)` over `RenderTarget` (`WholePublication`, `NamedScope`, `ScopeUnion`) and `Edition` (`Html`, `Pdf`).
- `RenderWorld` is the single Typst world; a `main`-overlay lets a generated entrypoint and its transformed `#include`s compile while imports, `.yml`, and assets resolve against the real source tree.
- HTML edition: one document per source. Cross-source `@label` references lower to explicit `#link` with real display text; numbered multi-source scopes seed `#counter(heading).update(n)`.
- PDF edition: the selected region is assembled into one `<artifact>.typ` entrypoint with `#metadata(none) <scope-start/end-id>` markers bracketing each scope, authored `#outline` targets bounded with `.after().before()`, authored bibliographies stripped, and one consolidated `#bibliography` emitted over the union of cited sources.

== Inspect and validation

- Inspect output: `src/inspect.rs` `inspect_publication` renders deterministic plain text from typed `Publication` data.
- CLI: `src/main.rs` exposes `render` and `inspect ... scopes` over the library.
- Snapshot evidence: `tests/model.rs` checks inspect output with `insta`; `tests/render_assembly.rs` compiles generated entrypoints and asserts real rendered output.
