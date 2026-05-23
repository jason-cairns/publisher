# PRD: Milestone 1 — Parse Typst files, build core entities, and inspect the API sketch example

## Source snapshot

This PRD is based on the current `main` branch of `jason-cairns/my-site` at commit `44f98d7fed447e2f1643224daafb46436e94f22c`.

Read inputs:

- `docs/api-sketch.typ`
- `docs/model.typ`
- `docs/style.typ`

The current Rust project is still a scaffold: package name `publisher`, edition `2024`, and `src/main.rs` only prints `Hello, world!`.

## Problem

The project has a clear conceptual model and a concrete Typst-facing API sketch, but the Rust implementation does not yet parse Typst source files, expose the core publication entities, or provide reviewable evidence that a publication has been represented correctly.

The first milestone should make the model inspectable before attempting rendering, full Typst evaluation, query resolution, reference resolution, bibliography generation, HTML/PDF output, or final CLI completeness.

## Goal

Implement the smallest coherent Rust model for the publication system, parse a real set of Typst source files matching the API sketch, and provide an `inspect` path that displays the resulting publication model.

The implementation must be deliberately simple. Code should read like a small, typed Rust representation of the Typst-facing API, not like a framework. The API and internal model should remain close to the concepts in Typst and the docs: source-file nodes, child declarations, scopes, properties, queries, projections, and derived spines.

## Milestone summary

By the end of this milestone:

1. Core entities exist as explicit Rust types.
2. The API sketch example exists as real Typst fixture files.
3. The example is parsed from those Typst files, not hand-constructed directly in Rust.
4. Running `cargo run --example api_sketch` parses the fixture, validates the model, and displays an `inspect` view.
5. The inspection output clearly shows every reachable node, declared child edge, property, scope, current-page scope, active scope spine, and projection represented by the example.
6. Unreachable `.typ` files in the fixture are reported as warnings, not treated as nodes in the publication tree.
7. There is concise, high-value test or snapshot evidence that the parser, model, and inspect output match the documented model and API sketch.

## Non-goals

This milestone does not need to render Typst, HTML, or PDF.

It does not need to deeply evaluate Typst. Parsing only needs to be deep enough to identify the exact publication API calls and simple Typst facts required by the model.

It does not need to evaluate full queries. Queries and projections only need enough representation to inspect them faithfully.

It does not need to resolve references, collect citations into rendered bibliographies, or produce final navigation/table-of-contents output.

It does not need a full production CLI. A runnable example that exercises `inspect` is enough.

It does not need hundreds of tiny tests. The suite should be small, intentional, and readable by a human reviewer.

It must not introduce speculative renderers, registries, plugin systems, full query engines, or generalized extension architecture.

## Design principles

The implementation should prefer plain structs, enums, helper methods, and typed collections over abstractions.

Avoid generic registries, trait-heavy plugin systems, macro-heavy APIs, visitor frameworks, or lifecycle machinery unless there is immediate evidence that they simplify this milestone.

Use structured data construction, not string manipulation. Inspection output should be generated from typed model data.

Use the Typst parser dependency for reading Typst files. Do not write an ad-hoc parser for Typst syntax if the Typst crate can provide the syntax tree needed for this milestone.

Keep naming close to the docs and Typst-facing API. Prefer names such as `Publication`, `Node`, `Scope`, `Property`, `Query`, `Projection`, `Spine`, `outline`, `children`, `child`, `scope`, `bibliography`, and `reference` over invented terminology.

The parsed Typst fixture should make `docs/api-sketch.typ` recognizable. A reader should be able to compare the docs sketch and the fixture files without mentally translating through a different architecture.

When the docs contain obvious small inconsistencies, correct them as part of the milestone. Keep those corrections narrow and strict. Do not use this milestone to reinterpret or redesign the sketch.

## Root and publication semantics

The publication root node is explicit.

Whichever Typst source file is given to the CLI/example is the root node. For this milestone, `index.typ` is the root node.

There is no separate implicit root node in the model for this milestone.

The global scope is implicit. It is rooted at the explicit root node and covers the publication tree reachable from that root.

Files not reachable from the explicit root through `publisher.child` or `publisher.children` are outside the publication tree. If they are `.typ` files under the fixture, report them as warnings so the reviewer can distinguish intentional helpers from accidental omissions.

