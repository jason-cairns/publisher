# PRD: Milestone 4 — `published.children()` as a true Typst selector, and AST-based source transforms

## Source snapshot

This PRD is based on the root Rust checkout at commit `f0b011a99a35410d29312ca49e706b992797435f`.

Read inputs:

- `design/prd-002-rendering.md` (rendering milestone that introduced the render path being changed)
- `design/prd-003-implementation-slices.typ` and `design/prd-003.1-implementation.typ` (scoped-rendering work that produced the current outline/scope machinery)
- `docs/implementation-notes.typ` (running log of what was tried and what worked; append to it per `AGENTS.md`)
- `README.md` (author-facing API and "how rendering works")
- `typst/lib.typ`, `typst/published.typ`, `typst/typst.toml` (the `@local/publisher:0.1.0` package source)
- `scripts/install-typst-package.sh`, `Makefile` (how the package is installed for consumers)
- `src/parser.rs`, `src/parser/markers.rs`, `src/parser/calls.rs`, `src/parser/scopes.rs`, `src/parser/world.rs`
- `src/model.rs` (`ScopePayload`, `Publication::payloads_for`)
- `src/render.rs` (the renderer and all the substring scanners being removed)
- `tests/render_assembly.rs`, `tests/model.rs`

This PRD is intended to be self-contained for implementation. The files above explain provenance, but the contract below restates the failing behavior, the verified Typst facts, the target architecture, the exact functions to delete/add, and acceptance criteria needed for the milestone. **No implementation has been started for this PRD** beyond an unrelated stopgap on a downstream consumer (see "Downstream consumer", below).

## Problem

Selecting published children is implemented with a fragile, multi-layer hack, and it breaks in normal use.

Concretely, a downstream site (`~/Development/jason.cair.nz/index.typ`) that wrote:

```typ
#import "@local/publisher:0.1.0": publish, published, scope
#title(outline(depth: 1, target: published.children()))
```

fails its build with:

```
index.typ: Typst marker evaluation failed: expected label, function, location, or selector, found dictionary
```

The failure is the direct, predictable consequence of the current mechanism:

1. **Magic dictionary, not a selector.** `typst/published.typ` defines `children()` to return a plain dict `(kind: "publisher-selector", selector: "children")`. That dict is not a Typst `selector`, so any real `outline(target: ...)` call rejects it.
2. **Name-shadowing gamble.** The dict only acquires meaning if `typst/lib.typ`'s **shadowing** `#let outline(..args)` (lib.typ:44–60) intercepts the call, sniffs `type(selector) == dictionary and selector.kind == "publisher-selector"`, and emits a `#metadata(..) <publisher-marker>` payload instead of calling builtin outline. If the author does not import publisher's `outline` (as the site did not — it imported only `publish, published, scope`), the dict falls through to **builtin** `outline` and produces the cryptic error above. Inside `#title(...)`, or anywhere the import is missing, it silently misbehaves.
3. **Stringly-typed selector.** When the marker path does fire, `src/parser/markers.rs` flattens the "selector" to `value: "children"` (a bare string) inside `PublisherMarker::ScopePayload`. A selector is reduced to a magic string passed through three layers (Typst → metadata → Rust).
4. **Substring scanning in the renderer.** At render time, `src/render.rs` finds inline `#outline(published.children())` calls with hand-rolled lexical scanners (`outline_call_at`, `call_open_paren`, `find_call_end`, `publisher_children_outline_depth`, `outline_depth`) and rewrites them by string surgery. The same scanner family (`bibliography_call_at`/`strip_bibliography_calls`, `bound_outlines`/`bound_outline_args`/`top_level_comma`, `replace_references`/`take_label`) drives every other source transform. `docs/implementation-notes.typ` already records the project's own conclusion that "string-stitching should be treated as the wrong implementation model, not merely a less preferred one."

This PRD removes all four layers of the hack.

## Goal

1. Make `published.children()` a **true Typst `selector`** value, so authors write standard builtin Typst:

   ```typ
   #import "@local/publisher:0.1.0": publish, published, scope
   #outline(target: published.children(), depth: 1)
   ```

   with **no** shadowed `outline`, **no** magic dict, and **no** required publisher-specific import beyond `published`.
2. Replace **all** substring-scanning source transforms in `src/render.rs` (bibliography strip, outline scope-bounding, reference rewrite, child-outline handling) with `typst_syntax` **AST/span-based** rewriting.

