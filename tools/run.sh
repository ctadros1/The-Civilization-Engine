#!/usr/bin/env bash
# Starts The Civilization Engine's observer: builds the web shell if it is missing or older than
# its sources, then runs civ-host (release build) and opens the browser.
#
#   tools/run.sh                  # http://127.0.0.1:7420/
#   tools/run.sh --port 7421      # any `civ-host serve` option
#
# Needs Rust (rustup reads rust-toolchain.toml) and Node.js 22 with npm.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
index="$root/web/dist/index.html"

if [[ ! -f "$index" ]] || [[ -n "$(find "$root/web/src" "$root/web/index.html" -newer "$index" -print -quit)" ]]; then
  echo "Building the web shell…"
  cd "$root/web"
  [[ -d node_modules ]] || npm ci
  npm run build
fi

cd "$root/kernel"
exec cargo run --release --locked -p civ-host -- serve --open "$@"
