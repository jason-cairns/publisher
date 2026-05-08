# cair.nz PRD

## Problem Statement

Jason wants `cair.nz` to be a personal static site authored primarily in Typst. The same source material should publish as semantic static HTML and as a unified PDF. The system should stay aggressively small, plain-text-oriented, JavaScript-free by default, and easy to reason about.

The immediate problem is to prove the publishing primitive before style, recommendations, thesis migration, or other richer sections: standalone Typst publications must compile independently, become routable HTML pages, participate in a unified PDF, and link to each other at publication level.

## Solution

Build a minimal static publishing pipeline:

1. Typst compiles standalone publications to intermediate HTML.
2. Typst compiles the site to a unified PDF.
3. Scryer Prolog transforms intermediate HTML into final site HTML.

Publications are discovered from the filesystem. Every non-underscore Typst document is a publication. Filesystem paths define routes. Top-level publications define the main navigation. Publication titles derive from the first level-1 heading in generated HTML.

The Prolog transformer accepts input and output directories at runtime. It does not parse Typst source.

## User Stories

1. As Jason, I want to author the site in Typst, so that HTML and PDF share a source of truth.
2. As Jason, I want each routable page to be a standalone Typst publication, so that pages remain independently understandable.
3. As Jason, I want Typst to produce intermediate HTML, so that the project does not need a custom renderer.
4. As Jason, I want Scryer Prolog to transform generated HTML, so that the site assembler stays small and testable.
5. As Jason, I want Prolog to accept input and output directories, so that it is not tied to one repository layout.
6. As Jason, I want filesystem paths to define routes, so that there is no route registry.
7. As Jason, I want top-level publications to form the main nav, so that navigation follows the visible site shape.
8. As Jason, I want subdirectory publications omitted from global nav, so that chapters and entries do not flood it.
9. As Jason, I want titles derived from first level-1 headings, so that metadata stays in normal Typst.
10. As Jason, I want underscore Typst files to be helpers, so that shared functions do not become pages.
11. As Jason, I want internal publication links, so that links resolve correctly in HTML and PDF.
12. As Jason, I want deep anchors deferred, so that the first slice stays narrow.
13. As Jason, I want semantic HTML, so that the site remains accessible and inspectable.
14. As Jason, I want a unified PDF, so that the whole site can be read as one document.
15. As Jason, I want automated tests from the beginning, so that every module is safe to change.
16. As Jason, I want GitHub Pages deployment, so that successful builds publish automatically.

## Implementation Decisions

- Typst is canonical.
- Typst-generated HTML is intermediate.
- Scryer Prolog transforms HTML only.
- Prolog must not parse Typst source.
- Prolog must accept input and output directories at runtime.
- Filesystem structure defines publications and routes.
- Non-underscore Typst documents are publications.
- Underscore Typst documents are helpers.
- Publications compile independently.
- Top-level publications become main navigation.
- Titles derive from first level-1 headings in generated HTML.
- Slice 1 supports publication-level links only.
- JavaScript, YAML, frontmatter, route registries, and frameworks are out by default.

Deep modules:

- Publication discovery.
- Route resolution.
- Navigation synthesis.
- Link rewriting.
- HTML transformation.
- PDF assembly.
- Build orchestration.
- Test harness.

Each module must include automated tests when created.

## Testing Decisions

Tests should verify external behavior, not implementation details.

Required coverage:

- publication discovery
- route derivation
- navigation generation
- link rewriting
- HTML transformation
- unified PDF artifact creation
- build orchestration
- broken local link detection
- golden output snapshots

The Prolog transformer should be testable without invoking Typst by using HTML fixtures.

## Out of Scope

Out of scope for the first implementation slice:

- visual style system
- deep cross-document anchors
- thesis migration tooling
- recommendations metadata fetching
- mood board behavior
- recipes implementation
- CV styling polish
- JavaScript interactivity
- Typst source parsing
- Nix or direnv integration
- recursive publication semantics

## Further Notes

The first implementation should be a tracer bullet through the whole publishing system, not a horizontal infrastructure layer. Keep the output visually plain until the semantic pipeline is correct.