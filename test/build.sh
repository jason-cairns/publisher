#!/bin/sh
set -eu

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
test -s public/site.pdf

# Intermediate HTML carries the marker protocol verbatim.
grep -q '<body>' build/html/index.html
grep -q '<publication-publish data-target="writing.typ">Writing</publication-publish>' build/html/index.html
grep -q '<publication-entry data-target="colophon.typ"></publication-entry>' build/html/index.html

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

# Markers are an internal protocol; they must not leak into final HTML.
if grep -R -q '<publication-' public; then
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

scryer-prolog site.pl -- src "$tmp/unreachable/html" "$tmp/unreachable/public" index.typ colophon.typ index.typ posts/foo.typ thesis/chapter.typ thesis.typ writing.typ orphan.typ

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

# Fixture: a publication in the rendered set publishes a target that doesn't exist.
# Constraint: ownership edges from rendered publications must resolve.
mkdir -p "$tmp/dangling-target/html" "$tmp/dangling-target/public"
cp -R build/html/. "$tmp/dangling-target/html/"
printf '<!DOCTYPE html><html><body><h2>Index</h2><publication-publish data-target="writing.typ">Writing</publication-publish><publication-entry data-target="missing.typ"></publication-entry></body></html>' > "$tmp/dangling-target/html/index.html"

if scryer-prolog site.pl -- src "$tmp/dangling-target/html" "$tmp/dangling-target/public" index.typ colophon.typ index.typ posts/foo.typ thesis/chapter.typ thesis.typ writing.typ >/dev/null 2>&1; then
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
printf '<!DOCTYPE html><html><body><h2>Colophon</h2><p><publication-link data-target="orphan.typ">Orphan</publication-link></p></body></html>' > "$tmp/dangling-link/html/colophon.html"

if scryer-prolog site.pl -- src "$tmp/dangling-link/html" "$tmp/dangling-link/public" index.typ colophon.typ index.typ posts/foo.typ thesis/chapter.typ thesis.typ writing.typ orphan.typ >/dev/null 2>&1; then
  echo 'reference edges from rendered publications must point into the rendered set' >&2
  exit 1
fi

# Fixture: a rendered publication links to a source that was never discovered.
# Constraint: broken local publication links must fail the transform.
mkdir -p "$tmp/missing-link/html" "$tmp/missing-link/public"
cp -R build/html/. "$tmp/missing-link/html/"
printf '<!DOCTYPE html><html><body><h2>Colophon</h2><p><publication-link data-target="missing.typ">Missing</publication-link></p></body></html>' > "$tmp/missing-link/html/colophon.html"

if scryer-prolog site.pl -- src "$tmp/missing-link/html" "$tmp/missing-link/public" index.typ colophon.typ index.typ posts/foo.typ thesis/chapter.typ thesis.typ writing.typ >/dev/null 2>&1; then
  echo 'reference edges from rendered publications must point to known rendered sources' >&2
  exit 1
fi

# Fixture: posts/foo is owned by both writing (#entry) and index (#publish).
# Constraint: every publication except the root has exactly one owner.
mkdir -p "$tmp/multi-owner/html" "$tmp/multi-owner/public/posts/foo"
cp -R build/html/. "$tmp/multi-owner/html/"
printf '<publication-publish data-target="posts/foo.typ">Foo</publication-publish>' >> "$tmp/multi-owner/html/index.html"

if scryer-prolog site.pl -- src "$tmp/multi-owner/html" "$tmp/multi-owner/public" index.typ colophon.typ index.typ posts/foo.typ thesis/chapter.typ thesis.typ writing.typ >/dev/null 2>&1; then
  echo 'publications with multiple owners must fail ownership validation' >&2
  exit 1
fi

# Fixture: the root publication is configurable, not hard-coded to index.typ.
# Constraint: any source can serve as the root; route resolution adapts.
mkdir -p "$tmp/alternate-root/html" "$tmp/alternate-root/public/index"
printf '<!DOCTYPE html><html><body><publication-publish data-target="index.typ">Home</publication-publish><p><publication-link data-target="index.typ">Home</publication-link></p></body></html>' > "$tmp/alternate-root/html/writing.html"
printf '<!DOCTYPE html><html><body><p><publication-link data-target="writing.typ">Root</publication-link></p></body></html>' > "$tmp/alternate-root/html/index.html"

scryer-prolog site.pl -- src "$tmp/alternate-root/html" "$tmp/alternate-root/public" writing.typ writing.typ index.typ

test -s "$tmp/alternate-root/public/index.html"
test -s "$tmp/alternate-root/public/index/index.html"
grep -q '<a href="/index/">Home</a>' "$tmp/alternate-root/public/index.html"
grep -q '<a href="/">Root</a>' "$tmp/alternate-root/public/index/index.html"
