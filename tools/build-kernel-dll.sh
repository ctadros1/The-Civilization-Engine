#!/usr/bin/env bash
# Builds the kernel library `tce_kernel` (ADR-0005) and publishes it with its C header and a
# manifest, for the Unreal plugin or any other host:
#
#   tools/build-kernel-dll.sh                 # into dist/tce_kernel/<target>/
#   tools/build-kernel-dll.sh --dest DIR      # into DIR (for example a plugin's Binaries folder)
#
# It first checks that the committed header matches the library's exports. On Windows, use
# tools\build-kernel-dll.ps1, which also publishes the PDB.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
dest=""
while [ $# -gt 0 ]; do
    case "$1" in
        --dest) dest="$2"; shift 2 ;;
        *) echo "unknown argument: $1 (only --dest DIR)" >&2; exit 2 ;;
    esac
done

cd "$root/kernel"
target="$(rustc -vV | sed -n 's/^host: //p')"
dest="${dest:-$root/dist/tce_kernel/$target}"
case "$(uname -s)" in
    Darwin) library=libtce_kernel.dylib ;;
    *) library=libtce_kernel.so ;;
esac
header=crates/civ-ffi/include/tce_kernel.h

echo "Checking the C header matches the exports..."
cargo test --locked --profile dll -p civ-ffi --test header --quiet
echo "Building $library for $target (profile dll)..."
cargo build --locked --profile dll -p civ-ffi

mkdir -p "$dest"
cp "target/dll/$library" "$header" "$dest/"
abi_major="$(sed -n 's/^#define TCE_ABI_MAJOR \([0-9]*\)$/\1/p' "$header")"
abi_minor="$(sed -n 's/^#define TCE_ABI_MINOR \([0-9]*\)$/\1/p' "$header")"
if command -v sha256sum > /dev/null; then
    sha256="$(sha256sum "$dest/$library" | cut -d' ' -f1)"
else
    sha256="$(shasum -a 256 "$dest/$library" | cut -d' ' -f1)"
fi
revision="$(git -C "$root" rev-parse HEAD 2> /dev/null || echo unknown)"
changed="$(git -C "$root" status --porcelain 2> /dev/null | wc -l | tr -d ' ')"
cat > "$dest/tce_kernel.json" << EOF
{
  "library": "$library",
  "abi": "$abi_major.$abi_minor",
  "target": "$target",
  "profile": "dll",
  "revision": "$revision",
  "uncommitted_changes": $changed,
  "sha256": "$sha256",
  "built": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF
echo "Published $library, tce_kernel.h and tce_kernel.json to $dest"
