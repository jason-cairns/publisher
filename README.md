# cair.nz

Minimal static publishing pipeline for Typst-first publications.

## Requirements

- `make`
- `uv`
- `typst` with HTML export support
- POSIX shell tools used by the tests

## Intent

Typst is the source of truth. `make build` compiles standalone Typst
publications to intermediate HTML, delegates the site transform to the Python
CLI, then compiles the unified PDF artifact from the generated Typst assembly
source. The `Makefile` is intentionally a thin wrapper around `uv run site`.

The implementation is intentionally small: no JavaScript, no YAML or
frontmatter, no route registry, and no transformer parsing of Typst source.
Publication ownership is declared in Typst with `#publish` and `#entry`.
`#publish` adds the child to an inherited publication index; `#entry` owns
the child anonymously. HTML pages and the unified PDF use the same
ownership-derived rendered set.

Run:

```sh
make build
make test
```
