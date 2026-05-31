= PRD-003 selector/scope discovery report

== Executive summary

PRD-003 is feasible if the publisher treats a rendered region as a generated Typst compilation entrypoint.
That entrypoint may be one source document, a named scope, a union of scopes, or a whole publication edition.
Inside that generated entrypoint, ordinary Typst behavior is strong: `#outline(...)`, counters, labels, references, query/introspection, and a single `#bibliography(...)` behave over the compiled document without a publisher query language.

PRD-003 is not feasible as a metadata-only context switch inside a larger Typst compilation.
A non-rendering `#scope(...)` marker can be recovered through Typst metadata and labels, but Typst does not use that marker as a native boundary for ordinary `outline`, `bibliography`, `query`, or references.
The publisher must still build the publication graph, compute scope extents, and generate the Typst input for the region being rendered.

The smallest viable architecture is therefore:

- keep only `#scope(...)` and `#publish(...)` as publisher semantic declarations;
- recover those declarations through labeled metadata using Typst introspection;
- build source-document graph and scope extent indexes in Rust;
- generate Typst entrypoints for each rendered source, scope, union, or edition;
- let Typst handle ordinary outline, counters, bibliography, labels, references, and selectors inside each generated entrypoint;
- add small adapters for cross-source HTML references, titled-scope reference display, `publisher.in-scope(...)` region assembly, duplicate-id/title diagnostics, and inspect output.

Typst selectors can still help inside that adapter.
For contiguous windows, Typst source supports selector composition such as `selector(heading).after(<scope-start>).before(<scope-end>)`, and `#outline(target: ...)` can consume that bounded selector.
That is useful for source-window and assembly-window filtering.
It does not remove the publisher's responsibility to define scope windows, order discontiguous unions, or decide which rendered region is being compiled.

== Decision table

- Works natively:
  `#scope(...)` as hidden metadata, `#publish(...)` as hidden metadata, source-local `#outline(...)`, source-local `#bibliography(...)`, source-local `query(...)`, labels and references when origin and target are in the same compiled document, counters within the generated document, one HTML output per source document when each source is compiled separately.

- Works with a small adapter:
  source graph and scope extent recovery, route derivation, contiguous selector windows with `after(...)` and `before(...)`, `publisher.in-scope("a", "b")[...]` as generated region assembly or an explicit bounded-selector adapter, scope diagnostics, duplicate scope id errors, duplicate display-title warnings, cross-scope reference display enrichment after normal target resolution.

- Risky:
  combined or scope assemblies that preserve authored `#bibliography(...)` calls when multiple bibliography calls occur in the same generated Typst document.
  Typst 0.14.2 reports `multiple bibliographies are not yet supported`.

- Not viable as written:
  treating Typst metadata alone as a native current-region boundary for ordinary `#outline(...)`, `#bibliography(...)`, `query(...)`, and references over discontiguous publisher scopes inside one existing compilation.
  Standard references across separately compiled HTML source documents also do not work without an adapter because the target label is absent from the source-local compilation.
  Hidden-including target sources to satisfy references is not viable because the hidden headings and citations still pollute ordinary introspection and bibliography inputs.

== Experiments run

Fixture and harness:

```sh
cargo run --example prd003_discovery
```

The harness writes:

```text
target/prd-003-discovery/evidence.txt
target/prd-003-discovery/full-publication.typ
target/prd-003-discovery/scope-thesis.typ
target/prd-003-discovery/scope-writing-thesis.typ
target/prd-003-discovery/examples/prd003_discovery_site/*.html
```

Direct Typst checks:

```sh
typst compile examples/prd003_discovery_site/index.typ /private/tmp/prd003-index.pdf
typst compile --root examples/prd003_discovery_site examples/prd003_discovery_site/writing/blog-1.typ /private/tmp/prd003-blog-1.pdf
typst compile --root examples/prd003_discovery_site examples/prd003_discovery_site/thesis/intro.typ /private/tmp/prd003-thesis-intro.pdf
typst compile --features html --format html --root examples/prd003_discovery_site examples/prd003_discovery_site/index.typ /private/tmp/prd003-index.html
typst compile --features html --format html --root examples/prd003_discovery_site examples/prd003_discovery_site/thesis/intro.typ /private/tmp/prd003-thesis-intro.html
typst compile --features html --format html --root examples/prd003_discovery_site examples/prd003_discovery_site/writing/blog-1.typ /private/tmp/prd003-blog-1.html
typst compile --features html --format html --root examples/prd003_discovery_site examples/prd003_discovery_site/writing/blog-2.typ /private/tmp/prd003-blog-2.html
```

Additional throwaway selector probes under `/private/tmp`:

```sh
typst query boundary.typ 'selector(heading).after(<scope-start>, inclusive: false).before(<scope-end>, inclusive: false)' --field body
typst query metadata-where.typ '<b-heading>' --field value
typst query outline-boundary.typ 'outline.entry' --field element
typst query block-query.typ '<block-query-count>' --field value
typst compile refs/blog.typ refs/blog.pdf
```

