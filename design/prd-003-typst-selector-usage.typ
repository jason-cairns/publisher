= PRD: Milestone 3 — Revisit and simplify architecture through the use of Typst selectors

== Problem

The project has reached an impasse of complexity, and many core typst primitives are being effectively recreated.
The API has duplicated much of typst functionality, and the entities and concepts have grown excessive.

== Goal

Remove many of the core entities such as the Query, using the typst engine itself with its selector mechanism.
Maintain "scope" as the core remaining concept, and ensure that a base typst bibliography and outline renders scope-specifically.
Counters, such as chapter numbers etc., should work entirely within scope.
The base typst syntax is all that is required, with the sole addition of scope.
No hacks or duplicate parsing is to be used -- this should integrate entirely with typst

== Resolved direction

The publisher should remove `Query` as a domain concept and user-facing API.
The intended replacement is not a new publisher query language under another name.

Typst selectors and Typst's own introspection/query machinery should describe selection:
`heading.where(...)`, `figure.where(...)`, labels, locations, `query(...)`, `outline(target: ...)`, and bibliography behavior should stay as close to normal Typst as possible.

`Scope` remains publisher-specific because routed publications need boundaries that may span multiple Typst source files and editions.
Selectors run within a publisher scope; they do not replace scope.

Discovery should therefore investigate how to evaluate Typst-native selectors inside publisher-defined scopes, rather than how to evolve the current `Query` model.

== Scope semantics

A scope is primarily a region over publication content, not necessarily a separate compiled document.
Scopes should support Typst-native selection and rendering behavior within that region while preserving the surrounding publication model.

A scope declaration is authored in a source Typst document.
By default, the scope is rooted at the declaring source document and covers that document plus all source documents reachable from it through the publisher's source-document graph.
Publication scope declarations are source-document-level metadata.
They apply to the declaring source document and its published descendants regardless of textual position in the source document.
If authors need lexical sub-document regions later, that should be a separate wrapper-style feature, not the default publication scope behavior.
For this milestone, prefer at most one explicit publication scope declaration per source document.
Use scope metadata such as `tags:` for classification rather than declaring many independent scopes on the same source document.
Multiple explicit scopes in one source document can be explored later if a concrete need appears.
Scopes may overlap.
The model should not force a source document or content region into exactly one scope.
Nested scopes should overlap inherited scopes, not shadow or replace them.
Scope ids are globally unique machine identities, so same-id shadowing should not exist.

When a helper needs a single implicit scope, `current scope` should resolve to the nearest active scope unless the helper asks for an explicit scope id.
Ambiguity between unrelated active scopes is an error only when an API asks for a single implicit scope and no nearest or named rule resolves it.

Every source Typst document should also have an implicit source scope.
This is the local boundary for behavior such as a source-local bibliography, source-local outline, source-local labels, and scoped compilation of one source document.
It should not be exposed as a core domain concept called `current-page`.
Discovery should prefer source-document language such as `publisher.current-source()` or `publisher.source-scope()`.

A scope may also be rendered as a standalone compilable unit when an edition needs it.
That standalone rendering is a projection of the scoped region, not the definition of scope itself.
Discovery should cover the transformations required for scoped compilation, including heading promotion, local outline behavior, bibliography behavior, counters, labels, and references.

Discovery should make scope extent inspectable and diagnosable.
Authors should be able to see exactly which source documents a scope covers, why a document is included or excluded, and which scopes are active for each published source document.
Scope ids should be explicit and globally unique.

`#scope(...)` should not render visible content by default.
It marks a publication region and provides context for Typst-native selection and rendering behavior.
Navigation, outlines, bibliographies, queries, counters, labels, and references should remain ordinary Typst behavior wherever possible.

Any navigation helper introduced later should be explicitly a rendering helper, not a scope declaration or a hidden query system.
For example, a future `#publisher.nav(target: heading.where(level: 1))` would be a convenience renderer inside a scope, while `#scope("site-nav", tags: ("nav",))` would only mark a region with nav metadata.

Discovery should focus on a small scope-boundary adapter for Typst selectors.
The publisher may provide current-scope boundaries to Typst through labels, metadata, locations, selectors, context helpers, or another Typst-native mechanism, but selection should remain Typst-native.

