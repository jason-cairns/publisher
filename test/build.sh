#!/bin/sh
set -eu

test -s build/html/index.html
test -s public/index.html
test -s public/site.pdf

grep -q '<body>' build/html/index.html
grep -q '<main>' public/index.html
grep -q 'standalone Typst publication' public/index.html

if grep -qi '<script' public/index.html; then
  echo 'public/index.html must not include JavaScript' >&2
  exit 1
fi