Focused project checks:

```sh
cargo test --no-run
```

== Files created or modified

- `examples/prd003_discovery_site/`: throwaway PRD-003 fixture using `#scope(...)`, `#publish(...)`, ordinary `#outline(...)`, ordinary `#bibliography(...)`, standard references, and literal `#include`.
- `examples/prd003_discovery.rs`: rerunnable evidence harness that discovers graph metadata, generates region assemblies, compiles source-local HTML, compiles generated region HTML, and writes evidence text.
- `docs/implementation-notes.typ`: failed attempts and corrected approaches observed during discovery.
- `design/prd-003-discovery-report.typ`: this report.

== Evidence snippets

Metadata recovery:

```text
scopes:
- id=writing title=Writing tags=nav declared-in=writing.typ
- id=thesis title=Thesis tags=nav,bibliography declared-in=thesis/intro.typ
- id=home title=<none> tags=nav declared-in=index.typ
```

Publication graph:

```text
publish edges:
- index.typ -> writing.typ
- writing.typ -> writing/blog-1.typ
- writing.typ -> writing/blog-2.typ
- index.typ -> thesis/intro.typ
- thesis/intro.typ -> thesis/ch-1.typ
- index.typ -> cv.typ
```

HTML route invariant:

```text
reachable source documents:
- 1. index.typ -> index.html
- 2. writing.typ -> writing.html
- 3. writing/blog-1.typ -> writing/blog-1.html
- 4. writing/blog-2.typ -> writing/blog-2.html
- 5. thesis/intro.typ -> thesis/intro.html
- 6. thesis/ch-1.typ -> thesis/ch-1.html
- 7. cv.typ -> cv.html
```

Ordinary source-local outline and bibliography evidence:

```text
heading introspection per source:
- thesis/intro.typ: level=1 text=Thesis Introduction | level=1 text=Contents | level=1 text=Bibliography
- writing/blog-1.typ: level=1 text=Blog One | level=2 text=Blog One Detail | level=1 text=Bibliography
```

Cross-source reference limitation:

```text
source-local HTML compilation:
- failed: examples/prd003_discovery_site/writing/blog-2.typ: label `<thesis-main>` does not exist in the document
```

Multiple bibliography limitation:

```text
generated assembly HTML compilation:
- failed: full publication: target/prd-003-discovery/full-publication.typ: multiple bibliographies are not yet supported
- failed: thesis scope: target/prd-003-discovery/scope-thesis.typ: multiple bibliographies are not yet supported
```

Duplicate display-title diagnostic:

```text
duplicate titled scope warnings:
- duplicate title "Writing" used by scope ids: writing, duplicate-title-a, duplicate-title-b
```

Selector/context boundary evidence:

```text
selector(heading).after(<scope-start>).before(<scope-end>) returned only the headings inside the bounded marker window.
block/context query probe returned headings outside the block, so block/context is not a lexical introspection boundary.
hidden included reference targets resolved labels, but query(cite).len() and query(heading) included the hidden content.
```

== Answers to required questions

1. Can a source-level `#scope(...)` marker be represented through Typst metadata, labels, or locations so the publisher can recover source-document-level scope regions?

Yes.
The current marker pattern works: emit `metadata(...) <publisher-marker>`, compile, then query `PagedDocument.introspector` with `Selector::Label`.
This recovers declarations and source provenance.
Scope extent over published descendants is not native Typst state; it is publisher graph state.

2. Can ordinary `#outline(...)` render over the current rendered region without replacing it with `publisher.outline(...)`?

Yes, if the current rendered region is the generated Typst document being compiled.
No, if the expectation is a hidden metadata boundary inside a larger compiled document.

3. Can ordinary `#bibliography(...)` render over the current rendered region without replacing it with `publisher.bibliography(...)`?

Yes for a single bibliography in a source-local or generated region.
Risky for combined/scope assemblies that contain multiple authored bibliography calls, because Typst 0.14.2 rejects multiple bibliographies in one document.

4. Can ordinary `query(...)` and selectors operate inside a publisher-supplied region boundary?

Yes when the region boundary is the compiled Typst entrypoint.
The harness also uses Rust introspection with `HeadingElem::ELEM.select()` and labeled metadata.
For contiguous windows, explicit selector composition with `after(...)` and `before(...)` can bound a selector.
No pure Typst metadata/context mechanism was found that makes a discontiguous publisher region become the implicit boundary for ordinary `query(...)`.

5. Can `publisher.in-scope("a", "b")[...]` be expressed as a Typst-native context switch over a union of scope regions?

Not as a pure Typst context switch.
The viable design is a small adapter: the publisher resolves scope ids, builds the union in publication graph order, de-duplicates overlaps, then either compiles generated content where ordinary Typst functions see that generated region or generates explicit bounded selectors for content that accepts selectors.

6. Can standard Typst labels and references preserve lookup while allowing scope-aware display enrichment after target resolution?

