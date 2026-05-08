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
3. Filesystem structure is publication structure.
4. Every routable page is a standalone publication.
5. Metadata is derived, not declared.
6. Navigation is filesystem-derived.
7. JavaScript-free by default.
8. Keep the implementation aggressively small.

## Publication Model

A publication is:
- a non-underscore `.typ` file
- independently compilable
- routable
- included in the unified PDF

Titles derive from the first level-1 Typst heading.

Routes derive from filesystem paths.

Examples:

- `about.typ` -> `/about/`
- `writing.typ` -> `/writing/`
- `thesis/ch1.typ` -> `/thesis/ch1/`

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
- route derivation
- nav synthesis
- link rewriting
- HTML normalization
- site chrome injection

## Navigation

Top-level `.typ` files become the main navigation.

Subdirectory publications are linked explicitly by parent publications.

No recursive publication semantics.

## Linking

Slice 1 supports publication-level links only.

Example:

```typst
#page("about.typ")[About]
```

Behavior:
- HTML -> `/about/`
- PDF -> anchor inside unified PDF

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
- all other `.typ` files are publications

## Testing

Test coverage should include:
- HTML generation
- PDF generation
- route correctness
- nav generation
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
- nav synthesis
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
