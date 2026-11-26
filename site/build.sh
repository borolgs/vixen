#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

bun run --cwd ../examples/vixen-preview build
rm -rf static/demo
mkdir -p static/demo
cp -R ../examples/vixen-preview/dist/. static/demo/
# The homepage embeds the demo; omit its standalone page and duplicate theme.
rm static/demo/index.html static/demo/theme.css
zola build "$@"
