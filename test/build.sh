#!/bin/sh
set -eu

test -s build/html/index.html
test -s build/html/writing.html
test -s build/html/posts/foo.html
test -s public/index.html
test -s public/writing/index.html
test -s public/posts/foo/index.html
test -s public/colophon/index.html
test -s public/site.pdf

grep -q '<body>' build/html/index.html
grep -q '<cairnz-nav data-target="writing.typ">Writing</cairnz-nav>' build/html/index.html
grep -q '<main>' public/index.html
grep -q 'standalone Typst publication' public/index.html
grep -q '<nav>' public/index.html
grep -q '<a href="/writing/">Writing</a>' public/index.html
grep -q '<a href="/posts/foo/">Foo</a>' public/writing/index.html
grep -q '<a href="/posts/foo/">Foo</a>' public/posts/foo/index.html
grep -q 'published without becoming' public/colophon/index.html

if grep -R -q '<cairnz-' public; then
  echo 'public HTML must not include internal publication markers' >&2
  exit 1
fi

if grep -q '<a href="/colophon/">colophon</a></li>' public/index.html; then
  echo 'published-only pages must not become visible nav items' >&2
  exit 1
fi

if grep -qi '<script' public/index.html; then
  echo 'public/index.html must not include JavaScript' >&2
  exit 1
fi

mkdir -p build/test-preview
${TYPST:?TYPST is required} compile --root src src/index.typ build/test-preview/index.pdf

tmp="build/test-ownership"
rm -rf "$tmp"
mkdir -p "$tmp/unreachable/html" "$tmp/unreachable/public/orphan"
cp -R build/html/. "$tmp/unreachable/html/"
printf '<!DOCTYPE html><html><body><h2>Orphan</h2></body></html>' > "$tmp/unreachable/html/orphan.html"

if scryer-prolog site.pl -- src "$tmp/unreachable/html" "$tmp/unreachable/public" index.typ colophon.typ index.typ posts/foo.typ writing.typ orphan.typ >/dev/null 2>&1; then
  echo 'unreachable publication candidates must fail ownership validation' >&2
  exit 1
fi

mkdir -p "$tmp/multi-owner/html" "$tmp/multi-owner/public/posts/foo"
cp -R build/html/. "$tmp/multi-owner/html/"
printf '<cairnz-publish data-target="posts/foo.typ"></cairnz-publish>' >> "$tmp/multi-owner/html/index.html"

if scryer-prolog site.pl -- src "$tmp/multi-owner/html" "$tmp/multi-owner/public" index.typ colophon.typ index.typ posts/foo.typ writing.typ >/dev/null 2>&1; then
  echo 'publications with multiple owners must fail ownership validation' >&2
  exit 1
fi

mkdir -p "$tmp/alternate-root/html" "$tmp/alternate-root/public/index"
printf '<!DOCTYPE html><html><body><cairnz-nav data-target="index.typ">Home</cairnz-nav><p><cairnz-link data-target="index.typ">Home</cairnz-link></p></body></html>' > "$tmp/alternate-root/html/writing.html"
printf '<!DOCTYPE html><html><body><p><cairnz-link data-target="writing.typ">Root</cairnz-link></p></body></html>' > "$tmp/alternate-root/html/index.html"

scryer-prolog site.pl -- src "$tmp/alternate-root/html" "$tmp/alternate-root/public" writing.typ writing.typ index.typ

test -s "$tmp/alternate-root/public/index.html"
test -s "$tmp/alternate-root/public/index/index.html"
grep -q '<a href="/index/">Home</a>' "$tmp/alternate-root/public/index.html"
grep -q '<a href="/">Root</a>' "$tmp/alternate-root/public/index/index.html"