## Parser requirements

The parser must read real `.typ` files from an example fixture directory.

Create a fixture such as:

```text
examples/api_sketch_site/
  index.typ
  writing.typ
  writing/
    blog-1.typ
    blog-2.typ
  thesis/
    intro.typ
    ch-1.typ
    bib.typ
    bibliography.bib
  cv.typ
  assets/
    img-1.jpg
    img-1.png
    img-2.svg
  works.yml
  README.md
```

`cargo run --example api_sketch` should parse this fixture and inspect the resulting model. The example should treat `index.typ` as the explicit root node.

The parser should be strict and exact for this milestone. It only needs to recognize the exact publisher-call forms used in the fixture, such as `#publisher.child(...)`, `#publisher.children(...)`, `#publisher.scope(...)`, `#publisher.outline(...)`, `#publisher.bibliography(...)`, and `#publisher.ref(...)`.

Do not implement alias handling such as `#let p = publisher` followed by `#p.child(...)`.

Do not implement variable-binding semantics for publisher scopes in this milestone. Use a simpler explicit suppression form in the fixture rather than `#let nav = publisher.scope(kind: "nav")` followed by `#nav.suppress()`.

The parser only needs shallow extraction. It should identify:

- source-file nodes from reachable `.typ` files
- unreachable `.typ` files as warnings
- first highest-level heading/title-like facts where present
- `#publisher.child("...")`
- `#publisher.children("...")` with filesystem glob expansion
- `#publisher.scope(kind: ...)`
- `#publisher.outline(...)`
- `#publisher.bibliography(...)`
- `#publisher.ref(...)`
- explicit nav suppression calls/properties used by the fixture
- simple citation markers such as `@cite` and `@bib-ref`
- simple bibliography source paths such as `works.yml` and `bibliography.bib`
- simple target/filter attributes in the outline example, such as `title: [List of Figures]` and `target: figure.where(kind: image)`

Title extraction rule: use the first heading at the highest heading level present in the node. A node containing only level-2 headings uses its first level-2 heading as its title-like property.

Title values should expose plain text for reviewability and may also retain the raw Typst span/content for later fidelity.

The parser does not need to interpret arbitrary Typst expressions. For unsupported expressions inside a recognized publisher call, use the following conservative rule:

- if the expression can be preserved without changing semantics, store it as an inspectable raw Typst expression and emit a parse warning only when useful for review;
- if the expression prevents identifying the call, origin, target, or required model entity, report a validation error.

Glob expansion must be deterministic. If no explicit order is supplied by the source file, sort expanded child paths lexically.

The parser must distinguish Typst source-file nodes from non-node files. Files such as images, `.bib`, `.yml`, and `README.md` are not nodes, but they may appear as property values, projection attributes, or referenced resources.

## Core model requirements

### Publication

A `Publication` represents a tree of Typst source-file nodes reachable from the explicit root node.

It must have an explicit root node and an implicit global scope rooted at that explicit root node.

It must own or index all parsed reachable nodes, scopes, properties, queries, and projections for inspection.

It must enforce, or at least detect and report during validation, that each non-root node has only one parent.

It must report unreachable `.typ` files under the fixture as warnings.

### Node

A `Node` represents one Typst source file.

At minimum, it must include:

- a stable node identifier
- a source path such as `index.typ` or `thesis/intro.typ`
- an optional parent, except for the explicit root
- ordered children
- properties contributed by the node
- scopes declared by the node
- projections declared by the node

Each source-file node must have its own constrained `current-page` scope that is not inherited by child nodes.

A node may be within many scopes.

### Scope

A `Scope` is a contextual region over source-file nodes.

At minimum, it must include:

- a stable scope identifier
- kind, such as `global`, `current-page`, `nav`, `outline`, `reference`, or `bibliography`
- optional name
- root node
- extent
- attributes
- whether it is implicit or explicit
- source declaration order where relevant

A scope covers its root node and child nodes recursively unless explicitly constrained.

The implicit global scope covers the full publication rooted at the explicit root node.

The implicit current-page scope covers only its owning node.

When multiple explicit scopes are declared by the same node, their order is source declaration order.

### Spine

A `Spine` is derived, not separately declared.

