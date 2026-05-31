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

A scope may also be rendered as a standalone compilable unit when an edition needs it.
That standalone rendering is a projection of the scoped region, not the definition of scope itself.
Discovery should cover the transformations required for scoped compilation, including heading promotion, local outline behavior, bibliography behavior, counters, labels, and references.

== Source document semantics

`Node` should stop being core domain language for this milestone.
The domain should prefer Typst-native language: a publication contains Typst content, scopes mark meaningful regions of that content, and editions render either the whole publication or selected scopes.

The implementation may still track source Typst documents as provenance and routing units.
Source documents are useful for diagnostics, imports, default route structure, and edition assembly, but they should not force users to think in a parallel publisher document tree.

For the HTML edition, preserve the invariant that one HTML document corresponds to one source Typst document.
Scoped rendering can be added as an optional projection or edition mode, but it should not erase the source-document-to-HTML-document mapping.
