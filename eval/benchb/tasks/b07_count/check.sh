#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
bb_baseline
expect_eq "$(get /count)" "1 todos, 0 done" "count after baseline"
add "a"; add "b"; add "c"
post /toggle id=2 >/dev/null; post /toggle id=3 >/dev/null
expect_eq "$(get /count)" "4 todos, 2 done" "count mixed"
expect_eq "$(curl -s -d x=1 -o /dev/null -w '%{http_code}' "$URL/count")" 404 "POST /count is not a route"
expect_code 200 /count
case "$(ctype /count)" in text/plain*) ;; *) fail "content type";; esac
[ "$(curl -s "$URL/count" | wc -l)" = 1 ] || fail "missing trailing newline"
post /delete id=2 >/dev/null; post /delete id=3 >/dev/null; post /delete id=4 >/dev/null; post /delete id=5 >/dev/null
expect_eq "$(get /count)" "0 todos, 0 done" "count empty"
bb_unit
echo PASS
