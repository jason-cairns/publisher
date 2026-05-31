= PRD-003 implementation slices

This document is intended as an orchestrator handoff.
It breaks PRD-003 into independently reviewable slices after the selector/scope discovery report.

== Implementation stance

Preserve the target authoring model:

```typ
#import "publisher.typ": scope, publish

= Home

#publish("writing.typ")
#scope("home", tags: ("nav",))
```

Authors should use ordinary Typst calls such as:

```typ
#outline(target: heading.where(level: 1))
#bibliography("works.yml")
@some-label
```

The publisher may adapt generated Typst input before compilation, but it should not reintroduce user-facing `publisher.outline(...)`, `publisher.bibliography(...)`, `publisher.ref(...)`, `Query`, `Projection`, or `Node` concepts.

The first implementation milestone should prove graph and scope semantics through inspect output before final rendering behavior.

== Hard decisions already made

Bibliographies:

- Allow at most one ordinary `#bibliography(...)` per active publisher scope.
- This applies even when the scope is an implicit source-document scope.
- Duplicate bibliography calls in the same scope are diagnostics, not something the renderer tries to merge silently.
- Scope/edition rendering may generate temporary bibliography inputs if needed to give Typst exactly the cited works for that scope.
- Prefer an elegant generated bibliography source or Typst-side data adapter later, but do not block PRD-003 on that elegance.

Cross-source HTML references:

- Typst label lookup is compilation-local.
- Source-local HTML keeps one Typst compilation per routed source document, so labels in other source documents are absent.
- Hidden-including other sources just to satisfy references is not acceptable because it pollutes ordinary `query(...)`, outlines, counters, and bibliography inputs.
- Therefore cross-source HTML references require an explicit publisher adapter.
- The adapter should preserve authored `@label` where possible in source, but generated per-source HTML input may need a replacement link/display expression for references whose target lives outside the current source compilation.

Outlines:

- Authored `#outline(...)` must behave scope-locally at scope-local call sites.
- Typst does not infer publisher scope boundaries from `#scope(...)` metadata.
- The publisher must provide scope locality before compilation by using a scoped entrypoint or generated bounded selectors.

== Slice 1: PRD-003 fixture and docs alignment

Goal:
Keep the fixture and design docs aligned with the accepted target shape.

Inputs:

- `design/prd-003-typst-selector-usage.typ`
- `design/prd-003-discovery-report.typ`
- `examples/prd003_discovery_site/`

Tasks:

- Ensure fixture uses only `#scope(...)` and `#publish(...)` for publisher semantics.
- Ensure ordinary Typst calls remain ordinary in fixture sources.
- Keep `#include` in the fixture as literal inclusion evidence.
- Make docs explicitly define render target, rendered region, and Typst entrypoint.

Acceptance:

```sh
typst compile design/prd-003-typst-selector-usage.typ /private/tmp/prd-003-typst-selector-usage.pdf
typst compile design/prd-003-discovery-report.typ /private/tmp/prd-003-discovery-report.pdf
cargo run --example prd003_discovery
```

== Slice 2: Add PRD-003 marker API

Goal:
Parse the new authoring API without preserving the old user-facing API as the implementation target.

Owned files:

- `src/parser/markers.rs`
- `src/parser/calls.rs`
- `src/parser/projections.rs` or a new parser module if cleaner
- `examples/prd003_discovery_site/publisher.typ`
- focused parser tests

Tasks:

- Decode `#publish(path)` metadata markers.
- Decode `#scope(id, title: none, tags: ())` metadata markers.
- Treat scope id as the globally unique machine identity.
- Treat title as optional display context, not identity.
- Treat tags as metadata only.
- Keep old API behavior stable for existing tests until an explicit migration slice removes it.

Acceptance:

```sh
cargo test --test api_sketch_parser
cargo run --example prd003_discovery
```

Inspect or test evidence must show decoded `publish` edges and scope metadata from the PRD-003 fixture.

== Slice 3: Source-document graph and route model

Goal:
Build the PRD-003 source-document graph from `#publish(...)` only.

Owned files:

- `src/model.rs`
- `src/parser.rs`
- `src/validation.rs`
- focused model/parser tests

Tasks:

