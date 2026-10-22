#!/usr/bin/env bash
# Keep compiler output outside the checkout that actions/checkout cleans between jobs.
set -euo pipefail
cd "$(dirname "$0")/../.."
tce_cache_root="${HOME:?}/.cache/tce-build"
for tce_workspace in commons kernel; do
  mkdir -p "$tce_cache_root/$tce_workspace"
  if [[ -e "$tce_workspace/target" || -L "$tce_workspace/target" ]]; then
    if [[ $(readlink "$tce_workspace/target") != "$tce_cache_root/$tce_workspace" ]]; then
      echo "Refusing to replace an existing $tce_workspace/target directory." >&2
      exit 1
    fi
  else
    ln -s "$tce_cache_root/$tce_workspace" "$tce_workspace/target"
  fi
done