For any node, the system must be able to report active scopes ordered from global scope to nearest scope.

Within the same node, the current-page scope comes before explicit scopes declared on that node.

The expected ordering shape is:

```text
global -> inherited explicit scopes -> current-page(node) -> explicit scopes declared on node in source order
```

Inspection must show each node’s spine so scope membership is visible and testable.

### Property

A `Property` is a fact about a node.

At minimum, it must include:

- key
- value
- source
- optional attributes
- owning node
- whether it is implicit or explicit

Properties belong to nodes, not scopes. Inspection should make this clear.

The model should support implicit properties such as source path and title. It should also leave room for number, target, and output path without requiring full rendering semantics in this milestone.

Nav suppression should be represented as a node-owned property and also consumed as projection behavior: when constructing or inspecting a navigation projection for a node, the projection should show that it is suppressed for that node because of the node property.

### Query

A `Query` is a structured selection plan.

For this milestone, it only needs enough structure to represent queries attached to projections and references from the API sketch.

At minimum, it should be able to represent:

- origin node
- selection target: nodes, scopes, properties, or resolved target
- search rule, such as current node, current scope, nearest scope, active scopes, named scope, or whole publication
- optional filters
- ordering, especially publication-tree order
- visibility
- fallback rule
- ambiguity rule
- whether each value was explicit or defaulted

Query evaluation may be deferred. Inspection must still display the query structure.

Defaults should be available to the model, but normal inspect output should not obscure what was actually written in the source. Materialized defaults should be shown only when a debug flag is enabled.

For the example, use a debug flag such as:

```sh
cargo run --example api_sketch -- --debug-defaults
```

The exact flag name may differ, but the PR must document it.

### Projection

A `Projection` is generated Typst-compatible output inserted into a node.

For this milestone, it should be represented but not rendered.

At minimum, it must include:

- origin node
- kind, such as `outline`, `navigation`, `bibliography`, or `reference`
- query
- rendering attributes
- suppression state where relevant

`publisher.outline(...)`, `publisher.bibliography(...)`, and `publisher.ref(...)` from the API sketch should appear as inspectable projections or inline query/projection records.

Nav suppression must be visible as projection behavior. It can be driven by a node-owned property, but inspect output should make clear that the nav projection is suppressed on that node.

### Validation

The example construction should run validation before inspection.

Validation should report clear errors for:

- duplicate node source paths
- missing child target paths
- multiple parents for one node
- duplicate scope identifiers or invalid scope roots
- projection/query origins that do not exist
- unsupported publisher calls that are close enough to look intentional but cannot be represented

Validation should report warnings for:

- unreachable `.typ` files under the fixture
- preserved raw Typst expressions where the reviewer should know shallow parsing was used

Validation should stay simple and local. Do not build a full compiler pipeline for this milestone.

## API sketch fixture requirements

Create `examples/api_sketch.rs` and a corresponding Typst fixture directory.

Running this command should produce the inspection output:

```sh
cargo run --example api_sketch
```

The fixture should mirror the publication described by `docs/api-sketch.typ`, with obvious small inconsistencies corrected. In particular, use a consistent thesis bibliography node/path such as `thesis/bib.typ` for the source node and `thesis/bibliography.bib` for the bibliography source.

Required source-file nodes:

- `index.typ`
- `writing.typ`
- `writing/blog-1.typ`
- `writing/blog-2.typ`
- `thesis/intro.typ`
- `thesis/ch-1.typ`
- `thesis/bib.typ`
- `cv.typ`

Required non-node files should not become nodes, but may be referenced by properties or projection attributes where relevant:

- `thesis/bibliography.bib`
- `assets/img-1.jpg`
- `assets/img-1.png`
- `assets/img-2.svg`
- `works.yml`
- `README.md`

The fixture should include at least one unreachable `.typ` file so warning behavior is visible and reviewable.

Required child declarations:

- `index.typ` has `writing.typ`, `thesis/intro.typ`, and `cv.typ`
- `writing.typ` has `writing/blog-1.typ` and `writing/blog-2.typ`
- `thesis/intro.typ` has `thesis/ch-1.typ` and `thesis/bib.typ`

Required scopes:

