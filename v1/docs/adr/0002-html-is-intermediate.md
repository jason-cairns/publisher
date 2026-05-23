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
  -> python cli transform
  -> final site html
```

The Python CLI operates only on generated HTML during transformation.

The transformer is responsible for:
- route mapping for the owned publication set
- ownership graph validation
- publication index synthesis from Typst-emitted markers
- internal link rewriting
- site chrome injection
- HTML normalization
- generated Typst PDF assembly source

The transformer does not parse Typst source.
HTML fixtures can exercise the transform directly without invoking Typst.
`make` remains a thin build wrapper around the CLI.

## Consequences

Positive:
- small implementation surface
- clear separation of responsibilities
- independent testing of the transformer
- no duplicate rendering engine

Negative:
- Typst HTML structure becomes an external dependency
- some transformations may be awkward downstream
