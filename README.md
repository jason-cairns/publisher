# publisher

A static publisher for [Typst](https://typst.app). You write ordinary Typst;
the only additions are two markers — `#publish(...)` and `#scope(...)` — that
describe how your source documents form a publication. The publisher renders
the whole publication, or any named region of it, to HTML or PDF.

## Concepts

- **Source document** — one `.typ` file. Each becomes one HTML page.
- **`#publish("other.typ")`** — adds another source document to the publication
  graph and renders it as its own page. Glob paths such as
  `#publish("writing/*.typ")` publish matching sources in sorted order.
  (Plain `#include` stays literal: it inlines content without creating a
  separate page.)
- **`#scope("id", title: [..], tags: (..))`** — marks a region of the
  publication. A scope covers the source document that declares it and every
  document published beneath it. Scope `id`s are globally unique; `title` is
  optional display text; `tags` are metadata. Scopes overlap rather than nest.
- **Ordinary Typst stays ordinary.** `#outline(...)`, `#bibliography(...)`, and
  `@label` references are written exactly as in normal Typst. The publisher
  makes them behave region-locally before compiling — it never asks you to call
  a `publisher.outline(...)`-style replacement.

A scope can be rendered as its own document. The **HTML edition** always emits
one page per source document; the **PDF edition** assembles the selected region
into a single document.

## Authoring example

`index.typ`:

```typ
#import "@local/publisher:0.1.0": scope, publish

= My Site

#publish("writing.typ")
#publish("thesis/intro.typ")
#publish("cv.typ")

#scope("home", tags: ("nav",))
```

`thesis/intro.typ`:

```typ
#import "@local/publisher:0.1.0": scope, publish

= Thesis Introduction <thesis-intro>

#scope("thesis", title: [Thesis])

#publish("thesis/ch-1.typ")

This introduction cites @book.

#outline(target: heading.where(level: 1))
#bibliography("../works.yml")
```

A blog post elsewhere can reference the thesis with a normal `@thesis-intro`;
across the scope boundary it renders as `Thesis Introduction, Thesis`.

See `examples/discovery_site/` for a complete fixture, and
`docs/api-sketch.typ` for the full authoring shape.

## Installation

```sh
# From a local checkout
cargo install --path . --locked

# From GitHub
cargo install --git https://github.com/jason-cairns/publisher --locked
```

This installs the `publisher` command.

Install the Typst marker helpers into Typst's local package directory as well:

```sh
./scripts/install-typst-package.sh
```

The script copies `typst/typst.toml` and `typst/lib.typ` to the standard local
package path `{data-dir}/typst/packages/local/publisher/0.1.0`. On macOS, for
example, `{data-dir}` is usually `~/Library/Application Support`; set
`PUBLISHER_TYPST_DATA_DIR` to override it.

Then import the helpers by package name in publication sources:

```typ
#import "@local/publisher:0.1.0": scope, publish
```

Use that package import instead of copying a `publisher.typ` file into each
project.

## Usage

```sh
# One HTML page per source document (default output: ./build)
publisher render --root index.typ --to html --out build

# The whole publication as one PDF
publisher render --root index.typ --to pdf

# A single named scope as its own PDF
publisher render --root index.typ --scope thesis --to pdf

# Inspect scopes, routes, active-scope stacks, and diagnostics
publisher inspect --root index.typ scopes
```

Without `--scope`, an HTML render produces every page and a PDF render covers
the whole publication. With `--scope <id>`, only that region is rendered.

## How rendering works

The publisher resolves the `#publish(...)` graph and scope extents, then
generates the Typst it hands to the compiler:

- **Outlines** are bounded to the active scope's region, so an ordinary
  `#outline(...)` lists only that scope's headings.
- **Bibliographies** are consolidated to exactly one per rendered PDF (over the
  union of cited sources, listing only cited works); each HTML page keeps its
  own. This sidesteps Typst's one-bibliography-per-document limit.
- **References** resolve natively within a document; across documents they lower
  to a link (HTML) or display text (PDF), carrying the target's scope title.
- **Heading counters** continue across sources in a numbered multi-source scope.

## Development

```sh
cargo test          # model + render tests (compiles real output)
cargo run --example discovery   # discovery evidence harness
```

Design notes live in `design/`; the implementation map is in `docs/model-map.typ` and failed approaches
with their corrections are recorded in `docs/implementation-notes.typ`.