- Represent source documents as implementation provenance/routing units without making user-facing `Node` language part of PRD-003 docs.
- Build graph edges only from `#publish(...)`.
- Leave Typst `#include` as literal content, not a routed edge.
- Derive default HTML route from source path, such as `thesis/intro.typ -> thesis/intro.html`.
- Preserve deterministic graph order from publication declaration order.

Acceptance:

```sh
cargo test --test api_sketch_parser
cargo test --test model_validation
```

Evidence must show:

- reachable source-document order;
- publish parent/child edges;
- literal included files are not routed source documents;
- route paths for every reachable source.

== Slice 4: Scope extent and inheritance model

Goal:
Make the intended scope semantics concrete and inspectable.

Owned files:

- `src/model.rs`
- `src/validation.rs`
- focused tests

Tasks:

- Explicit scopes cover the declaring source document and all published descendants.
- Scopes inherited through `#publish(...)` overlap child scopes; they do not shadow them.
- Add implicit source-document scopes for source-local outline, bibliography, labels, and compilation behavior.
- Keep source scopes implementation-facing; do not call them `current-page` in new PRD-003 docs.
- Compute active scope stacks per source document.
- Detect duplicate explicit scope ids as errors.
- Detect duplicate display titles as warnings when they may affect reference display.

Acceptance:

```sh
cargo test --test model_validation
```

Tests must cover:

- inherited parent scope;
- child-declared overlapping scope;
- duplicate id error;
- duplicate title warning;
- implicit source scope boundaries.

== Slice 5: PRD-003 inspect output

Goal:
Give authors and implementation agents enough diagnostics to trust scope behavior before rendering.

Owned files:

- `src/inspect.rs`
- `examples/` or tests for PRD-003 inspect fixture
- snapshot tests

Tasks:

- Add inspect output for source-document graph order.
- Show routes.
- Show publish edges.
- Show declared scopes with id, title, tags, declaring source.
- Show scope extents.
- Show active scope stack per source document.
- Show diagnostics for duplicate ids, duplicate titles, missing ids, and duplicate bibliography calls per scope.

Acceptance:

```sh
cargo test --test inspect_api_sketch
```

Add a PRD-003-specific inspect snapshot or equivalent focused assertion.

== Slice 6: Scope-local outline lowering

Goal:
Make ordinary authored `#outline(...)` behave scope-locally without `publisher.outline(...)`.

Owned files:

- render/generation code
- focused render tests

Tasks:

- Locate authored ordinary `#outline(...)` calls in generated source.
- Determine the active publisher scope at the call site.
- For render targets where the compiled entrypoint already equals the scope, leave the call ordinary where possible.
- For larger entrypoints, generate a bounded `target:` selector using scope start/end markers or a union of bounded windows.
- Preserve the author's existing `target:` selector by composing it with the generated scope boundary.

Acceptance:

```sh
cargo test --test render_assembly
cargo run --example prd003_discovery
```

Evidence must show a normal authored outline rendering only headings from the active scope.

== Slice 7: Bibliography diagnostics and scoped bibliography generation

Goal:
Implement the accepted one-bibliography-per-scope rule.

Owned files:

- parser/model validation
- render/generation code
- focused tests

Tasks:

- Detect ordinary `#bibliography(...)` calls and associate each with the active source/scope boundary.
- Emit a diagnostic when more than one bibliography call appears in the same scope, including implicit source-document scopes.
- For valid scope renders, ensure Typst sees exactly one bibliography call for the rendered scope.
- If rendering a scope needs a reduced bibliography input, generate a temporary bibliography file containing cited works for that scope.

Acceptance:

```sh
cargo test
cargo run --example prd003_discovery
```

Evidence must show:

- duplicate bibliography calls in one scope are rejected or warned as specified;
- one bibliography in a source scope succeeds;
- one bibliography in a named scope succeeds.

== Slice 8: Cross-source HTML reference adapter

Goal:
Make standard authored references work in one-HTML-per-source output.

Owned files:

- label/reference discovery code
- render/generation code
- focused tests

Tasks:

- Discover labels and their owning source document and active titled scopes.
- Discover reference origins and targets.
- For references whose target is in the same source compilation, preserve standard Typst lookup where possible.
- For references whose target is outside the source compilation, replace or lower the generated per-source HTML input to a link/display expression that does not require hidden inclusion.
- Add titled-scope display context when origin and target cross a meaningful titled scope boundary.
- Preserve normal lookup semantics: labels remain globally unique; no current-scope-first lookup.

