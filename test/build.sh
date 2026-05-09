#!/bin/sh
set -eu

# Pipeline produces intermediate HTML, final HTML, and a unified PDF.
test -s build/html/index.html
test -s build/html/writing.html
test -s build/html/posts/foo.html
test -s public/index.html
test -s public/writing/index.html
test -s public/posts/foo/index.html
test -s public/colophon/index.html
test -s public/site.pdf

# Intermediate HTML carries the marker protocol verbatim.
grep -q '<body>' build/html/index.html
grep -q '<cairnz-nav data-target="writing.typ">Writing</cairnz-nav>' build/html/index.html

# Final HTML wraps publication body in site chrome and resolves links to routes.
grep -q '<main>' public/index.html
grep -q 'standalone Typst publication' public/index.html
grep -q '<nav>' public/index.html
grep -q '<a href="/writing/">Writing</a>' public/index.html
grep -q '<a href="/posts/foo/">Foo</a>' public/writing/index.html
grep -q '<a href="/posts/foo/">Foo</a>' public/posts/foo/index.html
grep -q 'published without becoming' public/colophon/index.html

# Markers are an internal protocol; they must not leak into final HTML.
if grep -R -q '<cairnz-' public; then
  echo 'public HTML must not include internal publication markers' >&2
  exit 1
fi

# `#publish` owns without contributing a nav entry.
if grep -q '<a href="/colophon/">colophon</a></li>' public/index.html; then
  echo 'published-only pages must not become visible nav items' >&2
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
         "$tmp/unreachable/public/writing"
cp -R build/html/. "$tmp/unreachable/html/"
printf '<!DOCTYPE html><html><body><h2>Orphan</h2></body></html>' > "$tmp/unreachable/html/orphan.html"

scryer-prolog site.pl -- src "$tmp/unreachable/html" "$tmp/unreachable/public" index.typ colophon.typ index.typ posts/foo.typ writing.typ orphan.typ

test -s "$tmp/unreachable/public/index.html"
test -s "$tmp/unreachable/public/colophon/index.html"
test -s "$tmp/unreachable/public/posts/foo/index.html"
test -s "$tmp/unreachable/public/writing/index.html"

if [ -e "$tmp/unreachable/public/orphan/index.html" ]; then
  echo 'sources outside the ownership tree must be dropped silently, not published' >&2
  exit 1
fi

# Fixture: a publication in the rendered set publishes a target that doesn't exist.
# Constraint: ownership edges from rendered publications must resolve.
mkdir -p "$tmp/dangling-target/html" "$tmp/dangling-target/public"
cp -R build/html/. "$tmp/dangling-target/html/"
printf '<!DOCTYPE html><html><body><h2>Index</h2><cairnz-nav data-target="writing.typ">Writing</cairnz-nav><cairnz-publish data-target="missing.typ"></cairnz-publish></body></html>' > "$tmp/dangling-target/html/index.html"

if scryer-prolog site.pl -- src "$tmp/dangling-target/html" "$tmp/dangling-target/public" index.typ colophon.typ index.typ posts/foo.typ writing.typ >/dev/null 2>&1; then
  echo 'ownership edges from rendered publications must point to known sources' >&2
  exit 1
fi

# Fixture: a rendered publication links to a candidate that is silently dropped.
# Constraint: reference edges from rendered publications must point into the
# rendered set, since dropped candidates have no route.
mkdir -p "$tmp/dangling-link/html" "$tmp/dangling-link/public"
cp -R build/html/. "$tmp/dangling-link/html/"
printf '<!DOCTYPE html><html><body><h2>Orphan</h2></body></html>' > "$tmp/dangling-link/html/orphan.html"
printf '<!DOCTYPE html><html><body><h2>Colophon</h2><p><cairnz-link data-target="orphan.typ">Orphan</cairnz-link></p></body></html>' > "$tmp/dangling-link/html/colophon.html"

if scryer-prolog site.pl -- src "$tmp/dangling-link/html" "$tmp/dangling-link/public" index.typ colophon.typ index.typ posts/foo.typ writing.typ orphan.typ >/dev/null 2>&1; then
  echo 'reference edges from rendered publications must point into the rendered set' >&2
  exit 1
fi

# Fixture: posts/foo is owned by both writing (#nav) and index (#publish).
# Constraint: every publication except the root has exactly one owner.
mkdir -p "$tmp/multi-owner/html" "$tmp/multi-owner/public/posts/foo"
cp -R build/html/. "$tmp/multi-owner/html/"
printf '<cairnz-publish data-target="posts/foo.typ"></cairnz-publish>' >> "$tmp/multi-owner/html/index.html"

if scryer-prolog site.pl -- src "$tmp/multi-owner/html" "$tmp/multi-owner/public" index.typ colophon.typ index.typ posts/foo.typ writing.typ >/dev/null 2>&1; then
  echo 'publications with multiple owners must fail ownership validation' >&2
  exit 1
fi

# Fixture: the root publication is configurable, not hard-coded to index.typ.
# Constraint: any source can serve as the root; route resolution adapts.
mkdir -p "$tmp/alternate-root/html" "$tmp/alternate-root/public/index"
printf '<!DOCTYPE html><html><body><cairnz-nav data-target="index.typ">Home</cairnz-nav><p><cairnz-link data-target="index.typ">Home</cairnz-link></p></body></html>' > "$tmp/alternate-root/html/writing.html"
printf '<!DOCTYPE html><html><body><p><cairnz-link data-target="writing.typ">Root</cairnz-link></p></body></html>' > "$tmp/alternate-root/html/index.html"

scryer-prolog site.pl -- src "$tmp/alternate-root/html" "$tmp/alternate-root/public" writing.typ writing.typ index.typ

test -s "$tmp/alternate-root/public/index.html"
test -s "$tmp/alternate-root/public/index/index.html"
grep -q '<a href="/index/">Home</a>' "$tmp/alternate-root/public/index.html"
grep -q '<a href="/">Root</a>' "$tmp/alternate-root/public/index/index.html"
