= PRD-003 implementation slices

This document is intended as an orchestrator handoff.
It breaks PRD-003 into independently reviewable slices after the selector/scope discovery report.

== Required reading

An implementation orchestrator should read these first:

- `design/prd-003-typst-selector-usage.typ`: target authoring model and clarified rendered-region semantics.
- `design/prd-003-discovery-report.typ`: discovery evidence and implementation recommendation.
- `examples/prd003_discovery_site/`: main PRD-003 scope/publish fixture.
- `examples/prd003_counter_ref_site/`: focused cross-source counter and reference fixture.
- `docs/implementation-notes.typ`: failed approaches and corrected implementation conclusions.

== Discovery status

The discovery phase is complete enough to start implementation.
Do not create new discovery-only slices for outline scope locality, bibliography multiplicity, cross-source HTML references, or cross-source counters unless implementation evidence contradicts the recorded conclusions.

Remaining work should be implementation work with focused verification.
If an implementation attempt fails, record the failure and corrected approach in `docs/implementation-notes.typ`.

== Implementation checkpoint

Status after the PRD-003 implementation run that ended at commit `ac7b384`:

- Slice 1 is complete and committed as `84df818 Add PRD-003 marker API`.
  Evidence: `cargo test --test api_sketch_parser` and `cargo run --example prd003_discovery`.
- Slice 2 is complete and committed as `eab3f67 Add PRD-003 source routes`.
  Evidence: `cargo test --test api_sketch_parser` and `cargo test --test model_validation`.
- Slice 3 is complete and committed as `25ed4b9 Add PRD-003 scope validation`.
  Evidence: `cargo test`, `cargo run --example prd003_discovery`, and the PRD-003 model validation assertions.
- Slice 4 is complete and committed as `afca403 Add PRD-003 inspect snapshot`.
  Evidence: `cargo test --test inspect_api_sketch`; the PRD-003 snapshot shows routes, publish order, scope extents, active scope stacks, duplicate-title diagnostics, and parse warnings.
- Slice 5 has a first implementation committed as `43a3dbd Lower PRD-003 outlines by scope`.
  Evidence: `cargo test --test render_assembly`, `cargo test`, and `cargo run --example prd003_discovery`.
  Current implementation lowers ordinary `#outline(...)` in generated source to a scope-local rendered outline block using the nearest active publication scope.
  This proves scope-local rendered behavior, but it is not yet the final bounded-selector implementation described as the ideal shape below.
- Slice 6 diagnostics are committed as `53b695e Add PRD-003 bibliography diagnostics`.
  Evidence: `cargo test`, `cargo run --example prd003_discovery`, and the PRD-003 inspect snapshot.
  Ordinary `#bibliography(...)` calls are detected as `bibliography-source` properties, and duplicate bibliography calls are warned per active publication/source scope.
- Slice 6 current-renderer generation support is committed as `c44ebbc Add PRD-003 bibliography render boundaries`.
  Evidence: `cargo test --test render_assembly`, `cargo test`, and `cargo run --example prd003_discovery`.
  Generated source now has separate render boundaries:
  source-local HTML inputs under `typst/` preserve the ordinary authored `#bibliography(...)` call, while combined PDF inputs under `typst-pdf/` lower bibliography calls to an explicit scoped bibliography placeholder so Typst does not see multiple ordinary bibliography calls in the combined proof artifact.
  This proves source-local bibliography generation and prevents combined-render multiplicity, but it is not yet the final reduced bibliography file generation for future named-scope render targets.
- Slice 7 has a first implementation committed as `3be2f91 Adapt PRD-003 cross-source HTML refs`.
  Evidence: `cargo test --test render_assembly`, `cargo test`, `cargo run --example prd003_discovery`, and the focused `examples/prd003_counter_ref_site` reference commands.
  Source-local HTML generation now discovers labels across reachable source documents, preserves same-source references where the source-local compilation owns the target, and lowers cross-source references to explicit `#link(...)` expressions using the target route and titled target-scope context.
  The renderer does not hidden-include target sources for references, so local outlines and bibliography inputs are not polluted.
  Combined PDF proof inputs keep generated references as explicit text until a later combined-entrypoint reference policy is implemented.
- Slice 8 has a first implementation committed as `f7b0255 Seed PRD-003 source HTML counters`.
  Evidence: `cargo test --test render_assembly`, `cargo test`, `cargo run --example prd003_discovery`, and the focused `examples/prd003_counter_ref_site` counter/reference commands.
  Source-local HTML generation now computes a heading counter seed for numbered multi-source publication scopes and injects `#counter(heading).update(n)` before a later source's content, so a second thesis source can render as chapter `2.` without hidden-including the first source.
  Whole-publication PDF proof inputs remain unseeded, allowing Typst counters to continue naturally across the generated entrypoint.
  This is a first render-generation implementation; inspect output does not yet expose the computed counter context.
