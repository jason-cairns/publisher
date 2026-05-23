= Publication system rules

== Core entities
- A publication is a tree of Typst source-file nodes.
- A node is one Typst source file.
- The root node is implicit and always exists.
- The root node defines the global scope implicitly.
- A node may declare additional scopes.
- A scope is a contextual region over source-file nodes.
- A scope has a name, kind, attributes, root node, and extent.
- A scope covers its root node and its child nodes recursively, unless its extent is explicitly constrained.
- Each node has its own constrained current-page scope, which child nodes don't inherit.
- A node may be within many scopes.
- The active scopes of a node are ordered from global scope to nearest scope.
- The ordered active scopes of a node form that node’s spine.
- A spine is derived. It is not separately declared.
- A node declares its children.
- A node may have only one parent.

== Properties
- A node may contribute properties.
- A property is a fact about a source-file node.
- A property has a key, value, source, and optional attributes.
- A property may be implicit, such as title, number, target, source path, or output path.
- A property may be explicit, such as a property provided by a Typst command.
- Properties do not belong to scopes directly.
- Properties belong to nodes.
- A property is interpreted with the node’s active scopes.
- Scope is context, not storage.

== Queries
- A query is a structured selection plan over the publication.
- A query has an origin node.
- A query has a search rule, filters, ordering, visibility, fallback, and ambiguity rules.
- A query may select nodes, scopes, or properties.
- A query may select by scope name, scope kind, scope attributes, node position, property key, property value, or property attributes.
- A query may be evaluated from the origin node’s spine.
- A query may search the current node, the current scope, active scopes, named scopes, or the whole publication.
- A query may search active scopes from nearest to global or from global to nearest.
- A query returns matching nodes, scopes, properties, or resolved targets.
- References, tables of contents, navigation, and bibliographies are all expressed as queries over the same publication data.

Example:

```rust
query {
  origin: "chapters/intro.typ"
  search: current-scope
  select: properties
  where: property.key == "citation"
  order: publication-tree
  visibility: visible-from-origin
  fallback: empty
  ambiguity: allow-many
}
```

=== References
- A reference is an inline query.
- A reference has an origin node and a reference value.
- A reference normally searches for a property with key `target` whose value matches the reference value.
- A reference may search the current node, current scope, nearest matching scope, active scopes, explicit named scopes, or global scope.
- A reference resolves to a target node, target property, link destination, and display text.
- Display text may combine target properties and scope context, such as `Figure 3, Chapter 2`.
- Ambiguous references are errors unless the query defines a disambiguation rule.
- Missing references are errors unless the query defines a fallback rule.

== Projections
- A projection is generated Typst-compatible output inserted into a node.
- A projection has an origin node, a query, a kind, and rendering attributes.
- A projection runs its query, transforms the result, and produces Typst-compatible content.
- A projection does not own scope.
- A projection uses scope only through its query.
- A projection may render a table of contents, navigation, bibliography, index, backlink list, or other generated structure.
- The generated output should be as close to ordinary Typst input as possible.

=== Navigation
- Navigation is a projection.
- Navigation queries title-like properties from selected source-file nodes.
- Navigation may select nodes from the current scope, nearest scope, all active scopes, named scopes, or scopes matching attributes.
- Navigation is ordered by the publication tree unless another order is specified.
- Navigation may use the origin node to determine active state.
- Stacked navigation is represented by multiple navigation projections or by one projection with multiple scope-selecting queries.

=== Table of contents
- A table of contents is a projection.
- A table of contents queries title-like properties from selected source-file nodes.
- A table of contents is ordered by the publication tree.
- A table of contents may be limited to a selected scope.
- A table of contents may include depth, numbering, title, target, and hierarchy properties.

=== Bibliography
- A bibliography is a projection.
- A citation is a property contributed by a node.
- A bibliography queries citation properties.
- A bibliography may select citations from one scope, many named scopes, all active scopes, or scopes matching attributes.
- A bibliography may be local, scoped, shared, or global depending on its query.
- Bibliography scoping is determined by the selected citation properties, not by the bibliography output node alone.
- To stay close to normal Typst behavior, the publisher should collect the selected citation keys, provide Typst with a corresponding bibliography source or subset, and let Typst render the bibliography normally.

== Publisher

- The publisher takes a publication tree of Typst source files and produces an edition, such as HTML or PDF.
- the publisher may produce an edition of only a particular scope/s or a single node as well
- The publisher extracts nodes, scopes, and properties.
- The publisher evaluates queries.
- The publisher inserts projections.
- The publisher delegates final rendering to Typst wherever possible.
- Scopes may be specified in source files or supplied as publication-tree structure to the publisher.
- Source-level Typst should remain as close to standard Typst as possible.
- Publication-specific behavior should be expressed through scopes, properties, queries, and projections rather than custom rendering logic.

== Minimum model
- Node is a Typst source file.
- Scope is context over source-file nodes.
- Property is a fact about a source-file node.
- Query is a structured selection plan.
- Projection is generated Typst-compatible output.
- Spine is derived from scopes.
- Reference is an inline query.
- Navigation, table of contents, and bibliography are projections.
- Resolvers are query evaluators.
- Projectors are projection renderers.
- Registries are implementation indexes.
- Bindings are queries attached to projections.
