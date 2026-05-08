# AGENTS

- Keep the implementation aggressively small.
- Typst is the source of truth.
- Never parse Typst source in Prolog.
- HTML is an intermediate artifact.
- Filesystem structure defines routing.
- Every publication compiles independently.
- No YAML/frontmatter/config metadata layer.
- Prefer plain text and semantic HTML.
- Avoid JavaScript unless unavoidable.
- Prefer explicit Typst semantics over directory magic.
- Keep Typst sources plain, modern, and semantic.
- Keep Prolog pure and modern where possible, following Markus Triska's style: relations first, DCGs for sequences, clear predicate names, and side effects isolated at the boundary.
- Add dependencies only when clearly justified.
- Create a GitHub branch per issue before implementation.
- Interact with GitHub using the connector.
- After creating a branch with the GitHub connector, fetch that branch ref before switching locally, for example `git fetch origin branch-name:refs/remotes/origin/branch-name` then `git switch --track -c branch-name origin/branch-name`.
- Treat `docs/PRD.md` as important product and architecture context before implementation.
