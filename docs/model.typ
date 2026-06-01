= Publication system rules

This document describes the author-facing publication model first.
Implementation terms are noted only where they are still useful for parser, inspect, or renderer code.

== Author-facing model

- A publication starts from one root Typst source document.
- A source document may publish other source documents with `#publish(path)`.
- Published source documents form an ordered source-document graph.
- A source document may include ordinary Typst support content with `#include(path)`.
- Includes are Typst content, not routed publication edges.
- A source document may declare publisher scopes with `#scope(id, title: none, tags: ())`.
- A source document may attach CSS file payloads to the nearest preceding publication scope with `#css(path)`.
- A scope covers the declaring source document and its published descendants unless a later render target narrows the rendered region.
- Scopes may overlap. Child scopes do not shadow inherited scopes.
- Every source document also has an implicit source-local boundary for source-local HTML, labels, outlines, bibliography behavior, and diagnostics.
- Active scopes for a source document are ordered from global scope to nearest scope.

== Ordinary Typst behavior

- Authors use ordinary `#outline(...)`.
- Authors use ordinary `#bibliography(...)`.
- Authors use ordinary `@label` references and labels such as `<target>`.
- Authors use ordinary counters, selectors, and `query(...)`.
- The publisher adapts generated Typst input where needed so these ordinary calls operate over the selected rendered region.
- The publisher should not expose replacement APIs such as `publisher.outline(...)`, `publisher.bibliography(...)`, `publisher.ref(...)`, or `publisher.query(...)`.

== Bibliographies

- At most one ordinary `#bibliography(...)` may appear in an active publisher scope.
- This rule includes implicit source-document scopes.
- Duplicate bibliography calls are diagnostics.
- Source-local HTML may preserve one ordinary bibliography call for that source.
- Scope or union render targets may generate a scoped bibliography input so Typst sees exactly one bibliography call for that rendered region.

== CSS

- `#css(path)` accepts a CSS file path string.
- CSS paths are relative to the source document that declares the payload.
- CSS attaches to the nearest preceding explicit publication scope in that source.
- HTML pages include applicable CSS payloads in active-scope order, outermost first and nearest last.
- CSS is scoped by page inclusion only. Selectors are ordinary CSS and are not rewritten or isolated.
- CSS is ignored by PDF rendering.

== References

- Labels are publication-global identities.
- Reference lookup is not current-scope-first.
- If a generated source contains both the reference origin and target, the publisher may preserve native Typst reference behavior.
- If source-local HTML references a label in another routed source document, the publisher lowers the reference to an explicit link/display expression.
- Hidden-including target sources to satisfy references is not allowed because it pollutes outlines, counters, bibliography inputs, and query results.
- Cross-scope reference display may include the nearest meaningful titled target scope, such as `Thesis`.

== Counters

- Render targets that contain multiple source documents should use publication graph order.
- Whole-publication, named-scope, and union entrypoints can let Typst counters continue naturally across included generated sources.
- Per-source HTML compiles each routed source independently.
- For numbered multi-source scopes, the publisher may seed heading counters at the start of a generated source-local HTML input.
- Hidden-including prior sources to advance counters is not allowed.

== Implementation vocabulary

The Rust implementation has one internal structure name worth calling out: `Node`.
It is an implementation detail, not an author-facing concept.

- `Node` means an internal record for one reachable Typst source document.
- The old `Query` and `Projection` structures and the `publisher.outline/bibliography/ref/child` APIs were removed; rendering now lowers ordinary Typst calls at the generated entrypoint instead.
- User-facing documentation should prefer source document, scope, rendered region, route, and generated entrypoint.

== Publisher output

- The HTML edition emits one HTML document per published source document by default.
- The PDF proof artifact may use a whole-publication generated entrypoint.
- Named-scope and scope-union entrypoints are generated in publication graph order.
- Generated entrypoints de-duplicate overlapping source windows.
- Generated Typst sources should remain reviewable artifacts.
