#!/bin/sh
set -eu

exec uv run pytest "$@"
