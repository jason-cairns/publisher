# AGENTS

- Keep the implementation aggressively small.
- Typst is the source of truth.
- Never parse Typst source in the transformer.
- HTML is an intermediate artifact.
- The Python CLI transformer reads Typst-generated HTML markers.
- Typst-authored ownership defines publication structure.
- Filesystem paths map owned publications to routes.
- Every publication compiles independently.
- No YAML/frontmatter/config metadata layer.
- Prefer plain text and semantic HTML.
- Avoid JavaScript unless unavoidable.
- Prefer explicit Typst semantics over directory magic.
- Keep Typst sources plain, modern, and semantic.
- Add dependencies only when clearly justified.
- Create a GitHub branch per issue before implementation.
- Interact with GitHub using the connector.
- After creating a branch with the GitHub connector, fetch that branch ref before switching locally, for example `git fetch origin branch-name:refs/remotes/origin/branch-name` then `git switch --track -c branch-name origin/branch-name`.
- Treat `docs/PRD.md` as important product and architecture context before implementation.

## Python style

- Keep Python small, direct, typed where it clarifies the boundary, and boring.
- Prefer standard-library dataclasses, pathlib, and explicit functions over framework-style indirection.
- Use pytest fixtures for shared test setup; keep assertions close to the behavior under test.
- Build HTML with lxml elements and Python escaping helpers, not ad hoc string interpolation for user-authored content.

## Agent skills

### Issue tracker

Issues and PRDs are tracked in GitHub Issues for `jason-cairns/my-site` using the inbuilt GitHub connector. See `docs/agents/issue-tracker.md`.

### Triage labels

Use the default five-label triage vocabulary. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context repo: read `docs/PRD.md` and relevant ADRs under `docs/adr/`; suggest adding `CONTEXT.md` once context grows too large for the PRD/ADR setup. See `docs/agents/domain.md`.