Non-goals are listed below. The author-facing inline API is the hard requirement; the renderer is permitted exactly one irreducible materialization point for HTML cross-page links, documented under "Verified Typst facts".

## Non-goals

- Changing the PDF/HTML export pipeline, the `RenderWorld` overlay model, or the `build_pdf_entrypoint` scope-bracket scheme. Those are reused.
- Changing the parser's marker→`ScopePayload` data path. The `scope(includes:)` channel keeps using a payload marker (see Architecture (a)); only the *inline* call becomes a true selector.
- Nested multi-level outlines. `depth: 1` is the only depth in current use; `depth > 1` is allowed to render flat (see "Verified Typst facts"). Do not build nested `<ol>` trees unless a new fixture demands it.
- A CLI redesign, query-engine work, or fixture redesign beyond what these changes require.

## Verified Typst facts (typst 0.14.2)

These were confirmed by compiling minimal examples in **both** `--format pdf` and `--features html --format html`. They are the load-bearing constraints; re-verify with `typst compile` if anything seems off.

- **A `figure.where(kind: "publisher-child")` is a genuine `selector`.** So `#let children() = figure.where(kind: "publisher-child")` makes `#outline(target: published.children(), depth: 1)` an ordinary builtin-outline call over a true selector. No shadow, no dict.
- **`#metadata(..) <label>` cannot be outlined** — `error: cannot outline metadata`. The per-child carrier the renderer injects therefore **must be a real outlinable element**; a hidden `figure` works, bare metadata does not.
- **A `figure` carrier does not leak** into an author's `#outline(target: heading.where(level: 1))` — different element kind, so the two outlines stay independent.
- **HTML external links require a `show outline.entry` rule.** Builtin outline's *default* entry links to an in-document fragment (e.g. `<a href="#loc-1">`), not to `writing.html`. A rule that emits the carrier's `caption.body` (where the caption body is `#link("writing.html")[Writing]`) produces the clean `<a href="writing.html">Writing</a>`.
- **`#show figure.where(kind: "publisher-child"): none`** hides the visible figure while the outline still lists it. `hide(..)` and `place(..)` drop the element from introspection — do **not** use them.
- **`depth > 1` does not nest figure entries** (all carriers are level 1). Flat-at-depth is acceptable for this milestone.

### The one irreducible materialization point

"Pure builtin outline with zero renderer-generated markup" is **impossible for HTML cross-page links**: separate pages compile independently, and only a `show outline.entry` rule can redirect an entry to an external href. Therefore the author-facing API is a true selector + builtin outline (goal met), but the **renderer must inject, per child with a children-outline, one hidden `figure` carrier plus a non-author-facing show-rule prelude.** This is the single sanctioned place where the renderer materializes outline structure. Everything else uses the true selector.

## Architecture

### (a) Typst package — `typst/published.typ`, `typst/lib.typ`

- `typst/published.typ`: replace the magic-dict body with the true selector:

  ```typ
  #let children() = figure.where(kind: "publisher-child")
  ```

- `typst/lib.typ`: **delete** `#let builtin-outline = outline` (line 1) and the entire shadow `#let outline(..args) = { ... }` (lines 44–60). Keep `marker`, `emit-include`, `emit-includes`, `scope`, `publish`, `css`, `in-scope`.
- `scope(includes:)` channel: compiled outline **content** cannot round-trip through a string marker, so the `includes:` list keeps a declarative marker function (not a dict, not a shadow):

  ```typ
  #let child-outline(depth: none) = marker(
    "payload",
    fields: (payload_kind: "outline", value: "children", depth: depth),
  )
  ```

  Authors then write `scope("home", includes: (child-outline(depth: 1),))`. This reuses the existing `ScopePayload { kind: "outline", value: "children", depth }` parser path with **zero parser changes**. Document the known asymmetry: the **inline** body uses the true-selector call `outline(target: published.children(), depth: n)`; the **includes** list uses `child-outline(depth: n)`. Both produce the same rendered nav.
- After editing the package, **reinstall** it so consumers pick up the change: `make install-typst` (or `./scripts/install-typst-package.sh`). The installed copy at `~/Library/Application Support/typst/packages/local/publisher/0.1.0/` is what `@local/publisher:0.1.0` imports resolve to; the source `typst/` directory is not used directly by consumers.

