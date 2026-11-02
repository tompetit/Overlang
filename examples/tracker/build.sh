#!/usr/bin/env bash
# Usage: build.sh DIR. Creates a fresh store in DIR, adds the four modules in dependency order,
# then checks, tests (with fuzzing) and builds DIR/tracker from the stored `main`.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../.." && pwd)"
ov="${OV:-$root/target/release/ov}"
dir="${1:?usage: build.sh DIR}"
mkdir -p "$dir"
cd "$dir"
rm -rf .ovstore .ovbuild   # fresh store; .ovbuild caches C built from the old shims
for f in 01_db 02_models 03_render 04_routes; do
  "$ov" add "$here/src/$f.ov"
done
"$ov" check
"$ov" test
"$ov" build main -o tracker
echo "built $(pwd)/tracker"
