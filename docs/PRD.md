# cair.nz — Product Requirements Document

## Vision

Build a static publishing system where Typst is the canonical authoring environment and source of truth.

The system produces:
- semantic static HTML pages
- a unified PDF spanning the entire site

The architecture is intentionally minimal:
- Typst owns document semantics and rendering
- Scryer Prolog owns HTML post-processing and site composition
- Make orchestrates the build
- no JS
- no YAML/config metadata layer
- no frontend framework
- no database
- no AST parsing of Typst

## Core Principles

1. Typst is canonical.
2. HTML is an intermediate artifact.
3. Typst-authored ownership defines publication structure.
4. Every routable page is a standalone publication.
5. Metadata is derived, not declared.
6. Navigation is explicitly declared in Typst.
7. JavaScript-free by default.
8. Keep the implementation aggressively small.

## Publication Model

A publication is:
- a `.typ` file reachable from the root publication through ownership edges
- independently compilable
- routable
- included in the unified PDF

Titles derive from the first level-1 Typst heading.

Routes derive from source paths, but source paths do not decide whether a file is published.

Examples:

- `about.typ` -> `/about/`
- `writing.typ` -> `/writing/`
- `thesis/ch1.typ` -> `/thesis/ch1/`

Publication structure is a rooted ordered ownership tree:

- `#nav(target)[Label]` creates an ownership edge and a visible ordered navigation item.
- `#publish(target)` creates an ownership edge without adding a visible navigation item.
- publication links are references only; they do not create ownership and do not publish files implicitly.

The ownership graph must be acyclic. Every published page except the root must have exactly one owner.

## Build Pipeline

```text
.typ
  -> typst html
  -> prolog transform
  -> final site html
```

Typst phase responsibilities:
- HTML generation
- PDF generation
- headings/anchors
- document semantics

Prolog phase responsibilities:
- route mapping for the owned publication set
- ownership graph validation
- nav synthesis from explicit Typst markers
- link rewriting
- HTML normalization
- site chrome injection

## Navigation

The root publication's `#nav` entries become the main navigation.

Nested publications can declare their own `#nav` entries. Those entries create section navigation inherited by pages reached through that navigation edge.

`#publish` creates publication ownership without adding a visible nav item.

Ordinary publication links do not create ownership and do not affect inherited navigation context.

## Linking

Slice 1 supports publication-level links only.

Example:

```typst
#publication-link("about.typ")[About]
```

Behavior:
- HTML -> `/about/`
- PDF -> anchor inside unified PDF
- target must already be in the published ownership tree

Deep anchor linking is deferred.

## Repository Shape

```text
.
├── Makefile
├── site.pl
├── src/
├── build/
├── public/
├── test/
└── .github/workflows/
```

Rules:
- `_*.typ` are helper/import-only files
- non-helper `.typ` files must be reachable from the root ownership tree or the build fails
- filesystem structure maps routes but does not discover publications

## Testing

Test coverage should include:
- HTML generation
- PDF generation
- route correctness
- nav generation
- ownership graph validation
- link rewriting
- broken local link detection
- golden HTML snapshots
- Prolog unit tests

## Deployment

GitHub Actions should:
1. build HTML/PDF
2. run tests
3. deploy `public/` to GitHub Pages

## Deferred Work

Deferred intentionally:
- deep anchors
- thesis migration tooling
- recommendation APIs/cache
- styling system
- JS interactivity
- Typst AST parsing
- Nix integration

## Initial Vertical Slices

### Slice 1

Minimal publishing primitive:
- standalone publications
- HTML generation
- unified PDF
- Prolog transform
- explicit Typst-authored ownership graph
- nav synthesis from `#nav`
- publication-level links
- tests
- GitHub Pages deployment

### Slice 2

Deep-link support.

### Slice 3

Thesis migration prototype.

### Slice 4

Recommendations engine.

### Slice 5

Styling system.
