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
If a nested reachable source document declares a scope of the same kind, that nested scope shadows the inherited scope for that kind.

A scope may also be rendered as a standalone compilable unit when an edition needs it.
That standalone rendering is a projection of the scoped region, not the definition of scope itself.
Discovery should cover the transformations required for scoped compilation, including heading promotion, local outline behavior, bibliography behavior, counters, labels, and references.

Discovery should make scope extent inspectable and diagnosable.
Authors should be able to see exactly which source documents a scope covers, why a document is included or excluded, and where same-kind shadowing changes the active scope.
Scope names should default from the declaring source document's first suitable heading, with an explicit name available as an override.

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

Candidate authoring shape:

```typ
#import "/publisher.typ": scope, publish

= My Publication

Hello, welcome to my publication!

#publish("writing.typ")
#publish("thesis/intro.typ")
#publish("cv.typ")

#scope(kind: "nav")
```

In this shape, `index.typ`, `writing.typ`, `thesis/intro.typ`, and `cv.typ` remain separate source documents for HTML routing.
Typst-native selection should still be used within the relevant publisher scope.
