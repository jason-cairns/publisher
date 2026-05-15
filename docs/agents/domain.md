# Domain Docs

How the engineering skills should consume this repo's domain documentation when exploring the codebase.

## Layout

This is a single-context repo.

Read these before implementation work:

- `CONTEXT.md` at the repo root for the project's domain glossary and formal model.
- Relevant ADRs under `docs/adr/` for architectural decisions.
- `AGENTS.md` for repo-specific engineering rules.

## Vocabulary

Use the terms defined in `CONTEXT.md` exactly when writing issues, tests, docs, or code comments. If the concept you need isn't in the glossary yet, that's a signal — either you're inventing language the project doesn't use (reconsider) or there's a real gap (add it).

Load-bearing terms (full definitions in `CONTEXT.md`):

- **Source**, **stem**, **publication candidate**, **helper**.
- **Publication**, **rendered set**, **root publication**.
- **Ownership edge** (`publish`, `entry`), **reference edge** (`link`), **owner**.
- **Ownership graph / tree**, **ownership path**, **publication index context**.
- **Intermediate HTML**, **final HTML**, **site**.
- **Marker protocol**: `publication-graph-publish`, `publication-graph-entry`, `publication-graph-link`, `publication-graph-label`, `publication-graph-ref`.

## ADRs

Read ADRs that touch the area you're about to work in. If your output contradicts an existing ADR, surface the conflict explicitly rather than silently overriding it.