Partly.
Standard lookup works when origin and target are present in the same compilation.
Per-source HTML cannot resolve labels in other source documents without an adapter.
Scope-aware display text such as `Thesis, Chapter 3` is publisher enrichment after target resolution; it is not native reference behavior.

7. Can source-local rendering preserve one HTML document per published source Typst document?

Yes.
The harness emits one source-local HTML result per reachable source, and the existing renderer already has this shape.
Cross-source references remain the main adapter requirement.

8. Can scope rendering as a standalone compilable unit support heading promotion, local counters, bibliography, labels, and references?

Yes in principle by generating a standalone Typst entrypoint for that scope.
Heading promotion and counter behavior should be implemented as generated Typst transforms or show/set rules at the entrypoint.
Bibliography support is constrained by the multiple-bibliography limitation if the generated scope contains multiple authored bibliography calls.
References work when targets are present in the scope compilation.

9. What exact Typst crate APIs are required?

- `typst::compile::<PagedDocument>(&world)` for marker/introspection discovery.
- `typst::compile::<typst_html::HtmlDocument>(&world)` for HTML evidence.
- `typst::layout::PagedDocument`.
- `typst::introspection::MetadataElem`.
- `typst::foundations::Selector::Label`.
- `typst::foundations::Selector::{And, Or, Before, After, Location}` when generated bounded selectors are needed.
- `typst::foundations::Label`, `PicoStr`, `Value`, `Dict`.
- `typst::foundations::NativeElement` for `.select()`.
- `typst::model::HeadingElem` and `HeadingElem::ELEM.select()`.
- `typst::foundations::StyleChain` and `HeadingElem::resolve_level(...)`.
- `typst::{Library, LibraryExt, Features, Feature::Html}` for HTML-enabled worlds.
- `typst::World`, `FileId`, `VirtualPath`, `Source`, `Bytes`, `FontBook`, and `Font` for repository-backed compilation.

10. Where does Typst resist the target model and force a custom adapter?

- Metadata does not create an implicit rendered-region boundary.
- `block` and `context` do not create a lexical introspection boundary.
- Discontiguous scope unions are not native Typst contexts.
- Multiple bibliographies in one compiled document are not supported in Typst 0.14.2.
- Standard references require labels to exist in the same compiled document.
- Hidden target inclusion pollutes introspection and bibliography inputs, so it is not a clean cross-source reference solution.
- Scope-aware reference display text is not native.
- Heading promotion through show rules is brittle; generated-source or AST transforms are cleaner.
- Inspection of scope inheritance, graph order, routes, duplicate ids, and duplicate display titles is publisher responsibility.

== Implementation recommendation

Replace the old user-facing `Query`, `Projection`, and `Node` API language with source-document graph and scope-region internals, but do not remove the need for Rust indexes.
The final implementation should be a small scope-boundary adapter around Typst, not a replacement query engine.

Recommended slices:

1. Add PRD-003 marker decoding for `scope(id, title: none, tags: ())` and `publish(path)`.
2. Build a source-document graph from `#publish(...)` only; leave `#include` literal.
3. Derive scope extents from graph descendants and inherited publication context.
4. Add inspect output for graph order, routes, active scope stacks, scope extents, duplicate ids, duplicate display titles, and missing ids.
5. Generate render entrypoints for source-local HTML, whole publication, named scope, and explicit scope union.
6. Preserve ordinary Typst calls inside generated entrypoints wherever possible, and use generated bounded selectors for contiguous windows where a Typst function accepts a selector.
7. Add narrow adapters for per-source cross-source references and titled-scope display enrichment.
8. Treat multiple authored bibliographies in one generated document as a known risk requiring either PRD revision, an authoring restriction, or a bibliography consolidation adapter.

== Risks and unresolved questions

- Multiple ordinary `#bibliography(...)` calls in one generated document are currently blocked by Typst 0.14.2.
- Cross-source HTML references need a concrete adapter design that keeps lookup standard where possible while producing correct links and display text.
- Heading promotion for scope PDFs should be specified as an entrypoint transform, including whether labels remain stable after transformed heading emission.
- `publisher.in-scope(...)` authoring syntax should be clarified: it cannot directly mutate Typst's current introspection boundary; it triggers publisher-generated region rendering.
- The PRD should define whether an authored bibliography inside a child source should be preserved, suppressed, or consolidated when rendering a parent scope.

== PRD revision recommendation

Revise PRD-003 before implementation.
Keep the desired authoring model, but clarify these points:

- "current rendered region" means the Typst document generated for the current publisher render target.
- `publisher.in-scope(...)` is a publisher adapter that generates or selects a region entrypoint; it is not a pure Typst context switch.
- Per-source HTML keeps one document per source and uses an adapter for cross-source references.
- Multiple bibliographies in one generated region are not currently native Typst behavior; choose an authoring restriction or consolidation adapter.
- Scope metadata is recoverable through Typst, but scope extent and inheritance are publisher graph diagnostics.
