#!/bin/sh
set -eu

assert_matches_golden() {
  golden=$1
  generated=$2

  if ! cmp -s "$golden" "$generated"; then
    echo "$generated must match golden snapshot $golden" >&2
    diff -u "$golden" "$generated" >&2 || true
    exit 1
  fi
}

# Pipeline produces intermediate HTML, final HTML, and a unified PDF.
test -s build/html/index.html
test -s build/html/writing.html
test -s build/html/posts/foo.html
test -s build/html/thesis.html
test -s build/html/thesis/chapter.html
test -s public/index.html
test -s public/writing/index.html
test -s public/posts/foo/index.html
test -s public/thesis/index.html
test -s public/thesis/chapter/index.html
test -s public/colophon/index.html
test -s build/site.typ
test -s public/site.pdf

site_pdf_order="$(grep '^#publication("' build/site.typ | sed 's/^#publication("//; s/").*$//')"
expected_pdf_order='index.typ
writing.typ
posts/foo.typ
thesis.typ
thesis/chapter.typ
colophon.typ'

if [ "$site_pdf_order" != "$expected_pdf_order" ]; then
  echo 'unified PDF assembly must follow ownership preorder' >&2
  printf '%s\n' "$site_pdf_order" >&2
  exit 1
fi

# PDF assembly imports/includes are resolved from the assembly file location.
pdf_path_tmp="build/test-pdf-paths"
rm -rf "$pdf_path_tmp"
mkdir -p "$pdf_path_tmp/html" \
         "$pdf_path_tmp/nested" \
         "$pdf_path_tmp/public/colophon" \
         "$pdf_path_tmp/public/posts/foo" \
         "$pdf_path_tmp/public/thesis/chapter" \
         "$pdf_path_tmp/public/thesis" \
         "$pdf_path_tmp/public/writing"
cp -R build/html/. "$pdf_path_tmp/html/"

abs_src_dir="$(pwd -P)/src"
uv run site transform --src "$abs_src_dir" --html "$pdf_path_tmp/html" --out "$pdf_path_tmp/public" --pdf-typ "$pdf_path_tmp/nested/site.typ" --root index.typ colophon.typ index.typ posts/foo.typ thesis.typ thesis/chapter.typ writing.typ

grep -q '#import "../../../src/_publication.typ": publication' "$pdf_path_tmp/nested/site.typ"
grep -q '#include "../../../src/index.typ"' "$pdf_path_tmp/nested/site.typ"
${TYPST:?TYPST is required} compile --root . "$pdf_path_tmp/nested/site.typ" "$pdf_path_tmp/site.pdf"

# Intermediate HTML carries the marker protocol verbatim.
grep -q '<body>' build/html/index.html
grep -q '<publication-graph-publish data-target="writing.typ">Writing</publication-graph-publish>' build/html/index.html
grep -q '<publication-graph-entry data-target="colophon.typ"></publication-graph-entry>' build/html/index.html
grep -q '<publication-graph-label data-label="root-note"></publication-graph-label>' build/html/index.html
grep -q '<publication-graph-ref data-label="foo-note">foo note</publication-graph-ref>' build/html/index.html

# Final HTML wraps publication body in site chrome and resolves links to routes.
grep -q '<main>' public/index.html
grep -q 'standalone Typst publication' public/index.html
grep -q '<nav>' public/index.html
grep -q '<a href="/writing/">Writing</a>' public/index.html
grep -q '<a href="/thesis/">Thesis</a>' public/index.html
grep -q '<a href="/writing/" aria-current="page">Writing</a>' public/writing/index.html
grep -q 'anonymous ownership edge' public/posts/foo/index.html
grep -q '<a href="/thesis/" aria-current="page">Thesis</a>' public/thesis/index.html
grep -q '<a href="/thesis/chapter/">Chapter</a>' public/thesis/index.html
grep -q '<a href="/thesis/chapter/" aria-current="page">Chapter</a>' public/thesis/chapter/index.html
grep -q 'public without becoming' public/colophon/index.html

# Publication reference links are route-level links, not `.html` file links.
grep -q '<a href="/colophon/">colophon</a>' public/index.html
grep -q '<a href="/posts/foo/">Foo</a>' public/writing/index.html
grep -q '<a href="/writing/">Writing</a>' public/posts/foo/index.html
grep -q '<a href="/thesis/">Thesis</a>' public/thesis/chapter/index.html
grep -q '<a href="/">home</a>' public/colophon/index.html