- implicit global scope at `index.typ`
- implicit current-page scope for every reachable node
- `nav` scope declared by `index.typ`
- `outline` scope declared by `writing.typ`
- `reference` scope declared by `writing.typ`
- `outline` scope declared by `thesis/intro.typ`
- `reference` scope declared by `thesis/intro.typ`
- `bibliography` scope declared by `thesis/intro.typ`
- second `nav` scope declared by `thesis/intro.typ`

Required properties:

- source path for every reachable node
- title-like property for `index.typ`, `writing.typ`, `writing/blog-1.typ`, `writing/blog-2.typ`, and `thesis/intro.typ`
- citation-like property for `writing/blog-1.typ`
- citation-like property for `thesis/ch-1.typ`
- reference-like inline query or projection for `writing/blog-2.typ`
- bibliography source properties or projection attributes for `works.yml` and `bibliography.bib`
- nav suppression property for `index.typ` and `cv.typ`

Required projections:

- outline projection in `index.typ` with depth `1`
- suppressed navigation behavior for `index.typ` due to its nav suppression property
- outline projection in `writing.typ` with depth `1`
- bibliography projection in `writing/blog-1.typ` scoped to current page
- reference projection or inline query in `writing/blog-2.typ` targeting `<figure-1>`
- outline projection in `thesis/intro.typ` with depth `1`
- list-of-figures-style outline projection in `thesis/intro.typ` with title `List of Figures` and target equivalent to `figure.where(kind: image)`
- bibliography projection in `thesis/bib.typ`, defaulting to nearest bibliography scope
- suppressed navigation behavior for `cv.typ` due to its nav suppression property

## Inspect requirements

`inspect` should display the parsed model, not a hand-written summary.

The first implementation may expose `inspect` as a Rust function used by the example rather than as a complete CLI subcommand.

The output should be deterministic. Sort only where the model does not define ordering. Preserve declaration order for children, scopes, properties, and projections.

A readable plain-text format is sufficient. JSON is optional, but if added, plain text should remain available because the goal is to quickly inspect the model while developing and reviewing PRs.

Normal inspect output should emphasize source-authored facts. Debug inspect output should additionally show defaulted query/projection values.

The output must include sections for:

1. publication summary
2. node tree rooted at `index.typ`
3. node details
4. scopes
5. derived spines
6. properties
7. projections and attached queries
8. parse warnings, including unreachable `.typ` files
9. validation result

Minimum useful normal output shape:

```text
Publication
  root: index.typ
  nodes: 8
  scopes: ...
  properties: ...
  projections: ...
  warnings: 1

Tree
  index.typ
    writing.typ
      writing/blog-1.typ
      writing/blog-2.typ
    thesis/intro.typ
      thesis/ch-1.typ
      thesis/bib.typ
    cv.typ

Node index.typ
  parsed-from: examples/api_sketch_site/index.typ
  properties:
    title = "My Publication" implicit/source-heading
    source-path = "index.typ" implicit
    nav.suppressed = true explicit
  scopes declared:
    nav name="nav" extent=descendants
  spine:
    global(index.typ)
    current-page(index.typ)
    nav(index.typ)
  projections:
    outline depth=1
    nav suppressed=true reason=property(nav.suppressed)
```

The exact text can differ, but the information must be present and stable enough for snapshot testing.

## Evidence and review requirements

This milestone will be coded by autonomous agents and reviewed by a human. The PR must optimize for reviewability, not apparent completeness.

Required durable evidence in the repository:

1. `docs/model-map.typ`, mapping each required model term to the Rust type or function that implements it.
2. The `insta` snapshot or checked fixture output for `cargo run --example api_sketch`.
3. The Typst fixture files used by the example.

Required evidence in the PR description:

1. The exact command output, or a link to the checked snapshot, for `cargo run --example api_sketch`.
2. A short traceability table mapping each important API sketch call to the parsed entity it creates.
3. A list of unsupported Typst/API constructs encountered in the fixture, if any, and how they are represented.
4. A concise explanation of each dependency added. The Typst parser dependency and `insta` are expected; any additional dependency must justify itself.
5. A statement of what is intentionally not implemented yet, especially query evaluation, reference resolution, rendering, and final CLI shape.
6. A statement confirming that no speculative renderer, registry, plugin system, full query engine, or generalized extension framework was introduced.

