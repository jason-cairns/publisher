= Milestone 1 model map

This maps the model terms in `docs/model.typ` and `design/prd-001-core-entities-inspect.md` to the Rust implementation for the core entities inspect milestone.

== Core entities
- `Publication`: `src/model.rs` `Publication`, built by `src/parser.rs` `parse_publication`, validated by `Publication::validate`.
- `Node`: `src/model.rs` `Node`; one reachable Typst source file with parent, ordered children, properties, declared scopes, and projections.
- `Scope`: `src/model.rs` `Scope`, `ScopeKind`, and `ScopeExtent`; implicit global/current-page scopes are created by `Publication::new` and `Publication::add_node`.
- `Spine`: `src/model.rs` `Spine`, derived by `Publication::spine_for`.
- `Property`: `src/model.rs` `Property` and `PropertySource`; properties are node-owned and interpreted through `Publication::property_scope_context`.
- `Query`: `src/model.rs` `Query` and related query enums; `SourcedValue<T>` is implementation provenance for query fields, distinguishing values provided by source from parser/model defaults.
- `Projection`: `src/model.rs` `Projection`, `ProjectionKind`, and `ProjectionSuppression`; inherited navigation and page-level navigation suppression are projection state, not node-owned properties.

== Parser and fixture
- Typst parsing: `src/parser.rs` uses `typst-syntax` to walk syntax nodes and extract supported `publisher.*` calls.
- Fixture root: `examples/api_sketch_site/index.typ`.
- Reachability: child declarations from `publisher.child` and fixture-style `publisher.children("*.typ")` produce the publication tree.
- Warnings: unreachable Typst source files are reported as `ParseWarningKind::UnreachableTypFile`.

== Inspect and validation
- Inspect output: `src/inspect.rs` `inspect_publication` renders deterministic plain text from typed `Publication` data.
- Normal/debug mode: `InspectOptions::debug_defaults` controls whether defaulted query values are materialized.
- Example runner: `examples/api_sketch.rs` parses, validates, prints inspect output, and exits nonzero on validation errors.
- Snapshot evidence: `tests/inspect_api_sketch.rs` checks the full normal inspect output with `insta`.
