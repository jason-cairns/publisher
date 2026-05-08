# ADR 0002 — HTML Is An Intermediate Artifact

## Status

Accepted

## Context

The site must support:
- semantic HTML
- PDF output
- shared document semantics
- extremely small implementation size

There are multiple possible ownership boundaries:
- custom renderer from Typst source
- custom HTML generator
- Typst HTML post-processing

## Decision

Typst-generated HTML is treated as an intermediate artifact.

Pipeline:

```text
.typ
  -> typst html
  -> prolog transform
  -> final site html
```

Scryer Prolog operates only on generated HTML.

The Prolog transformer is responsible for:
- route derivation
- navigation synthesis
- internal link rewriting
- site chrome injection
- HTML normalization

The transformer does not parse Typst source.

## Consequences

Positive:
- small implementation surface
- clear separation of responsibilities
- independent testing of the transformer
- no duplicate rendering engine

Negative:
- Typst HTML structure becomes an external dependency
- some transformations may be awkward downstream
