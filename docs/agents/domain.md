# Domain Docs

How the engineering skills should consume this repo's domain documentation when exploring the codebase.

## Layout

This is a single-context repo.

Read these before implementation work:

- `docs/PRD.md` for product requirements, architecture, and core principles.
- Relevant ADRs under `docs/adr/` for architectural decisions.
- `AGENTS.md` for repo-specific engineering rules.

There is currently no root `CONTEXT.md` or `CONTEXT-MAP.md`.

## Context growth

The current source of product and domain context is `docs/PRD.md`, supported by ADRs. If the PRD starts accumulating glossary-like domain language, repeated implementation rules, or long-lived context that is not a product requirement, suggest creating a root `CONTEXT.md`.

Do not create `CONTEXT.md` automatically. Raise it as a recommendation when the project context has grown enough that separating domain language from product requirements would make future agent work clearer.

## ADRs

Read ADRs that touch the area you're about to work in. If your output contradicts an existing ADR, surface the conflict explicitly rather than silently overriding it.

## Vocabulary

Use this repo's existing terms when writing issues, tests, docs, or code comments. In particular, preserve the project's Typst-first publishing vocabulary:

- Typst is the source of truth.
- HTML is an intermediate artifact.
- Filesystem structure defines routing.
- Each publication compiles independently.
