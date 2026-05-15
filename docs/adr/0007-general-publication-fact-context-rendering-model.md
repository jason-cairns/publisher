# ADR 0007: General publication fact, context, and rendering model

## Status

Accepted

## Context

Multiple planned features need the same scoping and aggregation behavior:

- publication indexes/navigation
- scoped bibliographies
- scoped outlines (TOCs, figures, tables)
- publication-aware components
- parent/child metadata flows

The ownership tree is already the source of truth for publication structure. We need one model for facts and scopes that all renderers can reuse, without each feature inventing custom inheritance logic.

## Decision

Adopt a shared model with four core concepts:

- publication tree
- facts with node/source location
- contexts as inherited scoped fact spaces
- renderers as parameterized queries over one or more contexts

Compactly:

- A context is a named or anonymous inherited fact space.
- A renderer is a parameterized query + transform over one or more fact spaces.

A fact space is the scoped set of fact references available to a context membership after ownership-based scope resolution.

There is no separate collector abstraction. Fact accumulation falls out of context membership plus renderer queries: contexts hold scoped fact references, renderers query them. Naming a third role would just duplicate one of the two.

### Vocabulary

#### Fact

A fact is emitted by a publication and includes enough metadata for filtering and ordering:

- fact value
- fact category/type
- emitting publication identity
- tree location
- source/order location
- route/path metadata when relevant

Facts are attached to contexts by reference, never copied.

Because facts retain their tree and source location, location-aware filtering is a renderer concern, not a context concern. A renderer that has selected a context's fact space may further restrict to:

- all facts in the context
- facts upstream of the renderer in the ownership tree
- facts downstream of the renderer
- facts before or after the renderer in source order
- facts on the ownership path between the renderer and the root
- facts from selected branches

The context does not encode any of those filters; it only decides which facts are *in scope*. This keeps contexts a single concept rather than a family of context kinds.

#### Context

A context starts at a publication node and applies to descendants in the ownership tree.

A context can be:

- anonymous
- named

A publication can belong to many contexts at once.

If nested contexts reuse the same name, renderers that request the nearest matching named context resolve to the innermost one. Renderers may also read multiple matching contexts explicitly.

Sibling contexts never share facts unless a renderer explicitly reads both.

#### Renderer

A renderer instance is discovered in the publication tree and declares:

- which context(s) it reads
- which fact predicate it accepts
- how it transforms selected facts into output

Renderers consume scoped data and must not perform discovery walks for scoping.

The relationship between contexts and renderers is many-to-many in every direction:

- one context can feed many renderer instances
- one renderer instance can read from many contexts
- one renderer definition can be instantiated many times in the tree
- many contexts can collapse into one rendered output

Examples:

- a local table-of-contents renderer reads its nearest active context and selects heading facts
- a bibliography renderer reads named contexts `site` and `research-notes`, deduplicates citations, then renders one bibliography
- a navigation renderer reads all contexts in scope and renders one nav block per context

## Pipeline

1. Discover context start markers in the publication tree.
2. Discover renderer instances independently.
3. Resolve publication membership in contexts.
4. Resolve renderer attachment to contexts according to renderer policy (for example nearest, all active, named explicit set, or custom merge strategy).
5. Determine the fact predicates needed by renderers attached to each context.
6. Attach only matching fact references to each context membership.
7. Feed each renderer facts from its resolved contexts.
8. Renderer applies filtering/ordering/grouping/deduping and renders output.

This is intentionally multi-pass and deterministic: inherited context membership is resolved in a down-pass, fact attachment is resolved in an up-pass, then renderers transform scoped facts into output.

## Compiler model note

This design is attribute-grammar-inspired over the publication ownership tree:

- inherited attributes map to active context membership flowing downward
- synthesized attributes map to facts emitted by publications and attached upward by scope rules
- semantic actions map to renderer transforms over resolved scoped facts

In short: decorate the tree with inherited scope membership, attach synthesized fact references, then run renderer transformations.

## Scoping rules

- Ownership tree defines inheritance and scope.
- Context inheritance is explicit and deterministic.
- Renderers may attach to one or many contexts in scope.
- Sibling contexts do not leak into each other.
- Parent contexts do not implicitly consume child-scoped facts when the child establishes its own context, unless a renderer explicitly reads both scopes.
- Renderer explicit context names override implicit attachment.

## Output targets

Current implementation target is HTML output.

The model is designed so the same context/fact resolution layer can also feed Typst/PDF-oriented renderers in follow-up work.

## Consequences

### Positive

- One reusable scoping model for bibliography, outline, navigation/index, and components.
- Keeps transformer architecture small, stdlib-biased, and testable.
- Preserves Typst as source of truth and HTML as intermediate artifact.
- Public Typst API can document contexts and renderers without exposing internal marker internals.
- Follow-up issues can reference this ADR instead of redefining scoping semantics.

### Trade-offs

- Requires a clear separation between discovery and rendering passes.
- Introduces predicate planning before fact attachment to avoid collecting irrelevant facts.
- Cross-publication wording/numbering policy remains separate (JAS-40/JAS-41).

## Non-goals

- Implement bibliography, outline, or component features directly.
- Build a large framework or template language.
- Move tree discovery logic into renderers.
- Require zero dependencies under all circumstances (still prefer stdlib-first).

## Related issues

- JAS-22 / jason-cairns/my-site#27: Python CLI transformer
- JAS-23 / jason-cairns/my-site#28: citation contexts and scoped bibliographies
- JAS-24 / jason-cairns/my-site#29: outline contexts and scoped indices
- JAS-25 / jason-cairns/my-site#30: public Typst authoring API
- JAS-26 / jason-cairns/my-site#31: publication-aware components
- JAS-40 (blocks): cross-publication wording/numbering design
- JAS-44 (blocks): implement this fact/context/rendering model
