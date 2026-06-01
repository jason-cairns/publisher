#!/usr/bin/env sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
manifest="$repo_root/typst/typst.toml"
lib="$repo_root/typst/lib.typ"

name=$(awk -F ' *= *' '$1 == "name" { gsub(/"/, "", $2); print $2; exit }' "$manifest")
version=$(awk -F ' *= *' '$1 == "version" { gsub(/"/, "", $2); print $2; exit }' "$manifest")

if [ -n "${PUBLISHER_TYPST_DATA_DIR:-}" ]; then
  data_dir=$PUBLISHER_TYPST_DATA_DIR
elif [ -n "${XDG_DATA_HOME:-}" ]; then
  data_dir=$XDG_DATA_HOME
else
  case "$(uname -s)" in
    Darwin)
      data_dir="$HOME/Library/Application Support"
      ;;
    MINGW*|MSYS*|CYGWIN*)
      data_dir=${APPDATA:?APPDATA is required on Windows}
      ;;
    *)
      data_dir="$HOME/.local/share"
      ;;
  esac
fi

dest="$data_dir/typst/packages/local/$name/$version"
mkdir -p "$dest"
cp "$manifest" "$lib" "$dest/"

printf 'Installed @local/%s:%s to %s\n' "$name" "$version" "$dest"