Evidence should be generated from the implementation where practical. Avoid manually written claims that cannot be checked against code, tests, snapshots, fixture files, or inspect output.

## Test strategy

Tests should be few, meaningful, and easy to audit.

Avoid dozens of small tests that merely lock in implementation details. Each test should correspond to a core model invariant or a review-critical behavior.

Use `insta` for the inspect snapshot unless a simpler checked-output approach is clearly better.

Required tests:

1. `parse_api_sketch_fixture_builds_expected_publication`
   - Parses the real fixture.
   - Asserts the expected node set, child tree, explicit scopes, major properties, major projections, and unreachable `.typ` warning exist.

2. `current_page_scopes_do_not_flow_to_children`
   - Proves each node has its own constrained current-page scope.
   - Proves a child does not inherit the parent’s current-page scope.

3. `spines_are_ordered_global_to_nearest`
   - Uses a representative nested node.
   - Proves active scopes are ordered as the model requires, including current-page before same-node declared scopes.

4. `properties_are_node_owned_and_scope_interpreted`
   - Proves properties are stored on nodes.
   - Proves scope context is derived through the node’s spine rather than by storing properties directly on scopes.
   - Includes nav suppression as a property consumed by projection behavior.

5. `inspect_api_sketch_snapshot_is_stable`
   - Checks the full inspect output or a deliberately compact snapshot.
   - Ensures a reviewer can see the model produced by the parser.

These five tests should be enough for the milestone. Add another test only if it protects a materially different invariant.

## Acceptance criteria

The milestone is complete when all of the following are true:

- `Cargo.toml` includes the Typst parser dependency required to parse Typst source files.
- `Cargo.toml` includes `insta` for snapshot evidence, unless the PR justifies an alternative.
- Any dependency beyond Typst and `insta` has a clear, written justification.
- `cargo test` passes.
- `cargo run --example api_sketch` runs successfully.
- `cargo run --example api_sketch -- --debug-defaults` or the documented equivalent shows materialized defaults.
- The example reads real Typst files from the fixture directory.
- The parser extracts only exact supported publication API calls needed by the milestone.
- The parser does not support publisher aliases or variable-bound scope calls.
- The example output includes all required reachable nodes from the API sketch.
- The example output includes the declared child tree rooted at `index.typ`.
- The example output warns about unreachable `.typ` files.
- The example output includes implicit global and current-page scopes.
- The example output includes explicit `nav`, `outline`, `reference`, and `bibliography` scopes from the sketch.
- The example output includes derived spines for every reachable node.
- Spine order follows `global -> inherited explicit scopes -> current-page(node) -> same-node explicit scopes in source order`.
- The example output includes node-owned properties, including title/source-path/citation/reference-relevant facts and nav suppression.
- The example output includes outline, bibliography, navigation/reference-related projections or query records.
- The example output shows nav suppression as projection behavior driven by node-owned properties.
- The example output includes parse warnings or validation failures when applicable, and no validation failures for the valid reachable fixture.
- `docs/model-map.typ` exists and maps model/API requirements to implementation locations.
- The implementation contains no speculative renderer, plugin registry, generalized extension framework, or full query engine.
- The public Rust parsing and model code is simple enough that the fixture and inspect output can be reviewed directly against `docs/api-sketch.typ` and `docs/model.typ`.

## Suggested implementation shape

Keep the first pass small:

```text
src/
  lib.rs
  model.rs
  parse.rs
  inspect.rs
  validate.rs
docs/
  model-map.typ
examples/
  api_sketch.rs
  api_sketch_site/
    index.typ
    writing.typ
    writing/
      blog-1.typ
      blog-2.typ
    thesis/
      intro.typ
      ch-1.typ
      bib.typ
      bibliography.bib
    cv.typ
    works.yml
    README.md
```

`model.rs` should define the core data types.

`parse.rs` should parse Typst syntax and extract only the supported publication facts.

`validate.rs` should perform simple structural checks.

`inspect.rs` should format the model from structured data.

`examples/api_sketch.rs` should load the fixture, parse it with `index.typ` as the explicit root, validate it, and print inspection output.

`docs/model-map.typ` should be short and review-oriented. It should map docs concepts to implementation locations, not narrate implementation history.