# Site-global label references resolve to stable fragments. Same-publication
# references use local fragments; cross-publication references include routes.
grep -q '<span id="root-note"></span>' public/index.html
grep -q '<a href="#root-note">root note</a>' public/index.html
grep -q '<span id="foo-note"></span>' public/posts/foo/index.html
grep -q '<a href="#foo-note">own note</a>' public/posts/foo/index.html
grep -q '<a href="/posts/foo/#foo-note">foo note</a>' public/index.html

# Markers are an internal protocol; they must not leak into final HTML.
if grep -R -q '<publication-graph-' public; then
  echo 'public HTML must not include internal publication markers' >&2
  exit 1
fi

# Protocol markers are generic and must not be branded to this site.
if grep -R -q 'cairn[z]' site.pl src test; then
  echo 'implementation code must not include site-branded protocol names' >&2
  exit 1
fi

# `#entry` owns without contributing a publication-index item.
if grep -q '<a href="/colophon/">colophon</a></li>' public/index.html; then
  echo 'entry-owned pages must not become visible index items' >&2
  exit 1
fi

if [ "$(grep -c '<nav>' public/writing/index.html)" -ne 1 ]; then
  echo 'anonymous children must not add an inherited publication index' >&2
  exit 1
fi

if [ "$(grep -c '<nav>' public/posts/foo/index.html)" -ne 1 ]; then
  echo 'anonymous child descendants must inherit only ancestor publication indexes' >&2
  exit 1
fi

if [ "$(grep -c '<nav>' public/thesis/chapter/index.html)" -ne 2 ]; then
  echo 'indexed child subtrees must inherit each publication index on their ownership path' >&2
  exit 1
fi

# Site is JavaScript-free by default.
if grep -qi '<script' public/index.html; then
  echo 'public/index.html must not include JavaScript' >&2
  exit 1
fi

# Representative final HTML pages are pinned as golden snapshots.
assert_matches_golden test/golden/public/index.html public/index.html
assert_matches_golden test/golden/public/posts/foo/index.html public/posts/foo/index.html
assert_matches_golden test/golden/public/thesis/chapter/index.html public/thesis/chapter/index.html
assert_matches_golden test/golden/public/colophon/index.html public/colophon/index.html

# Each publication compiles independently to a standalone PDF.
mkdir -p build/test-preview
${TYPST:?TYPST is required} compile --root src src/index.typ build/test-preview/index.pdf

tmp="build/test-ownership"
rm -rf "$tmp"

# Fixture: a candidate source (orphan.typ) that no publication owns.
# Constraint: candidates outside the ownership tree are dropped silently — no
# build error, no published artefact.
mkdir -p "$tmp/unreachable/html" \
         "$tmp/unreachable/public/colophon" \
         "$tmp/unreachable/public/posts/foo" \
         "$tmp/unreachable/public/thesis/chapter" \
         "$tmp/unreachable/public/thesis" \
         "$tmp/unreachable/public/writing"
cp -R build/html/. "$tmp/unreachable/html/"
printf '<!DOCTYPE html><html><body><h2>Orphan</h2></body></html>' > "$tmp/unreachable/html/orphan.html"

uv run site transform --src src --html "$tmp/unreachable/html" --out "$tmp/unreachable/public" --pdf-typ "$tmp/unreachable/site.typ" --root index.typ colophon.typ index.typ posts/foo.typ thesis/chapter.typ thesis.typ writing.typ orphan.typ

test -s "$tmp/unreachable/public/index.html"
test -s "$tmp/unreachable/public/colophon/index.html"
test -s "$tmp/unreachable/public/posts/foo/index.html"
test -s "$tmp/unreachable/public/thesis/index.html"
test -s "$tmp/unreachable/public/thesis/chapter/index.html"
test -s "$tmp/unreachable/public/writing/index.html"

if [ -e "$tmp/unreachable/public/orphan/index.html" ]; then
  echo 'sources outside the ownership tree must be dropped silently, not published' >&2
  exit 1
fi

grep -q '#publication("colophon.typ")' "$tmp/unreachable/site.typ"

if grep -q '#publication("orphan.typ")' "$tmp/unreachable/site.typ"; then
  echo 'sources outside the ownership tree must not enter unified PDF assembly' >&2
  exit 1
fi

# Fixture: a publication in the rendered set publishes a target that doesn't exist.
# Constraint: ownership edges from rendered publications must resolve.
mkdir -p "$tmp/dangling-target/html" "$tmp/dangling-target/public"
cp -R build/html/. "$tmp/dangling-target/html/"
printf '<!DOCTYPE html><html><body><h2>Index</h2><publication-graph-publish data-target="writing.typ">Writing</publication-graph-publish><publication-graph-entry data-target="missing.typ"></publication-graph-entry></body></html>' > "$tmp/dangling-target/html/index.html"

