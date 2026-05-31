= Implementation model map

This maps the PRD-003 source-document model to current Rust implementation pieces.
Some type names are legacy internal names from PRD-001 and PRD-002; they are not author-facing API language.

== Core implementation records

- `Publication`: `src/model.rs` `Publication`, built by `src/parser.rs` `parse_publication`, validated by `Publication::validate`.
- Source document record: `src/model.rs` `Node`; one reachable Typst source document with source path, route, parent, ordered published children, properties, declared scopes, and legacy projection records.
- `Scope`: `src/model.rs` `Scope`, `ScopeKind`, and `ScopeExtent`; explicit PRD-003 scopes use `ScopeKind::Publication`.
- Source-local boundary: `ScopeKind::CurrentPage`, still named from an earlier milestone, is the implicit source-document scope.
- `Spine`: `src/model.rs` `Spine`, derived by `Publication::spine_for`, stores active scope ids from global to nearest.
- `Property`: `src/model.rs` `Property` and `PropertySource`; properties are owned by source documents and interpreted through active scopes.

== Legacy internals

- `Query`: retained for old projection placeholders and inspect output. It should not appear in new PRD-003 authoring docs as a user concept.
- `Projection`: retained for old navigation, outline, bibliography, and reference placeholders. PRD-003 rendering should prefer ordinary Typst calls plus generated render-boundary adapters.
- Old `publisher.child`, `publisher.children`, `publisher.outline`, `publisher.bibliography`, and `publisher.ref` examples remain only as legacy fixture coverage until their tests are migrated or deleted.

== Parser

- Typst parsing: `src/parser.rs` uses `typst-syntax` to walk syntax nodes for headings, references, and ordinary bibliography calls.
- Marker evaluation: `src/parser/world.rs` embeds Typst with a root-confined parser world, and `src/parser/markers.rs` reads `metadata(...) <publisher-marker>` values emitted by the local publisher library.
- Marker dispatch: `src/parser/calls.rs` converts decoded `publish`, `scope`, and legacy markers into the internal model.
- PRD-003 fixture root: `examples/prd003_discovery_site/index.typ`.
- Legacy fixture root: `examples/api_sketch_site/index.typ`.
- Reachability: `#publish(path)` markers produce publication edges. Typst `#include(path)` does not.
- Warnings: unreachable Typst source files are reported as `ParseWarningKind::UnreachableTypFile`.

== Rendering

- `src/render.rs` writes generated source-local HTML inputs under `typst/`.
- `src/render.rs` writes combined-PDF inputs under `typst-pdf/`.
- The whole-publication proof entrypoint is `<artifact>.typ`.
- Named-scope entrypoints are written under `scopes/`.
- Pairwise scope-union entrypoints are written under `scope-unions/`.
- Source-local HTML reference adapters lower cross-source `@label` references to explicit links.
- Source-local HTML counter seeding injects `#counter(heading).update(n)` for numbered multi-source scopes.
- Combined PDF proof inputs keep bibliography and reference adapter output explicit where needed to avoid Typst multiplicity or missing-numbering failures.

== Inspect and validation

- Inspect output: `src/inspect.rs` `inspect_publication` renders deterministic plain text from typed `Publication` data.
- Normal/debug mode: `InspectOptions::debug_defaults` controls whether defaulted legacy query values are materialized.
- Example runner: `examples/api_sketch.rs` parses, validates, renders, and exits nonzero on validation errors.
- Snapshot evidence: `tests/inspect_api_sketch.rs` checks API-sketch and PRD-003 inspect output with `insta`.
