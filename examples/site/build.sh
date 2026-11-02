#!/usr/bin/env bash
# Builds the site generator from html.ov and main.ov and writes docs/index.html and docs/site.css.
# The prelude has no file writing, so the program prints each file to stdout and this script redirects it.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../.." && pwd)"
ov="${OV:-$root/target/release/ov}"
cd "$root"   # .ovbuild/ lands in the repo root, which is gitignored
"$ov" test examples/site/html.ov examples/site/main.ov
"$ov" build examples/site/html.ov examples/site/main.ov -o .ovbuild/site
.ovbuild/site > docs/index.html
.ovbuild/site css > docs/site.css
echo "wrote docs/index.html ($(wc -c < docs/index.html) bytes), docs/site.css ($(wc -c < docs/site.css) bytes)"
