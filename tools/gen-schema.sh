#!/usr/bin/env bash
# Regenerates the FlatBuffers code for the boundary and save schemas (ADR-0001).
#
#   tools/gen-schema.sh            regenerate
#   tools/gen-schema.sh --check    regenerate into a temp dir and fail if the committed code differs
#
# Needs flatc 25.12.19 (the version must match the Rust `flatbuffers` runtime). Set FLATC to its
# path if it is not on PATH. Generated files are committed so ordinary builds never need flatc.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
flatc="${FLATC:-flatc}"
expected="25.12.19"
schema_dir="$root/kernel/crates/civ-schema/schema"
rust_out="$root/kernel/crates/civ-schema/src/generated"
ts_out="$root/web/src/schema/generated"

version="$("$flatc" --version | awk '{print $3}')"
if [[ "$version" != "$expected" ]]; then
  echo "flatc $expected required, found $version ($flatc)" >&2
  exit 2
fi

generate() {
  local rust_dir="$1" ts_dir="$2"
  rm -rf "$rust_dir" "$ts_dir"
  mkdir -p "$rust_dir" "$ts_dir"
  "$flatc" --rust --gen-all -o "$rust_dir" "$schema_dir/tce_wire.fbs" "$schema_dir/tce_save.fbs"
  "$flatc" --ts --gen-all -o "$ts_dir" "$schema_dir/tce_wire.fbs"
}

if [[ "${1:-}" == "--check" ]]; then
  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' EXIT
  generate "$tmp/rust" "$tmp/ts"
  diff -r "$tmp/rust" "$rust_out" && diff -r "$tmp/ts" "$ts_out" || {
    echo "generated schema code is stale: run tools/gen-schema.sh and commit the result" >&2
    exit 1
  }
  echo "generated schema code is up to date"
else
  generate "$rust_out" "$ts_out"
  echo "regenerated $rust_out and $ts_out"
fi