Keep modules small and direct. Do not create architecture for future renderers, registries, or plugin systems in this milestone.

## Open decisions

Unsupported Typst expressions inside recognized publisher calls need a final policy after the first parser pass. The provisional policy is to preserve raw Typst where it does not block model extraction, warn when useful for review, and error only when the model entity cannot be identified.

The exact debug flag name for showing materialized defaults may be chosen during implementation, but it must be documented in the example, PR body, or `docs/model-map.typ`.

## PR checklist

### Docs and fixture alignment

- [ ] Correct obvious small inconsistencies in `docs/api-sketch.typ` without changing the intended model.
- [ ] Add `docs/model-map.typ`.
- [ ] Add real Typst fixture files matching the corrected API sketch.
- [ ] Include at least one unreachable `.typ` file to demonstrate warning behavior.

### Dependencies

- [ ] Add the Typst parser dependency.
- [ ] Add `insta` for snapshot evidence, or justify an alternative.
- [ ] Justify every dependency beyond Typst and `insta`.

### Parsing

- [ ] Parse reachable `.typ` files from the fixture using `index.typ` as the explicit root.
- [ ] Warn about unreachable `.typ` files.
- [ ] Extract source-path and title-like properties using the first highest-level heading rule.
- [ ] Extract exact `publisher.child` declarations.
- [ ] Extract exact `publisher.children` declarations with deterministic glob expansion.
- [ ] Extract exact `publisher.scope` declarations.
- [ ] Extract exact `publisher.outline` projections.
- [ ] Extract exact `publisher.bibliography` projections.
- [ ] Extract exact `publisher.ref` projections or inline query records.
- [ ] Extract explicit nav suppression calls/properties.
- [ ] Extract simple citation markers.
- [ ] Preserve unsupported-but-relevant Typst expressions as inspectable raw expressions where needed.
- [ ] Do not support aliases such as `#let p = publisher` / `#p.child(...)`.
- [ ] Do not support variable-bound scope calls such as `#let nav = publisher.scope(...)` / `#nav.suppress()`.

### Model

- [ ] Implement `Publication`.
- [ ] Implement explicit root node semantics.
- [ ] Implement `Node`.
- [ ] Implement `Scope`.
- [ ] Implement implicit global scope rooted at the explicit root node.
- [ ] Implement per-node current-page scopes.
- [ ] Implement child tree and single-parent validation.
- [ ] Implement derived active scope spines.
- [ ] Implement same-node scope ordering as current-page first, then declared scopes in source order.
- [ ] Implement node-owned properties.
- [ ] Implement nav suppression as a node property consumed by projection behavior.
- [ ] Implement query structures sufficient for this milestone.
- [ ] Implement projection structures sufficient for this milestone.
- [ ] Track whether query/projection values are explicit or defaulted.

### Inspect and validation

- [ ] Implement structural validation with clear errors.
- [ ] Implement deterministic plain-text inspect output.
- [ ] Include publication summary in inspect output.
- [ ] Include node tree in inspect output.
- [ ] Include node details in inspect output.
- [ ] Include scopes in inspect output.
- [ ] Include derived spines in inspect output.
- [ ] Include properties in inspect output.
- [ ] Include projections and attached queries in inspect output.
- [ ] Include nav suppression behavior in inspect output.
- [ ] Include parse warnings and validation result in inspect output.
- [ ] Add a debug/defaults inspect mode and document how to run it.

### Example and evidence

- [ ] Add `examples/api_sketch.rs`.
- [ ] Ensure `cargo run --example api_sketch` parses the fixture and prints inspect output.
- [ ] Ensure `cargo run --example api_sketch -- --debug-defaults` or documented equivalent prints materialized defaults.
- [ ] Add the five required high-value tests.
- [ ] Ensure `cargo test` passes.
- [ ] Include inspect output or `insta` snapshot in the PR.
- [ ] Include traceability from model docs to Rust types/functions in `docs/model-map.typ`.
- [ ] Include traceability from API sketch calls to parsed entities in the PR body.
- [ ] Explain unsupported constructs and deliberate deferrals in the PR.
- [ ] Keep the PR small enough for line-by-line human review.
- [ ] Avoid speculative renderers, registries, plugin systems, full query engines, or generalized extension architecture.

