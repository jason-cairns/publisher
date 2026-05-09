# Context

The shared vocabulary for cair.nz. Use these terms exactly when writing
issues, tests, ADRs, code, or comments. If a term you need isn't here yet,
either you're inventing language the project doesn't use, or there's a
real gap — add it here.

The product framing lives in [`docs/PRD.md`](docs/PRD.md). Architectural
decisions live in [`docs/adr/`](docs/adr/). This file is the glossary plus
the formal model that the architecture rests on.

## Glossary

### Source material

**Source** — the relative path of a `.typ` file inside `src/`, e.g.
`index.typ`, `posts/foo.typ`. A source is the canonical identifier for a
publication. Sources are passed verbatim as `data-target` attributes inside
the marker protocol.

**Stem** — a source with the `.typ` suffix removed, e.g. `posts/foo`. Used
to derive routes and output paths.

**Publication candidate** — any source the build presents to the
transformer. The build's underscore-prefix discovery convention (`_*.typ`
files are skipped) is operational, not load-bearing: a non-underscore
source is a candidate; whether it ends up published is decided by the
ownership tree, not by its filename.

**Helper** — a `.typ` file the author treats as import-only (typically
named `_*.typ`). Helpers are filtered out at discovery. The convention is
a hint, not an enforced rule: a non-underscore file that no one owns is
also dropped at the transformer (silently, as a non-publication).

### Publishing structure

**Publication** — a candidate that is reachable from the root publication
through the ownership relation. Every publication has exactly one route,
one published HTML file, and one section in the unified PDF.

**Rendered set** — the set of publications, written `R`. `R` is the
transitive closure of the root publication under ownership edges, restricted
to known candidates. Candidates not in `R` are **dropped silently** — no
build error, no published artefact (ADR 0006).

**Root publication** — the unique entry point of the ownership tree,
passed to the transformer as `ROOT_SOURCE`. Conventionally `index.typ`,
but the transformer accepts any source. The root has no owner.

**Owner** — the publication that authors an ownership edge pointing at
another publication.

**Ownership edge** — a directed edge `(owner, target, kind, label)` where
`kind ∈ {nav, publish}`. Authored in Typst with `#nav` or `#publish`.
Emitted as a marker into intermediate HTML, then read by the transformer.
Ownership edges define the rendered set.

**Reference edge** — a directed edge `(source, target, label)` authored in
Typst with `#publication-link`. Reference edges express ordinary
hyperlinks. They do not contribute to ownership.

**Ownership graph** — the directed graph on publications whose edges are
the ownership edges restricted to `R`. Constrained to be a rooted tree
(see Formal model).

**Ownership tree** — the same graph viewed as a rooted tree. Used
interchangeably with **Ownership graph** in this codebase.

**Ownership path** — the unique path from the root to a given publication
in the ownership tree. Determines the publication's navigation context.

**Navigation entry** — a `nav` ownership edge, contributing a visible item
to its owner's published nav bar. A `publish` edge owns without producing
a nav entry.

**Navigation context** — the ordered list of nav bars inherited along a
publication's ownership path. The published page renders one nav bar per
ancestor that has nav entries.

### Pipeline artefacts

**Intermediate HTML** — Typst's HTML output before transformation. Lives
under `build/html/`. Contains ownership and reference edges as marker
elements.

**Final HTML** — the transformed, published HTML written by the Prolog
transformer to `public/`. Markers have been resolved into nav bars and
anchors; site chrome has been added.

**Site** — the complete published artefact set: every publication's final
HTML plus the unified PDF (`public/site.pdf`).

### Marker protocol

The contract between Typst (`src/_publication.typ`) and the transformer
(`site.pl`). Markers are semantic HTML elements emitted into intermediate
HTML and consumed by the transformer.

| Marker | Edge produced | Authored as |
| --- | --- | --- |
| `<cairnz-nav data-target="T">Label</cairnz-nav>` | ownership edge `(u, T, nav, "Label")` | `#nav("T")[Label]` |
| `<cairnz-publish data-target="T"></cairnz-publish>` | ownership edge `(u, T, publish, "")` | `#publish("T")` |
| `<cairnz-link data-target="T">Label</cairnz-link>` | reference edge `(u, T, "Label")` | `#publication-link("T")[Label]` |

Where `u` is the source containing the marker and `T` is the target source.

## Formal model

Let `C` be the finite set of candidate sources presented to the transformer.
Let `r ∈ C` be the root publication.

Let

```
E_own ⊆ C × C × {nav, publish} × Σ*
E_ref ⊆ C × C × Σ*
```

be the ownership and reference edges extracted from the intermediate HTML.

Let `own ⊆ C × C` be the projection of `E_own` to its first two
components: `own = { (u, v) | ∃ k, ℓ.  (u, v, k, ℓ) ∈ E_own }`.

The **rendered set** `R ⊆ C` is the smallest set with:

```
r ∈ R
(u, v) ∈ own  ∧  u ∈ R  ∧  v ∈ C   ⟹   v ∈ R
```

A valid build satisfies all of:

1. **Ownership target closure.** For every `(u, v, k, ℓ) ∈ E_own` with
   `u ∈ R`, `v ∈ R`. (By construction of `R`, this is equivalent to
   requiring `v ∈ C` for those edges; a target outside `C` is a build
   error.)
2. **Single-owner.** For every `p ∈ R \ {r}`, there is exactly one
   `u ∈ R` with `(u, p) ∈ own`.
3. **Root unowned.** There is no `u ∈ R` with `(u, r) ∈ own`.
4. **Reference closure.** For every `(u, v, ℓ) ∈ E_ref` with `u ∈ R`,
   `v ∈ R`.

Conditions (1)–(3) make `own` restricted to `R × R` a rooted tree with
root `r`. Reference edges (condition 4) may be cyclic and are not
constrained beyond closure into `R`.

Edges originating outside `R` are ignored — they place no constraints on
the build.

**Sibling order.** Nav entries authored by a single owner are ordered by
their appearance in that source's intermediate HTML
(DCG-sequential).

**Ownership path.** The unique path `r = p_0, p_1, …, p_n = p` in the
ownership tree. Used to render `p`'s navigation context.

**Route function** `ρ : R → URL`:

```
ρ(r) = "/"
ρ(p) = "/" + stem(p) + "/"        for p ≠ r
```

**Output-path function** `π : R → FilePath`, parameterised by output
directory `D`:

```
π(r) = D + "/index.html"
π(p) = D + "/" + stem(p) + "/index.html"   for p ≠ r
```

The transformer is a function
`T : (E_own, E_ref, body : R → HTML) → Site` that, given valid inputs,
writes the final HTML at every `π(p)` for `p ∈ R` and rewrites every
reference marker in those bodies to an anchor whose `href` is
`ρ(target)`.

## Invariants worth restating

- The ownership relation lives entirely in Typst source. The transformer
  reads markers from intermediate HTML; it never parses Typst.
- Filesystem paths under `src/` map publications to routes (`stem(p)`),
  but they do not decide publication membership. `R` is decided by
  ownership reachability.
- The underscore convention is a discovery hint, not a publishing rule.
  Anything not in `R` — underscored or not — is dropped silently.
- A failure of any condition (1)–(4) on edges originating in `R` is a
  build error. Edges originating outside `R` impose no constraints.