if uv run site transform --src src --html "$tmp/dangling-target/html" --out "$tmp/dangling-target/public" --pdf-typ "$tmp/dangling-target/site.typ" --root index.typ colophon.typ index.typ posts/foo.typ thesis/chapter.typ thesis.typ writing.typ >/dev/null 2>&1; then
  echo 'ownership edges from rendered publications must point to known sources' >&2
  exit 1
fi

# Fixture: a rendered publication links to a candidate that is silently dropped.
# Constraint: reference edges from rendered publications must point into the
# rendered set, since dropped candidates have no route. Reference edges must
# not create ownership edges or publish otherwise unreachable sources.
mkdir -p "$tmp/dangling-link/html" "$tmp/dangling-link/public"
cp -R build/html/. "$tmp/dangling-link/html/"
printf '<!DOCTYPE html><html><body><h2>Orphan</h2></body></html>' > "$tmp/dangling-link/html/orphan.html"
printf '<!DOCTYPE html><html><body><h2>Colophon</h2><p><publication-graph-link data-target="orphan.typ">Orphan</publication-graph-link></p></body></html>' > "$tmp/dangling-link/html/colophon.html"

if uv run site transform --src src --html "$tmp/dangling-link/html" --out "$tmp/dangling-link/public" --pdf-typ "$tmp/dangling-link/site.typ" --root index.typ colophon.typ index.typ posts/foo.typ thesis/chapter.typ thesis.typ writing.typ orphan.typ >/dev/null 2>&1; then
  echo 'reference edges from rendered publications must point into the rendered set' >&2
  exit 1
fi

# Fixture: a rendered publication links to a source that was never discovered.
# Constraint: broken local publication links must fail the transform.
mkdir -p "$tmp/missing-link/html" "$tmp/missing-link/public"
cp -R build/html/. "$tmp/missing-link/html/"
printf '<!DOCTYPE html><html><body><h2>Colophon</h2><p><publication-graph-link data-target="missing.typ">Missing</publication-graph-link></p></body></html>' > "$tmp/missing-link/html/colophon.html"

if uv run site transform --src src --html "$tmp/missing-link/html" --out "$tmp/missing-link/public" --pdf-typ "$tmp/missing-link/site.typ" --root index.typ colophon.typ index.typ posts/foo.typ thesis/chapter.typ thesis.typ writing.typ >/dev/null 2>&1; then
  echo 'reference edges from rendered publications must point to known rendered sources' >&2
  exit 1
fi

# Fixture: rendered labels are site-global and must be unique.
mkdir -p "$tmp/duplicate-label/html" "$tmp/duplicate-label/public"
cp -R build/html/. "$tmp/duplicate-label/html/"
printf '<publication-graph-label data-label="root-note"></publication-graph-label>' >> "$tmp/duplicate-label/html/colophon.html"

if uv run site transform --src src --html "$tmp/duplicate-label/html" --out "$tmp/duplicate-label/public" --pdf-typ "$tmp/duplicate-label/site.typ" --root index.typ colophon.typ index.typ posts/foo.typ thesis/chapter.typ thesis.typ writing.typ >/dev/null 2>&1; then
  echo 'duplicate rendered site-global publication labels must fail validation' >&2
  exit 1
fi

# Fixture: label references close over rendered labels only. A label in an
# unowned candidate must not become routable or publish that candidate.
mkdir -p "$tmp/dangling-label/html" "$tmp/dangling-label/public"
cp -R build/html/. "$tmp/dangling-label/html/"
printf '<!DOCTYPE html><html><body><h2>Orphan</h2><publication-graph-label data-label="orphan-note"></publication-graph-label></body></html>' > "$tmp/dangling-label/html/orphan.html"
printf '<publication-graph-ref data-label="orphan-note">orphan note</publication-graph-ref>' >> "$tmp/dangling-label/html/index.html"

if uv run site transform --src src --html "$tmp/dangling-label/html" --out "$tmp/dangling-label/public" --pdf-typ "$tmp/dangling-label/site.typ" --root index.typ colophon.typ index.typ posts/foo.typ thesis/chapter.typ thesis.typ writing.typ orphan.typ >/dev/null 2>&1; then
  echo 'label references must point to labels in rendered publications' >&2
  exit 1
fi

# Fixture: missing site-global labels fail the transform.
mkdir -p "$tmp/missing-label/html" "$tmp/missing-label/public"
cp -R build/html/. "$tmp/missing-label/html/"
printf '<publication-graph-ref data-label="missing-note">Missing</publication-graph-ref>' >> "$tmp/missing-label/html/index.html"