- Slice 9 has a first implementation committed as `ac7b384 Generate PRD-003 scope entrypoints`.
  Evidence: `cargo test --test render_assembly`, `cargo test`, and `cargo run --example prd003_discovery`.
  Rendering now writes reviewable named-scope entrypoint files under `scopes/` and deterministic pairwise scope-union entrypoint files under `scope-unions/`, using generated PDF-boundary sources in publication graph order.
  The `thesis` scope entrypoint includes only thesis sources, and the `writing+thesis` union entrypoint includes writing and thesis sources once each while excluding `cv`.
  This is generation-only entrypoint support; no CLI/API surface for selecting and compiling these entrypoints has been added yet.

Resume from Slice 10:

- Perform PRD cleanup and migration.
- Keep the scope entrypoints from `ac7b384`, the counter seeding from `f7b0255`, the reference adapter from `3be2f91`, the duplicate-bibliography diagnostics from `53b695e`, and the split HTML/PDF render-boundary generation from `c44ebbc`.
- When named-scope render targets are introduced later, finish the remaining bibliography refinement there: generate exactly one scoped bibliography input, possibly with a reduced temporary bibliography file, for the selected named-scope render boundary.

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

Cross-source counters:

- Combined scope/edition entrypoints naturally continue Typst counters across source files.
- Per-source HTML compilations reset counters unless the publisher seeds counter state.
- For numbered multi-source scopes, the publisher should compute and inject counter seed/state into generated per-source HTML input.
- Hidden-including prior sources to advance counters is not acceptable.

== Preflight: verify discovery artifacts

Goal:
Confirm the handoff artifacts still compile and the discovery evidence remains reproducible.
This is not a discovery slice and should not expand scope.

Tasks:

- Compile the PRD and report.
- Run the main discovery harness.
- Run the counter/reference fixture commands when counter or reference behavior is being touched.

Acceptance:

```sh
typst compile design/prd-003-typst-selector-usage.typ /private/tmp/prd-003-typst-selector-usage.pdf
typst compile design/prd-003-discovery-report.typ /private/tmp/prd-003-discovery-report.pdf
cargo run --example prd003_discovery
typst compile --features html --format html examples/prd003_counter_ref_site/thesis-combined.typ /private/tmp/thesis-combined.html
typst compile --features html --format html examples/prd003_counter_ref_site/thesis/ch-02.typ /private/tmp/ch-02-unseeded.html
typst compile --features html --format html examples/prd003_counter_ref_site/thesis-ch-02-seeded.typ /private/tmp/ch-02-seeded.html
typst compile --features html --format html examples/prd003_counter_ref_site/combined-ref.typ /private/tmp/combined-ref.html
typst compile --features html --format html examples/prd003_counter_ref_site/blog-adapted-ref.typ /private/tmp/blog-adapted-ref.html
```

Expected known failure:

```sh
typst compile examples/prd003_counter_ref_site/blog-standalone.typ /private/tmp/blog-standalone.pdf
```

This should fail with a missing `<ch-02>` label.
That failure is the evidence that per-source HTML cross-source references need an adapter.

== Slice 1: Add PRD-003 marker API

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

== Slice 2: Source-document graph and route model

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

== Slice 3: Scope extent and inheritance model

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

== Slice 4: PRD-003 inspect output

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

== Slice 5: Scope-local outline lowering

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

== Slice 6: Bibliography diagnostics and scoped bibliography generation

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

== Slice 7: Cross-source HTML reference adapter

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

== Slice 8: Cross-source counters and heading numbering

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

== Slice 9: Named scope and union render entrypoints

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

== Definition of done

All slices are complete only when the implementation satisfies every PRD-003 acceptance point below.
The final orchestrator closeout should cite the slice, command, and artifact proving each row.

=== Authoring and graph

- Fixture shape:
  `index.typ` publishes `writing.typ`, `thesis/intro.typ`, and `cv.typ`.
  Covered by slices 1 and 2.
  Validate with `cargo run --example prd003_discovery` and PRD-003 inspect output.

- Writing scope:
  `writing.typ` declares `#scope("writing", title: [Writing])` and publishes at least two writing source documents.
  Covered by slices 1, 2, 3, and 4.
  Validate with parser/model tests and inspect output.

- Thesis scope:
  `thesis/intro.typ` declares `#scope("thesis", title: [Thesis])` and publishes at least one thesis chapter.
  Covered by slices 1, 2, 3, and 4.
  Validate with parser/model tests and inspect output.

- Literal include remains distinct:
  `#include "summary.typ"` is literal content and not a routed source document.
  Covered by slice 2.
  Validate with graph/route inspect output.

- HTML routing:
  The HTML edition emits one HTML document per published source Typst document.
  Covered by slices 2 and 9.
  Validate with render tests and generated HTML paths.

