= Agent prompt: PRD-003 selector/scope discovery

You are the orchestrating agent for the Milestone 3 selector/scope discovery spike in this repository.

Your job is to determine whether the target authoring model in `design/prd-003-typst-selector-usage.typ` can be implemented mostly through Typst-native selectors, context, labels, metadata, references, bibliography behavior, outlines, counters, and scoped compilation, without rebuilding the old publisher `Query`/`Projection` architecture.

Do not start by rewriting the main Rust model.
This is discovery first.
Use small throwaway fixtures, small experimental Rust or Typst programs, and focused evidence.
Keep the final implementation recommendation grounded in what Typst actually supports.

== Required reading

Read these files before planning:

- `design/prd-003-typst-selector-usage.typ`
- `docs/model.typ`
- `docs/model-map.typ`
- `docs/api-sketch.typ`
- `docs/implementation-notes.typ`
- the current parser/rendering code under `src/`
- the current API sketch fixture under `examples/api_sketch_site/`

While reading, separate current implementation facts from the target Milestone 3 model.
The target model deliberately demotes old concepts such as `Query`, `Projection`, and user-facing `Node`.

== Target model to preserve

Preserve these decisions unless direct Typst evidence proves one impossible:

- `#scope(...)` is the only publisher-specific semantic primitive besides `#publish(...)`.
- `#scope(...)` is source-document-level metadata, not visible rendered content and not a lexical wrapper.
- `#publish(...)` declares a routed source-document edge; it is not literal inclusion.
- Typst `#include` remains literal inclusion.
- Standard Typst `#outline(...)`, `#bibliography(...)`, `query(...)`, labels, references, counters, and selectors should do the obvious thing in the current rendered publication region.
- Edition selection belongs to CLI/runtime invocation, not authored source.
- `publisher.in-scope("a", "b")[...]` is an explicit escape hatch for ordinary Typst content over one or more named scope ids.
- Scope ids are globally unique machine identities.
- Scope titles are explicit display text and do not need to be globally unique.
- Labels are globally unique across the publication.
- Cross-scope references use normal Typst lookup, then may add nearest explicit titled-scope display context.
- HTML output keeps one HTML document per published source Typst document.

== Orchestration style

Act as one orchestrator.
Use subagents or separate workstreams where useful, but keep one coherent decision log and final recommendation.
Parallelize independent discovery work, then rejoin before making architecture conclusions.

Suggested workstreams:

1. Typst introspection workstream
   Investigate selectors, labels, metadata, locations, `query(...)`, `outline(target: ...)`, and whether a source-level marker can become a region boundary visible to ordinary Typst functions.

2. Compilation-boundary workstream
   Investigate scoped compilation and generated assembly.
   Determine whether rendering a source document, a named scope, or a whole publication can make ordinary Typst functions naturally see only the current rendered region.
   Include counters, heading levels, and heading promotion experiments.

3. Bibliography/reference workstream
   Investigate whether standard `#bibliography(...)` and standard references can work with publisher-supplied rendered regions.
   Test globally unique labels, cross-source references, cross-scope reference display context, and duplicate scope titles.

4. Publication graph and routing workstream
   Prototype `#publish(...)` metadata, route derivation, source-document graph order, scope inheritance through `#publish(...)`, and one HTML document per published source Typst document.
   Keep `#include` literal and distinct from `#publish`.

5. API/diagnostics workstream
   Determine inspect output required to make scope extent, active scope stacks, publication graph order, routes, missing scope ids, duplicate scope ids, and duplicate display-title warnings reviewable.

Workstreams 1 through 4 can start in parallel after the initial reading.
The diagnostics workstream should run after the first evidence pass, because it depends on what the prototypes can expose.

== Spike fixture

Create a small throwaway fixture for discovery.
It may live under a temporary directory or a clearly named experimental directory, but do not confuse it with production examples unless the discovery is accepted.

The fixture should include:

```text
index.typ
publisher.typ
writing.typ
writing/blog-1.typ
writing/blog-2.typ
thesis/intro.typ
thesis/ch-1.typ
cv.typ
works.yml or works.bib
summary.typ
```

Required authored shape:

```typ
#import "publisher.typ": scope, publish

= Home

#publish("writing.typ")
#publish("thesis/intro.typ")
#publish("cv.typ")
#scope("home", tags: ("nav",))
```

`writing.typ` should declare:

```typ
#scope("writing", title: [Writing])
```

and publish at least two writing source documents.

`thesis/intro.typ` should declare:

```typ
#scope("thesis", title: [Thesis])
```

and publish at least one thesis chapter.

Include a standard Typst reference from outside `thesis` to a globally unique label inside `thesis`.
Include normal `#outline(target: heading.where(level: 1))`.
Include normal `#bibliography(...)`.
Include at least one `publisher.in-scope("writing", "thesis")[...]` experiment.
Include a literal `#include "summary.typ"` to prove include remains distinct from publish.

== Questions to answer with evidence

Answer these with concrete experiments, not speculation:

1. Can a source-level `#scope(...)` marker be represented through Typst metadata, labels, or locations so the publisher can recover source-document-level scope regions?
2. Can ordinary `#outline(...)` render over the current rendered region without replacing it with `publisher.outline(...)`?
3. Can ordinary `#bibliography(...)` render over the current rendered region without replacing it with `publisher.bibliography(...)`?
4. Can ordinary `query(...)` and selectors operate inside a publisher-supplied region boundary?
5. Can `publisher.in-scope("a", "b")[...]` be expressed as a Typst-native context switch over a union of scope regions?
6. Can standard Typst labels and references preserve lookup while allowing scope-aware display enrichment after target resolution?
7. Can source-local rendering preserve one HTML document per published source Typst document?
8. Can scope rendering as a standalone compilable unit support heading promotion, local counters, bibliography, labels, and references?
9. What exact Typst crate APIs are required?
10. Where does Typst resist the target model and force a custom adapter?

== Required outputs

Produce a discovery report in `design/`.
Use Typst format.
The report should include:

- a one-page executive summary of feasibility;
- a decision table: works natively, works with small adapter, risky, not viable;
- experiments run and exact commands;
- files created or modified for the spike;
- evidence snippets from output or diagnostics;
- implementation recommendation;
- risks and unresolved questions;
- whether the PRD needs revisions before implementation.

Also produce any small spike code or fixtures needed to back the report.
Keep spike code clearly separated from production implementation.

== Validation expectations

Run the smallest meaningful checks for each experiment.
Prefer commands that can be rerun by a later implementation agent.

At minimum, try to prove or disprove these capabilities:

```sh
typst compile ...
cargo test ...
cargo run --example ...
```

Use the actual commands that fit the spike.
If a command fails because the target behavior is not supported, preserve the failure and document the correct conclusion in the report.

Per repository guidance, when something fails during discovery, add the failed approach and the correct approach or current conclusion to `docs/implementation-notes.typ`.

== Commit discipline

Make regular atomic commits.
Do not sweep unrelated worktree changes into commits.
Good commit slices:

- create discovery fixture;
- prove one Typst API behavior;
- add reference/bibliography evidence;
- add inspect/diagnostic evidence;
- write final discovery report;
- update implementation notes.

== Non-goals

Do not implement the final Milestone 3 architecture.
Do not port the whole parser or renderer.
Do not preserve the old user-facing `publisher.child`, `publisher.outline`, `publisher.bibliography`, `publisher.ref`, `Query`, `Projection`, or `Node` concepts unless evidence proves the target model cannot work without a limited adapter.
Do not add a source-authored `#edition(...)` API.
Do not make tags drive region selection in this milestone.

== Final answer expected from the orchestrator

Return:

- whether PRD-003 is feasible as written;
- what the smallest implementation architecture should be;
- what Typst APIs make it possible;
- which parts need custom adapters;
- which PRD decisions should change, if any;
- the commit hashes for discovery artifacts.
