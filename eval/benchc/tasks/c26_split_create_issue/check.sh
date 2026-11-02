#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
bb_baseline
x120="$(printf 'x%.0s' $(seq 120))"
e120="$(printf '%%C3%%A9%.0s' $(seq 120))"   # 120 x "é", form-encoded (240 bytes)
eacc="$(printf 'é%.0s' $(seq 120))"
base2=$'BASE-1 closed P1 alice [bug,ui] Login fails\nBASE-2 open P3 bob [] Typo on home'

# create: the new check, its position in the order, and nothing stored on failure
expect_err 422 "title too long" /p/BASE/issues "title=${x120}y"
expect_err 422 "title too long" /p/BASE/issues "title=${e120}%C3%A9"
expect_err 422 "title too long" /p/BASE/issues "title=%20%20${x120}y%20%20"
expect_err 422 "title too long" /p/BASE/issues "title=${x120}y&priority=9"
expect_err 422 "title too long" /p/BASE/issues "title=${x120}y&priority=abc"
expect_err 422 "title too long" /p/BASE/issues "title=${x120}y&assignee=nobody&labels=nope"
expect_err 404 "no such project: NOPE" /p/NOPE/issues "title=${x120}y"
expect_err 422 "title required" /p/BASE/issues "title=%20%20&priority=9"
expect_err 400 "bad priority" /p/BASE/issues "title=ok&priority=abc"
expect_err 422 "bad priority" /p/BASE/issues "title=ok&priority=9"
expect_err 422 "no such user: nobody" /p/BASE/issues "title=ok&assignee=nobody"
expect_err 422 "no such label: nope" /p/BASE/issues "title=ok&labels=ui,nope"
expect_eq "$(get /p/BASE/issues.txt)" "$base2" "rejected creates stored nothing"

# create: the limit is inclusive, in code points, after trimming
expect_code 200 /p/BASE/issues "title=%20%20${x120}%20%20"
expect_code 200 /p/BASE/issues "title=${e120}&assignee=alice&labels=ui&priority=2"
expect_eq "$(get /p/BASE/issues.txt)" "$base2"$'\n'"BASE-3 open P3 - [] ${x120}"$'\n'"BASE-4 open P2 alice [ui] ${eacc}" "issues.txt"
expect_has "$(get /p/BASE/4)" "<h1>BASE-4: ${eacc}</h1>" "issue page"

# edit
expect_err 422 "title too long" /p/BASE/3/edit "title=${x120}y"
expect_err 422 "title too long" /p/BASE/3/edit "title=${x120}y&priority=abc"
expect_err 422 "title too long" /p/BASE/3/edit "title=${x120}y&priority=2&body=changed"
expect_err 422 "title too long" /p/BASE/4/edit "title=${e120}%C3%A9&body=changed"
expect_err 404 "no such issue: BASE-99" /p/BASE/99/edit "title=${x120}y"
expect_err 400 "bad priority" /p/BASE/3/edit "title=ok&priority=abc"
expect_err 422 "bad priority" /p/BASE/3/edit "title=ok&priority=0"
expect_eq "$(get /p/BASE/3.txt)" "BASE-3 open P3 - [] ${x120}" "rejected edit kept issue 3"
expect_has "$(get /p/BASE/4.json)" '"body":"","status":"open","priority":2' "rejected edit kept issue 4"
expect_code 200 /p/BASE/3/edit "title=&priority=2"
expect_eq "$(get /p/BASE/3.txt)" "BASE-3 open P2 - [] ${x120}" "empty title keeps the old one"
expect_code 200 /p/BASE/3/edit "title=%20${x120}%20&body=b"
expect_code 200 /p/BASE/4/edit "title=Short+one"
expect_eq "$(get /p/BASE/4.txt)" "BASE-4 open P2 alice [ui] Short one" "edit"
expect_code 200 /p/BASE/4/edit "title=${e120}"
expect_eq "$(get /p/BASE/3/activity.txt)" "created" "edits log nothing"
expect_eq "$(get /p/BASE/4/activity.txt)" "created" "edits log nothing"

bb_restart
expect_eq "$(get /p/BASE/issues.txt)" "$base2"$'\n'"BASE-3 open P2 - [] ${x120}"$'\n'"BASE-4 open P2 alice [ui] ${eacc}" "after restart"
expect_has "$(get /p/BASE/3.json)" '"body":"b"' "body after restart"
expect_err 422 "title too long" /p/BASE/issues "title=${x120}yz"
bb_unit
echo PASS