Candidate shapes to investigate:

```typ
#context outline(
  target: heading.where(level: 1).and(publisher.current-scope())
)
```

or, if selector composition cannot express the boundary directly:

```typ
#context publisher.scoped[
  #outline(target: heading.where(level: 1))
]
```

The milestone should avoid introducing a replacement query API such as `publisher.query(scope: "current", select: "headings")`.

Scope categories should not be closed publisher behavior categories.
The old kinds such as `nav`, `outline`, `reference`, and `bibliography` should not drive hard-coded renderer behavior.
They may survive as ordinary author metadata or tags that Typst-native helpers can inspect.

Candidate shape:

```typ
#scope("writing", title: [Writing], tags: ("nav",))
#scope("thesis", title: [Thesis], tags: ("nav", "bibliography"))
```

In this shape, the scope id and metadata help authors and helpers identify regions, but outline, bibliography, navigation, references, counters, and labels remain Typst-native behavior inside the selected region.

== Reference semantics

The target model should not introduce scoped reference lookup.
Labels should be globally unique across a publication, and authors should use standard Typst reference syntax such as `@label` or `#ref(<label>)`.

The publisher should preserve normal Typst lookup semantics.
It should not reinterpret ordinary references as "current scope first" lookups.

The publisher may add scope-aware display context after a reference target is resolved.
When a reference crosses a meaningful named scope boundary, the rendered reference should be able to include the target's containing scope context.
For example, a blog post reference to a thesis chapter may render as `Thesis, Chapter 3`.

Lookup and display are separate concerns:

- Lookup answers which globally unique label is referenced.
- Scope context answers how that target should be described from the origin.

Inside the same meaningful scope, references should behave like normal local Typst references.
Scope context should be added when the origin and target cross a meaningful named scope boundary, not merely because the target belongs to a named scope.

A scope becomes a display context only when it has an explicit `title:` field.
Do not derive scope display titles from source-document headings.
Headings are document content; scope titles are publication metadata.
If headings implicitly created display contexts, ordinary titled content would accidentally change cross-scope reference text.

Untitled scopes are operational regions only.
They can still bound selection, counters, bibliography behavior, compilation projections, or diagnostics, but they should not contribute text to cross-scope references.

Cross-boundary references should include the nearest active explicitly titled scope for the target that is not also active at the origin.
If no such explicitly titled scope exists, standard Typst reference display should be preserved.
Use only that nearest titled target scope by default, not the full titled-scope path.
For example, a blog post reference to a thesis chapter should prefer `Thesis, Chapter 3` over `Publication, Thesis, Chapter 3`.

This keeps the authoring signal minimal and intentional:

```typ
#scope("thesis", title: [Thesis])
```

The scope id identifies the region for publisher and helper behavior.
The explicit title opts that region into display text such as `Thesis, Chapter 3` when referenced from outside the region.
Scope ids should be globally unique machine identities.
Scope titles are display text and do not need to be globally unique.
Duplicate titled scopes may be worth a diagnostic warning when they produce confusing reference text, but they should not be a model error.

Do not add separate `reference-context`, `display`, or `reference-title` controls unless discovery proves that explicit `title:` alone is insufficient.

== Bibliography semantics

Authors should use standard Typst bibliography syntax.
Do not introduce a replacement `publisher.bibliography(...)` API unless discovery proves it is necessary.

The default bibliography boundary should be the current rendered publication region supplied by the publisher.
Authors should not need to wrap every bibliography call in an explicit scoped block to get scoped behavior.

Candidate authoring shape:

```typ
#bibliography("works.yml")
```

When the publisher renders a source document, scope, or edition, it should provide the appropriate region boundary so Typst bibliography behavior is limited to that rendered region.
Discovery should investigate whether this can be achieved through scoped compilation, context, labels, metadata, selectors, or another Typst-native mechanism.

== Rendered-region defaults

Standard Typst rendering and introspection functions should operate over the current rendered publication region by default.
This applies to authored functions such as:

```typ
#outline(target: heading.where(level: 1))
#bibliography("works.yml")
#context query(figure.where(kind: image))
```