### (b) Renderer injection — `src/render.rs`

When (and only when) a node has a children-outline to render (inline call present, or an `"outline"` payload for the node's scope spine, and the relevant node has ≥1 child), inject:

- A renderer-owned **prelude** (both editions):

  ```typ
  #show figure.where(kind: "publisher-child"): none
  #show outline.entry: it => {
    if it.element.func() == figure and it.element.kind == "publisher-child" {
      it.element.caption.body
    } else { it }
  }
  ```

- A per-child **carrier**, injected before the children-outline:
  - **HTML:** `#figure([], kind: "publisher-child", supplement: none, caption: [#link("<relative-route>")[<title>]])`, where `<relative-route>` comes from the existing `relative_route(&origin.html_route, &child.html_route)` helper and `<title>` from the existing `outline_node_label` / `title_for_node`.
  - **PDF:** the same figure with a plain (non-link) caption, injected **inside** the existing `<scope-start-LABEL>` / `<scope-end-LABEL>` bracket that `build_pdf_entrypoint` (render.rs:496–531) already emits before each `#include`. The bounded selector already lists in-assembly content; the figure carrier gives the children-outline its entries with real page numbers.

Keep wrapping the rendered children-outline region in the `class="publisher-outline"` container so existing HTML assertions hold. The author's inline `#outline(target: published.children(), depth: n)` call is **left intact** in the transformed source — with the carriers + prelude present it is valid builtin Typst that resolves correctly. Guard injection on child count `> 0` so empty children emit nothing (current behavior).

### (c) Replace all substring scanners with AST/span rewriting — `src/render.rs`

`typst_syntax` is already a dependency (`src/parser.rs` uses `typst_syntax::parse`). Use one span-based rewriter helper for every transform:

- Build `typst_syntax::Source::new(id, text)`. Walk the tree with `LinkedNode::new(source.root())`; each node's `.range()` gives a byte `Range<usize>` into the source (and `source.range(span)` maps a `Span` to bytes). Collect a `Vec<(Range<usize>, String)>` of edits, then apply them **back-to-front** so earlier offsets stay valid.
- **Bibliography strip:** match `ast::FuncCall` whose callee is the ident `bibliography`; replace its range with empty. Replaces `strip_bibliography_calls` + `bibliography_call_at`.
- **Reference rewrite:** match `ast::Ref` (use `.target()` for the label); replace its range with the computed `#link(..)`/display text via the existing closures. Replaces `replace_references` + `take_label`.
- **Outline scope-bounding (PDF):** match `ast::FuncCall` whose callee is `outline`; read/edit the `target:` named argument via `ast::Args`; wrap the target expression with `.after(<scope-start-LABEL>, inclusive: false).before(<scope-end-LABEL>, inclusive: false)` (the existing string in `bound_outline_args`, render.rs:646–662, but built from AST spans rather than `top_level_comma` splitting). Replaces `bound_outlines` + `bound_outline_args` + `top_level_comma`.

**Delete** once nothing references them: `outline_call_at`, `bibliography_call_at`, `call_open_paren`, `find_call_end`, `top_level_comma`, `take_label`, `is_label_char` (the render.rs copy), `replace_publisher_outline_calls`, `publisher_children_outline_depth`, `outline_depth`, `published_children_outline_source`, `published_children_outline_list`, and the `OutlineSourceKind` enum. Let the compiler enumerate dead references (the same bottom-up deletion approach `docs/implementation-notes.typ` records for the legacy-strip slice).

**Stretch (same fragility class):** `src/parser/world.rs::sanitize_for_marker_evaluation` is also a hand-rolled char scanner (it rewrites `@label` shorthand to `[]` so marker evaluation compiles). Convert it to an AST/span pass for consistency if low-risk; otherwise leave it with a `// TODO` note referencing this PRD. Not required for acceptance.

## Files to change

- `typst/published.typ`, `typst/lib.typ` (+ `make install-typst`) — true selector; remove shadow; add `child-outline`.
- `src/render.rs` — prelude + per-child carrier injection (HTML + PDF); AST rewrite of all transforms; delete the scanner family and generated-markup functions.
- `src/parser/world.rs` — optional AST sanitize (stretch).
- `tests/render_assembly.rs` — rewrite the fixture in `published_children_outline_renders_inline_and_as_scope_nav` (lines ~321–389) to use the true-selector inline call + `child-outline(depth: 1)` in `includes:`; `pdf_outline_is_bounded_to_active_scope` (lines ~585–601) should still pass since the scope-bracket scheme is reused. Update the shared `fixture()` helper if it imports/uses `outline` from the package.
- `docs/api-sketch.typ`, `examples/discovery_site/*.typ` (esp. `index.typ` and the local `publisher.typ` stub), `README.md` — update the documented API to the true-selector inline call and `child-outline` for includes.
- `docs/implementation-notes.typ` — append findings (per `AGENTS.md`: record what was tried and the correct approach).

## Downstream consumer (out of this repo, do last)

`~/Development/jason.cair.nz/index.typ` consumes `@local/publisher:0.1.0`. A **stopgap** was applied to make its `make build` pass against the *current* shadowed-outline package: it now imports `outline` and calls `#title(outline(published.children(), depth: 1))`. Once this PRD ships and the package is reinstalled, **revert the stopgap** to the final API:

```typ
#import "@local/publisher:0.1.0": publish, published, scope
#title(outline(target: published.children(), depth: 1))
```

(`title` is a real Typst 0.14.2 builtin — confirmed — so the `#title(...)` wrapper is fine.) Then run `make build` there and confirm `build/index.html`'s outline links to `writing.html` and `cv.html`, and `build/index.pdf` lists the children.

## Suggested implementation slices

Keep the work atomic (per `AGENTS.md`):

0. PRD/docs only (this file) + record empirical findings in `docs/implementation-notes.typ`.
1. Add the span-rewriter helper; convert **bibliography strip** to AST. `cargo test`.
2. Convert **reference rewrite** to AST. `cargo test`.
3. Convert **outline scope-bounding** to AST; delete `top_level_comma` and the scanner helpers it used. `cargo test`.
4. Package: true-selector `published.children()`, remove shadow `outline`, add `child-outline`; `make install-typst`. Verify with a `/tmp` `typst compile` probe in both formats.
5. Renderer prelude + per-child carrier injection (HTML + PDF); delete generated-markup functions and `OutlineSourceKind`; update `tests/render_assembly.rs` fixtures. `cargo test` + `cargo run -- render` on `examples/discovery_site` (html + pdf).
6. Docs/examples/README; then revert/repoint the downstream site and run its `make build`.

Do not combine the AST-transform slices (1–3) with the package/selector change (4–5) in a single commit; they are independently verifiable.

## Acceptance criteria

- `cargo test` is green, including a rewritten `published_children_outline_renders_inline_and_as_scope_nav` that uses the true-selector inline call and `child-outline(depth: 1)` in `includes:`, and the still-passing `pdf_outline_is_bounded_to_active_scope`.
- `src/render.rs` contains **no** substring-scanning call finders: `outline_call_at`, `call_open_paren`, `find_call_end`, `top_level_comma`, `take_label`, `bibliography_call_at`, `replace_publisher_outline_calls`, `publisher_children_outline_depth`, `outline_depth`, `published_children_outline_source`, `published_children_outline_list`, and `OutlineSourceKind` are gone. All source transforms go through the AST/span rewriter.
- `typst/lib.typ` defines **no** `outline`; `typst/published.typ`'s `children()` returns a true `selector` (`figure.where(kind: "publisher-child")`).
- A `/tmp` probe compiling `#outline(target: published.children(), depth: 1)` against the reinstalled package, with the renderer's injected carriers + prelude, produces external `<a href="...">` links in HTML and a page-numbered list in PDF.
- `cargo run -- render --root examples/discovery_site/index.typ` regenerates html + pdf; the children-outline links resolve across pages.
- End-to-end downstream: after reverting the stopgap, `cd ~/Development/jason.cair.nz && make build` succeeds and `build/index.html` links to `writing.html` and `cv.html`.

## Tradeoffs (honest)

- The true-selector directive is fully met on the **inline** path. The `scope(includes:)` path keeps a declarative `child-outline()` marker because compiled outline content cannot pass through a string marker.
- HTML external links require the renderer-injected `figure` carriers + `outline.entry` show rule. This is unavoidable given Typst's per-page HTML model, but it is **not author-facing**.
- `depth > 1` renders a flat list; current usage is `depth: 1`.
- Empty children emit nothing (guarded).