=== Scope semantics and diagnostics

- Scope extents:
  Explicit scopes cover the declaring source and published descendants, with inherited scopes overlapping child scopes.
  Covered by slices 3 and 4.
  Validate with model tests and active scope stack inspect output.

- Inspect output:
  Inspect shows scope ids, explicit titles, tags, source-document inheritance, active scope stacks, publication graph order, routes, and diagnostics.
  Covered by slice 4.
  Validate with a PRD-003 inspect snapshot or focused assertions.

- Duplicate scope ids:
  Duplicate explicit scope ids are errors.
  Covered by slices 3 and 4.
  Validate with model/validation tests.

- Duplicate display titles:
  Duplicate display titles are warnings, not model errors.
  Covered by slices 3 and 4.
  Validate with model/validation tests and inspect diagnostics.

=== Ordinary Typst behavior in scopes

- Scope-local outline:
  A normal authored `#outline(target: heading.where(level: 1))` renders over the active scope-local region without `publisher.outline(...)`.
  Covered by slice 5.
  Validate with render output showing only headings in the active scope.

- Scope-local bibliography:
  A normal authored `#bibliography("works.yml")` renders for the active scope/source boundary under the one-bibliography-per-scope rule.
  Covered by slice 6.
  Validate with render output and duplicate-bibliography diagnostics.

- Explicit scope union:
  `publisher.in-scope("writing", "thesis")[...]` can render an outline or bibliography over the union in publication graph order.
  Covered by slices 5, 6, and 9.
  Validate with generated union entrypoint or bounded selector output.

- Counters across sources:
  A numbered multi-source scope can render `ch-01.typ` and `ch-02.typ` as `1.` and `2.` in scope/edition output, and can continue numbering in per-source HTML when required.
  Covered by slice 8.
  Validate with counter fixture expectations from `examples/prd003_counter_ref_site/`.

- Source-local resets:
  Implicit source-document scopes can still reset local behavior when they are the intended boundary.
  Covered by slices 3, 6, and 8.
  Validate with source-local bibliography/counter tests.

=== References

- Same-compilation references:
  Standard Typst references preserve native lookup when origin and target are present in the same generated entrypoint.
  Covered by slices 7 and 9.
  Validate with combined reference fixture output.

- Cross-source HTML references:
  A standard authored reference from outside `thesis` to a globally unique thesis label can render in per-source HTML without hidden inclusion.
  Covered by slice 7.
  Validate with generated HTML link output.

- Scope-aware reference display:
  Cross-scope references can include the nearest explicit titled target scope, such as `Thesis`, when origin and target cross a meaningful titled-scope boundary.
  Covered by slice 7.
  Validate with rendered HTML/PDF display text and titled-scope tests.

- Label rules:
  Labels are globally unique across the publication, and reference lookup is not current-scope-first.
  Covered by slices 3, 4, and 7.
  Validate with duplicate-label and cross-scope reference tests.

=== Render targets and CLI/API capability

- Whole publication render:
  The publisher can generate and compile a whole-publication entrypoint.
  Covered by slice 9.
  Validate with render tests.

- Named scope render:
  The publisher can generate and compile a named-scope entrypoint, such as `thesis`.
  Covered by slice 9.
  Validate with render tests.

- Source-local HTML render:
  The publisher can generate per-source HTML entrypoints and route outputs.
  Covered by slices 7, 8, and 9.
  Validate with render tests and generated files.

- Scope inspect command/API:
  The publisher exposes enough inspect output for `publisher inspect --root index.typ scopes` or the current equivalent.
  Covered by slice 4.
  Validate with CLI/example output or snapshot tests.

=== Migration cleanup

- Old authoring APIs:
  New target docs and fixtures do not rely on `publisher.child`, `publisher.outline`, `publisher.bibliography`, or `publisher.ref`.
  Covered by slice 10.
  Validate with docs/examples review and tests.

- Old model terms:
  `Query`, `Projection`, and user-facing `Node` are demoted from PRD-003 authoring docs.
  Covered by slice 10.
  Validate with docs review.

=== Final validation bundle

Run this before claiming PRD-003 complete:

```sh
cargo test
cargo run --example prd003_discovery
typst compile design/prd-003-typst-selector-usage.typ /private/tmp/prd-003-typst-selector-usage.pdf
typst compile design/prd-003-discovery-report.typ /private/tmp/prd-003-discovery-report.pdf
typst compile design/prd-003-implementation-slices.typ /private/tmp/prd-003-implementation-slices.pdf
typst compile docs/model.typ /private/tmp/model.pdf
typst compile docs/model-map.typ /private/tmp/model-map.pdf
typst compile docs/api-sketch.typ /private/tmp/api-sketch.pdf
```

Also run the focused counter/reference fixture commands from the preflight section whenever reference or counter behavior has changed.

== Slice 10: PRD cleanup and migration

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
