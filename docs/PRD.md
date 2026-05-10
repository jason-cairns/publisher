# cair.nz PRD

## Problem Statement

Jason wants `cair.nz` to be a personal static site authored primarily in Typst. The same source material should publish as semantic static HTML and as a unified PDF. The system should stay aggressively small, plain-text-oriented, JavaScript-free by default, and easy to reason about.

The immediate problem is to prove the publishing primitive before style, recommendations, thesis migration, or other richer sections: standalone Typst publications must compile independently, become routable HTML pages, participate in a unified PDF, and belong to an explicit Typst-authored ownership tree.

## Solution

Build a minimal static publishing pipeline:

1. Typst compiles standalone publications to intermediate HTML.
2. Typst compiles the site to a unified PDF.
3. Scryer Prolog transforms intermediate HTML into final site HTML.

Publications are discovered from explicit Typst ownership edges. A publication
owns child publications through `#publish` and `#entry`. Filesystem paths map
owned publications to routes, but they do not decide whether a file is
published. A publication's `#publish` entries define an ordered publication
index that is inherited by descendants and rendered as plain HTML `<nav>`
blocks.
`#entry` owns a child publication anonymously, without adding an inherited
index item. Publication titles derive from the first level-1 heading in
generated HTML.

The Prolog transformer accepts input and output directories at runtime. It does not parse Typst source.
The transformer also writes the generated Typst assembly source for the
unified PDF from the same ownership-derived rendered set used for final HTML.

## User Stories

1. As Jason, I want to author the site in Typst, so that HTML and PDF share a source of truth.
2. As Jason, I want each routable page to be a standalone Typst publication, so that pages remain independently understandable.
3. As Jason, I want Typst to produce intermediate HTML, so that the project does not need a custom renderer.
4. As Jason, I want Scryer Prolog to transform generated HTML, so that the site assembler stays small and testable.
5. As Jason, I want Prolog to accept input and output directories, so that it is not tied to one repository layout.
6. As Jason, I want Typst-authored ownership edges to define publication structure, so that ownership is explicit without a metadata registry.
7. As Jason, I want filesystem paths to map owned publications to routes, so that there is no route registry.
8. As Jason, I want root `#publish` declarations to form the site-level publication index, so that indexed ownership is explicit in Typst.
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
- Typst-authored ownership defines publications.
- Filesystem paths map owned publications to routes.
- The ownership tree defines the rendered set: candidates reachable from the root via ownership edges become publications. Candidates outside the tree are dropped silently and produce no build error (ADR 0006).
- Underscore Typst documents are helpers by convention; the build skips them at discovery, but the convention is not load-bearing.
- Publications compile independently.
- `#publish` creates publication ownership and a visible ordered publication index item.
- `#entry` creates publication ownership without a visible publication index item.
- publication links are references only.
- Titles derive from first level-1 headings in generated HTML.
- Slice 1 supports publication-level links only.
- JavaScript, YAML, frontmatter, route registries, and frameworks are out by default.

Deep modules:

- Ownership graph discovery.
- Ownership graph validation.
- Route resolution.
- Publication index synthesis.
- Link rewriting.
- HTML transformation.
- PDF assembly.
- Build orchestration.
- Test harness.

Each module must include automated tests when created.

## Testing Decisions

Tests should verify external behavior, not implementation details.

Required coverage:

- ownership graph discovery
- ownership graph validation
- route derivation
- publication index generation
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

## Further Notes

The first implementation should be a tracer bullet through the whole publishing system, not a horizontal infrastructure layer. Keep the output visually plain until the semantic pipeline is correct.
