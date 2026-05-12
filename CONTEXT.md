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
`kind ∈ {publish, entry}`. Authored in Typst with `#publish` or `#entry`.
Emitted as a marker into intermediate HTML, then read by the transformer.
Ownership edges define the rendered set.

**Reference edge** — a directed edge `(source, target, label)` authored in
Typst with `#publication-link`. Reference edges express ordinary
hyperlinks. They do not contribute to ownership.

**Publication label** — a site-global label defined in a rendered
publication with `#publication-label(<name>)`. The stable HTML fragment is
the label name, e.g. `<pricing-signal>` becomes `#pricing-signal`.

**Label reference** — a reference to a publication label, authored with
`#publication-ref(<name>)[Text]`. Label references do not contribute to
ownership, publication inclusion, PDF order, or publication index context.

**Ownership graph** — the directed graph on publications whose edges are
the ownership edges restricted to `R`. Constrained to be a rooted tree
(see Formal model).

**Ownership tree** — the same graph viewed as a rooted tree. Used
interchangeably with **Ownership graph** in this codebase.

**Ownership path** — the unique path from the root to a given publication
in the ownership tree. Determines the publication's inherited publication
index context.

**Publication index entry** — a `publish` ownership edge, contributing a
visible ordered item to its owner's publication index. An `entry` edge owns
without producing an index item.

**Publication index context** — the ordered list of publication indexes
inherited along a publication's ownership path. The published page renders
one plain HTML `<nav>` block per ancestor that has publication index entries.

### Pipeline artefacts

**Intermediate HTML** — Typst's HTML output before transformation. Lives
under `build/html/`. Contains ownership and reference edges as marker
elements.

**Final HTML** — the transformed, published HTML written by the Prolog
transformer to `public/`. Markers have been resolved into index blocks and
anchors; site chrome has been added.

**PDF assembly source** — the generated Typst file written by the
transformer under `build/`. It includes publications in ownership preorder
from the same rendered set as final HTML. It is an intermediate artifact,
not authored metadata.

**Site** — the complete published artefact set: every publication's final
HTML plus the unified PDF (`public/site.pdf`).

### Marker protocol

The contract between Typst (`src/_publication.typ`) and the transformer
(`site.pl`). Markers are semantic HTML elements emitted into intermediate
HTML and consumed by the transformer.
The transformer reserves only the `publication-graph-` marker prefix;
other `publication-*` elements remain ordinary authored HTML.

| Marker | Edge produced | Authored as |
| --- | --- | --- |
| `<publication-graph-publish data-target="T">Label</publication-graph-publish>` | ownership edge `(u, T, publish, "Label")` | `#publish("T")[Label]` |
| `<publication-graph-entry data-target="T"></publication-graph-entry>` | ownership edge `(u, T, entry, "")` | `#entry("T")` |
| `<publication-graph-link data-target="T">Label</publication-graph-link>` | reference edge `(u, T, "Label")` | `#publication-link("T")[Label]` |
| `<publication-graph-label data-label="A"></publication-graph-label>` | label definition `(u, A)` | `#publication-label(<A>)` |
| `<publication-graph-ref data-label="A">Label</publication-graph-ref>` | label reference `(u, A, "Label")` | `#publication-ref(<A>)[Label]` |

Where `u` is the source containing the marker, `T` is the target source,
and `A` is a site-global label name.

## Formal model

Let `C` be the finite set of candidate sources presented to the transformer.
Let `r ∈ C` be the root publication.

Let

```
E_own ⊆ C × C × {publish, entry} × Σ*
E_ref ⊆ C × C × Σ*
L_def ⊆ C × Σ*
L_ref ⊆ C × Σ* × Σ*
```

be the ownership edges, publication reference edges, publication label
definitions, and label references extracted from the intermediate HTML.

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
5. **Label uniqueness.** For every label name `a`, there is at most one
   `u ∈ R` with `(u, a) ∈ L_def`.
6. **Label reference closure.** For every `(u, a, ℓ) ∈ L_ref` with
   `u ∈ R`, there is some `v ∈ R` with `(v, a) ∈ L_def`.

Conditions (1)–(3) make `own` restricted to `R × R` a rooted tree with
root `r`. Reference edges (condition 4) may be cyclic and are not
constrained beyond closure into `R`. Label references (condition 6) are
ordinary references into the rendered set's site-global label table.

Edges originating outside `R` are ignored — they place no constraints on
the build.

**Sibling order.** Publication index entries authored by a single owner are
ordered by their appearance in that source's intermediate HTML
(DCG-sequential).

**Ownership path.** The unique path `r = p_0, p_1, …, p_n = p` in the
ownership tree. Used to render `p`'s publication index context.

**Route function** `ρ : R → URL`:

```
ρ(r) = "/"
ρ(p) = "/" + stem(p) + "/"        for p ≠ r
```

**Label fragment function** `φ : Σ* → Fragment`:

```
φ(a) = "#" + a
```

**Output-path function** `π : R → FilePath`, parameterised by output
directory `D`:

```
π(r) = D + "/index.html"
π(p) = D + "/" + stem(p) + "/index.html"   for p ≠ r
```

The transformer is a function
`T : (E_own, E_ref, L_def, L_ref, body : R → HTML) → Site` that, given valid inputs,
writes the final HTML at every `π(p)` for `p ∈ R`, rewrites every
reference marker in those bodies to an anchor whose `href` is `ρ(target)`,
rewrites every label reference to `φ(label)` for local references or
`ρ(label_source) + φ(label)` for cross-publication references,
and writes the PDF assembly source by traversing the ownership tree in
preorder.

## Invariants worth restating

- The ownership relation lives entirely in Typst source. The transformer
  reads markers from intermediate HTML; it never parses Typst.
- Filesystem paths under `src/` map publications to routes (`stem(p)`),
  but they do not decide publication membership. `R` is decided by
  ownership reachability.
- The underscore convention is a discovery hint, not a publishing rule.
  Anything not in `R` — underscored or not — is dropped silently.
- A failure of any condition (1)–(6) on edges originating in `R` is a
  build error. Edges originating outside `R` impose no constraints.
- Label definitions and label references outside `R` are ignored; they do
  not publish otherwise dropped candidates.
