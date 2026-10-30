#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
bb_baseline
add "x <y>"
expect_has "$(get /)" "x &lt;y&gt;" "page"
bb_unit
bb_probe
if [ "$LANG_" = overlang ]; then
  ! grep -E '^fn respond_page\(.*!fs' "$APP/main.ov" >/dev/null || fail "respond_page still declares !fs"
fi
echo PASS
