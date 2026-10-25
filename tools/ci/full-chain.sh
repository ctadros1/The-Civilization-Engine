#!/usr/bin/env bash
# Linux counterpart of CI's Windows lane, plus the web and schema checks.
set -euo pipefail
cd "$(dirname "$0")/../.."
if (( $(df -Pk . | awk 'NR == 2 {print $4}') < 20 * 1024 * 1024 )); then
  echo 'The full chain needs at least 20 GiB free in its work directory.' >&2
  exit 1
fi
for workspace in commons kernel; do
  (
    cd "$workspace"
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets --locked -- -D warnings
    if [[ $workspace == kernel ]]; then
      cargo test --workspace --locked --profile simcheck
    else
      cargo test --workspace --locked
    fi
  )
done
(cd kernel; cargo run --locked --profile simcheck -p civ-host -- content validate; cargo run --locked --profile simcheck -p civ-host -- smoke)
FLATC=$(command -v flatc) tools/gen-schema.sh --check
(cd web; npm ci; npm run build; npm test)
(cd kernel; cargo build --locked --release -p civ-host)
(cd web; npx playwright install chromium; npm run test:e2e)