The publisher should not replace these with `publisher.outline(...)`, `publisher.bibliography(...)`, or `publisher.query(...)`.
Instead, when the publisher renders a source document, scope, or edition, it should make that rendered region the Typst context those functions naturally see.

Discovery should test whether the current scope declaration interface is sufficient for this.
The open interface question is how a non-rendering `#scope(...)` declaration becomes a Typst-visible region boundary for ordinary Typst functions without requiring authors to wrap every rendering call.

Edition rendering should be chosen by the CLI or runtime publisher invocation, not by authored source code.
The source declares publication structure and scope metadata; it should not contain an `#edition(...)` configuration API for deciding which edition is rendered.

Normal Typst functions should do the obvious thing in the current rendered region.
For example, `#bibliography(...)`, `#outline(...)`, and `query(...)` should operate over the source document, scope, or whole-publication region that the publisher is currently rendering.

When an author wants behavior across a different region, such as a bibliography or outline across many scopes, explicit scope-id contexts should be the escape hatch.
Discovery should investigate the smallest Typst-native way to express that context without introducing replacement publisher query/rendering APIs.

Candidate escape hatch:

```typ
#context publisher.in-scope("writing", "thesis")[
  #outline(target: heading.where(level: 1))
  #bibliography("works.yml")
]
```

`publisher.in-scope(...)` should be a context switch for ordinary Typst content, not a replacement query or projection API.
With one scope id, the current rendered region becomes that scope.
With multiple scope ids, the current rendered region becomes the union of those scopes.
Each explicit scope-id context should include the source document where that scope is declared, plus all covered published source documents reachable below that declaration.
Literal `#include` content belongs to the source document where it is included.
Union ordering should follow publication graph order by default, not argument order.
Overlapping regions should be de-duplicated by source/content identity.
Missing scope ids or duplicate scope ids should be diagnostics.

== Source document semantics

`Node` should stop being core domain language for this milestone.
The domain should prefer Typst-native language: a publication contains Typst content, scopes mark meaningful regions of that content, and editions render either the whole publication or selected scopes.

The implementation may still track source Typst documents as provenance and routing units.
Source documents are useful for diagnostics, imports, default route structure, and edition assembly, but they should not force users to think in a parallel publisher document tree.

For the HTML edition, preserve the invariant that one HTML document corresponds to one source Typst document.
Scoped rendering can be added as an optional projection or edition mode, but it should not erase the source-document-to-HTML-document mapping.

== Publication graph authoring

Standard Typst `#include` should remain available for literal inclusion.
The publisher should not reinterpret every include as a routed publication edge, because authors may reasonably want to inline Typst content without creating a separate HTML document or source-document boundary.

Publication graph edges therefore need an explicit publisher form.
The target API should be `#publish(...)`.
The older `#child(...)` language should not be the milestone target because it pulls the user-facing model back toward source-file nodes and trees.

The authoring distinction should be:

- `#include "summary.typ"` means literal Typst inclusion in the current source document.
- `#publish("writing.typ")` means include another source Typst document in the publication graph and render it as its own HTML document in the HTML edition.

`#publish(path, ..options)` declares a routed source document in the publication graph.
The graph can still be ordered and parented by declaration position, but the API should name the publishing outcome rather than asking authors to think in nodes.
It is not literal inclusion; use Typst `#include` when the target content should be inserted at the call site.

At its authored call site, `#publish(...)` contributes a publication edge and may render as a link, card, or entry for the target source document.
It should inherit the active scope context from the source document where it is declared.
Authors should not need to pass `scope:` to `#publish(...)` for the normal case.
The published source document should inherit all active scopes from the declaration site.
Scopes declared in the published source document or its descendants add overlapping active scopes; they do not replace inherited scopes.
Diagnostics should show the active scope stack for every published source document so authors can see why a document is inside or outside a scope.
In the HTML edition, it establishes a routed HTML document for the target source document.
In a combined PDF edition, the publisher may assemble published source documents in publication-graph order as an edition concern, independently of the visual content emitted at each `#publish(...)` call site.

Candidate authoring shape:

```typ
#import "/publisher.typ": scope, publish

= My Publication

Hello, welcome to my publication!

#publish("writing.typ")
#publish("thesis/intro.typ")
#publish("cv.typ")

#scope("home", tags: ("nav",))
```