if uv run site transform --src src --html "$tmp/missing-label/html" --out "$tmp/missing-label/public" --pdf-typ "$tmp/missing-label/site.typ" --root index.typ colophon.typ index.typ posts/foo.typ thesis/chapter.typ thesis.typ writing.typ >/dev/null 2>&1; then
  echo 'missing site-global publication labels must fail validation' >&2
  exit 1
fi

# Fixture: a heading can start with inline markup emitted by Typst HTML.
# Constraint: title extraction must use the heading text without turning valid
# inline heading markup into a transform failure.
mkdir -p "$tmp/inline-heading/html" "$tmp/inline-heading/public"
printf '<!DOCTYPE html><html><body><h2><em>Intro</em></h2><p>Body.</p></body></html>' > "$tmp/inline-heading/html/index.html"

uv run site transform --src src --html "$tmp/inline-heading/html" --out "$tmp/inline-heading/public" --pdf-typ "$tmp/inline-heading/site.typ" --root index.typ index.typ

grep -q '<title>Intro - cair.nz</title>' "$tmp/inline-heading/public/index.html"
grep -q '<h2><em>Intro</em></h2>' "$tmp/inline-heading/public/index.html"

# Fixture: posts/foo is owned by both writing (#entry) and index (#publish).
# Constraint: every publication except the root has exactly one owner.
mkdir -p "$tmp/multi-owner/html" "$tmp/multi-owner/public/posts/foo"
cp -R build/html/. "$tmp/multi-owner/html/"
printf '<publication-graph-publish data-target="posts/foo.typ">Foo</publication-graph-publish>' >> "$tmp/multi-owner/html/index.html"

# Fixture: authored HTML elements may use the generic publication-* namespace
# without being mistaken for internal protocol markers.
mkdir -p "$tmp/authored-publication-element/html" "$tmp/authored-publication-element/public"
printf '<!DOCTYPE html><html><body><h2>Index</h2><publication-note>Keep me.</publication-note></body></html>' > "$tmp/authored-publication-element/html/index.html"

uv run site transform --src src --html "$tmp/authored-publication-element/html" --out "$tmp/authored-publication-element/public" --pdf-typ "$tmp/authored-publication-element/site.typ" --root index.typ index.typ

grep -q '<publication-note>Keep me.</publication-note>' "$tmp/authored-publication-element/public/index.html"

if uv run site transform --src src --html "$tmp/multi-owner/html" --out "$tmp/multi-owner/public" --pdf-typ "$tmp/multi-owner/site.typ" --root index.typ colophon.typ index.typ posts/foo.typ thesis/chapter.typ thesis.typ writing.typ >/dev/null 2>&1; then
  echo 'publications with multiple owners must fail ownership validation' >&2
  exit 1
fi

# Fixture: the root publication is configurable, not hard-coded to index.typ.
# Constraint: any source can serve as the root; route resolution adapts.
mkdir -p "$tmp/alternate-root/html" "$tmp/alternate-root/public/index"
printf '<!DOCTYPE html><html><body><publication-graph-publish data-target="index.typ">Home</publication-graph-publish><p><publication-graph-link data-target="index.typ">Home</publication-graph-link></p></body></html>' > "$tmp/alternate-root/html/writing.html"
printf '<!DOCTYPE html><html><body><p><publication-graph-link data-target="writing.typ">Root</publication-graph-link></p></body></html>' > "$tmp/alternate-root/html/index.html"

uv run site transform --src src --html "$tmp/alternate-root/html" --out "$tmp/alternate-root/public" --pdf-typ "$tmp/alternate-root/site.typ" --root writing.typ writing.typ index.typ

test -s "$tmp/alternate-root/public/index.html"
test -s "$tmp/alternate-root/public/index/index.html"
grep -q '<a href="/index/">Home</a>' "$tmp/alternate-root/public/index.html"
grep -q '<a href="/">Root</a>' "$tmp/alternate-root/public/index/index.html"

alternate_root_pdf_order="$(grep '^#publication("' "$tmp/alternate-root/site.typ" | sed 's/^#publication("//; s/").*$//')"
expected_alternate_root_pdf_order='writing.typ
index.typ'

if [ "$alternate_root_pdf_order" != "$expected_alternate_root_pdf_order" ]; then
  echo 'alternate-root unified PDF assembly must follow that root ownership preorder' >&2
  printf '%s\n' "$alternate_root_pdf_order" >&2
  exit 1
fi
