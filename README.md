# cair.nz

Minimal static publishing pipeline for Typst-first publications.

## Requirements

- `make`
- `typst` with HTML export support
- `scryer-prolog`
- POSIX shell tools used by the tests

## Intent

Typst is the source of truth. `make build` compiles standalone Typst
publications to intermediate HTML, runs a Scryer Prolog transform to produce
final site HTML and a generated Typst PDF assembly source, then compiles the
unified PDF artifact from that generated source.

The implementation is intentionally small: no JavaScript, no YAML or
frontmatter, no route registry, and no Prolog parsing of Typst source.
Publication ownership is declared in Typst with `#publish` and `#entry`.
`#publish` adds the child to an inherited publication index; `#entry` owns
the child anonymously. HTML pages and the unified PDF use the same
ownership-derived rendered set.

Run:

```sh
make build
make test
```