In this shape, `index.typ`, `writing.typ`, `thesis/intro.typ`, and `cv.typ` remain separate source documents for HTML routing.
Typst-native selection should still be used within the relevant publisher scope.

== Remaining discovery defaults

Tags are metadata for this milestone.
They should support diagnostics, styling, future helpers, and author classification, but they should not drive rendered-region selection yet.
`publisher.in-scope(...)` should select by globally unique scope id only.
If tag-based selection becomes necessary, discovery should introduce a distinct API such as `publisher.in-tag("public")[...]` rather than overloading scope ids.

Scope ids should be authored as strings.
They should be globally unique within a publication and should be stable enough for CLI targeting, diagnostics, and generated artifacts.
Discovery should prefer simple ids such as `writing` and `thesis` before introducing path-like ids, namespaces, or automatic derivation.
Duplicate explicit scope ids are errors.
Duplicate display titles are allowed.

`#publish(...)` should support explicit source paths first.
Glob publishing, such as `#publish("writing/*.typ")`, is useful but should be treated as a discovery question, not assumed as part of the first implementation.
If glob publishing is added, expansion must be deterministic, diagnostics must show the expanded order, and authors must have a way to override ordering when lexical order is wrong.

Default route and output paths for HTML should derive from source Typst paths.
For example, `writing.typ` should map to `writing.html`, and `thesis/intro.typ` should map to `thesis/intro.html` unless an explicit route option is later added.
Route overrides are not required for the first selector/scope discovery spike.

Edition selection belongs to the CLI or runtime publisher invocation.
The discovery spike should prove at least these targets:

```sh
publisher render --root index.typ --to html
publisher render --root index.typ --to pdf
publisher render --root index.typ --scope thesis --to pdf
publisher inspect --root index.typ scopes
```

The exact CLI syntax may change, but the capabilities should remain:
render the whole publication, render per-source HTML, render a named scope as a compilable unit, and inspect scope/source-document relationships.

== Discovery acceptance criteria

Use a small fixture that proves the new model without reintroducing the old entities:

- `index.typ` publishes `writing.typ`, `thesis/intro.typ`, and `cv.typ`.
- `writing.typ` declares `#scope("writing", title: [Writing])` and publishes at least two writing source documents.
- `thesis/intro.typ` declares `#scope("thesis", title: [Thesis])` and publishes at least one thesis chapter.
- A normal `#outline(target: heading.where(level: 1))` renders over the current rendered region.
- A normal `#bibliography("works.yml")` renders over the current rendered region.
- `publisher.in-scope("writing", "thesis")[...]` can render an outline or bibliography over the union in publication graph order.
- A standard Typst reference from outside `thesis` to a globally unique thesis label can render with `Thesis` as scope display context.
- The HTML edition emits one HTML document per published source Typst document.
- The inspect output shows scope ids, explicit titles, tags, source-document inheritance, active scope stacks, publication graph order, routes, and diagnostics.

== Migration stance

The discovery spike should not preserve old user-facing concepts by default.
`Query`, `Projection`, and `Node` should not appear as core domain language in new API documentation.
`publisher.outline(...)`, `publisher.bibliography(...)`, and `publisher.ref(...)` should be removed from the target authoring model unless discovery proves a Typst-native approach cannot support the required behavior.
`publisher.child(...)` should be treated as old vocabulary; the target publication-edge form is `#publish(...)`.

Implementation may still use internal data structures for source documents, generated artifacts, or Typst API bridging.
Those structures should be documented as implementation machinery, not user-facing publication concepts.

== Discovery risks

The main risk is whether Typst can be made to evaluate ordinary `outline`, `bibliography`, `query`, counters, labels, and references against a publisher-supplied rendered region without rebuilding a parallel query/projection engine.
Discovery should test Typst labels, metadata, locations, selectors, context, and scoped compilation directly before committing to implementation architecture.

If Typst cannot expose the needed boundary behavior, the fallback should still preserve the authoring model as much as possible:
base Typst syntax, explicit `#scope(...)`, explicit `#publish(...)`, standard labels/references, and minimal publisher helpers only where Typst has no native hook.