Acceptance:

```sh
cargo test --test render_assembly
cargo run --example prd003_discovery
```

Evidence must show a blog page can link to a thesis label in per-source HTML without hidden target inclusion and without polluting local outline or bibliography inputs.

== Slice 9: Cross-source counters and heading numbering

Goal:
Make counters behave correctly across published source documents for render targets that span multiple sources.

Motivating example:

```text
thesis/ch-01.typ -> heading number 1.
thesis/ch-02.typ -> heading number 2.
```

Implementation distinction:

- For whole-publication, named-scope, and scope-union render targets, the publisher should generate one Typst entrypoint containing the ordered source/content region.
  Typst can then run counters naturally across `#include`d or assembled source content.
- For per-source HTML, each source is compiled independently to preserve one HTML file per source document.
  Typst therefore cannot naturally know the prior source's counter state unless the publisher provides it.

Owned files:

- render/generation code
- model/inspect code for render order and source scope order
- focused tests

Tasks:

- Define which scopes reset heading counters.
- Define how source-document boundaries affect heading counters.
- Generate whole-scope and whole-publication entrypoints in publication graph order so Typst counters naturally produce `1.`, `2.`, etc. across source files.
- For per-source HTML, decide and implement one of:
  - source-local numbering resets per source document; or
  - publisher-provided counter seed/state at the start of each source HTML compilation.
- Prefer source-local resets only for implicit source scopes where that is the intended boundary.
- For numbered multi-source scopes such as a thesis, provide counter seed/state so `ch-02.typ` can render as chapter `2.` even when compiled as its own HTML page.
- Make the inspect output show computed counter context per source document for review.
- Avoid hidden-including prior sources to advance counters, because that pollutes outlines, bibliography, and queries.

Acceptance:

```sh
cargo test --test render_assembly
cargo run --example prd003_discovery
```

Evidence must show:

- a multi-source scope entrypoint renders ordered heading numbers across `ch-01.typ` and `ch-02.typ`;
- per-source HTML for `ch-02.typ` can render with the correct continued heading number when the active scope requires cross-source numbering;
- source-local scopes can still reset numbering when intended;
- hidden inclusion is not used to seed counters.

== Slice 10: Named scope and union render entrypoints

Goal:
Render selected scopes and explicit scope unions as generated Typst entrypoints.

Owned files:

- render/generation code
- example or CLI surface
- focused tests

Tasks:

- Generate whole-publication entrypoint.
- Generate source-document HTML entrypoints.
- Generate named-scope entrypoint.
- Generate explicit scope-union entrypoint in publication graph order.
- De-duplicate overlapping source/content windows.
- Preserve one HTML document per published source document for HTML edition.

Acceptance:

```sh
cargo test
cargo run --example prd003_discovery
```

Evidence must show:

- whole publication entrypoint;
- `thesis` scope entrypoint;
- `writing + thesis` union entrypoint;
- per-source HTML route outputs.

== Slice 11: PRD cleanup and migration

Goal:
Remove stale language after PRD-003 behavior is proven.

Owned files:

- `docs/model.typ`
- `docs/model-map.typ`
- `docs/api-sketch.typ`
- design docs
- examples

Tasks:

- Demote old `Query`, `Projection`, and user-facing `Node` concepts in docs.
- Keep implementation-internal structures documented only where needed.
- Update examples from `publisher.child` to `publish`.
- Remove or quarantine old `publisher.outline`, `publisher.bibliography`, and `publisher.ref` examples once replacements are implemented.

Acceptance:

```sh
cargo test
typst compile docs/model.typ /private/tmp/model.pdf
typst compile docs/model-map.typ /private/tmp/model-map.pdf
typst compile docs/api-sketch.typ /private/tmp/api-sketch.pdf
```

== Suggested orchestrator prompt

```text
You are implementing PRD-003 from design/prd-003-implementation-slices.typ.
Start with the earliest incomplete slice.
Do not implement final rendering before graph/scope inspect output is trustworthy.
Preserve ordinary Typst authoring syntax.
Do not reintroduce user-facing publisher.outline, publisher.bibliography, publisher.ref, Query, Projection, or Node concepts.
Commit each slice atomically with evidence in the final response.
When an attempted approach fails, record it and the correct approach in docs/implementation-notes.typ.
```
