#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
bb_baseline
add "after baseline"
expect_has "$(list)" "after baseline" "add still works"
bb_unit
if [ "$LANG_" = overlang ]; then
  (cd "$RUN" && "$OV" build "$APP/main.ov" "$TASK/probe_trap.ov" --entry probe_main -o "$RUN/probe") >"$RUN/probe_build.log" 2>&1 \
    || { cat "$RUN/probe_build.log" >&2; fail "probe build failed"; }
  set +e; (cd "$RUN" && ./probe >"$RUN/probe.out" 2>"$RUN/probe.err"); rc=$?; set -e
  [ "$rc" = 3 ] || { cat "$RUN/probe.out" "$RUN/probe.err" >&2; fail "bad-handle add_todo should trap with exit 3, got $rc"; }
  grep -Eq '"kind": *"ensures"' "$RUN/probe.err" || { cat "$RUN/probe.err" >&2; fail "trap kind should be ensures"; }
else
  bb_probe
fi
echo PASS
