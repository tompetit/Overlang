#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
bb_baseline
add "Buy milk"; add "write report"; add "MILKshake recipe"
expect_eq "$(post /search q=milk)" $'3 [ ] Buy milk\n5 [ ] MILKshake recipe' "search milk"
post /toggle id=5 >/dev/null
expect_eq "$(post /search q=MILK)" $'3 [ ] Buy milk\n5 [x] MILKshake recipe' "search MILK"
expect_eq "$(post /search q=report)" "4 [ ] write report" "search report"
expect_eq "$(post /search q=nothing)" "" "no match"
expect_code 200 /search q=nothing
expect_code 400 /search q=
expect_code 400 /search q=++
expect_eq "$(post /search q=)" "query required" "blank body"
expect_eq "$(post /search q=%26)" "2 [ ] write & ship!" "decoded query"
case "$(curl -s -o /dev/null -w '%{content_type}' -d q=milk "$URL/search")" in text/plain*) ;; *) fail "content type";; esac
expect_code 404 /search
bb_unit
echo PASS
