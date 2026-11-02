#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
bb_baseline
for i in $(seq 20); do post /p/BASE/issues "title=n+$i" >/dev/null; done        # BASE-3 .. BASE-22
post /p/BASE/issues 'title=under_score+test&body=a%2Cb&priority=2&labels=ui' >/dev/null   # BASE-23
H='key,title,status,priority,assignee,labels,body'
L1='BASE-1,Login fails,closed,1,alice,bug;ui,Steps: **click**'
L2='BASE-2,Typo on home,open,3,bob,,50% of the text'
L23='BASE-23,under_score test,open,2,,ui,"a,b"'
n() { local i; for i in "$@"; do echo "BASE-$((i + 2)),n $i,open,3,,,"; done; }
# csv_is QUERY LINE...: the export with that query is exactly these lines, each ending \n.
csv_is() {
  local q="$1"; shift
  expect_code 200 "/p/BASE/issues.csv$q"
  getf "/p/BASE/issues.csv$q" "$RUN/got"; printf '%s\n' "$@" >"$RUN/want"
  cmp -s "$RUN/got" "$RUN/want" || fail "issues.csv$q: want [$(cat "$RUN/want")] got [$(cat "$RUN/got")]"
}
mapfile -t ALLN < <(n $(seq 20))
csv_is "" "$H" "$L1" "$L2" "${ALLN[@]}" "$L23"
csv_is "?status=open" "$H" "$L2" "${ALLN[@]}" "$L23"
csv_is "?page=2" "$H" "$L1" "$L2" "${ALLN[@]}" "$L23"
csv_is "?status=closed&page=2" "$H" "$L1"
csv_is "?q=_" "$H" "$L23"
csv_is "?q=50%25" "$H" "$L2"
csv_is "?q=N+1" "$H" "${ALLN[0]}" "${ALLN[@]:9:10}"   # n 1, n 10 .. n 19
csv_is "?label=bug" "$H" "$L1"
csv_is "?label=ui&priority=2" "$H" "$L23"
csv_is "?assignee=bob" "$H" "$L2"
csv_is "?assignee=nobody" "$H"
csv_is "?label=nope" "$H"
csv_is "?priority=3&status=open&q=n" "$H" "$L2" "${ALLN[@]}"
expect_eq "$(ctype '/p/BASE/issues.csv?status=closed')" "text/csv; charset=utf-8" "content type"

expect_err 400 "bad status" '/p/BASE/issues.csv?status=done'
expect_err 400 "bad priority" '/p/BASE/issues.csv?priority=5'
expect_err 400 "bad priority" '/p/BASE/issues.csv?priority=x'
expect_err 400 "bad page" '/p/BASE/issues.csv?page=0'
expect_err 400 "bad page" '/p/BASE/issues.csv?page=abc'
expect_err 400 "bad status" '/p/BASE/issues.csv?page=0&status=done'
expect_err 400 "bad priority" '/p/BASE/issues.csv?page=0&priority=9'
expect_err 404 "no such project: NOPE" '/p/NOPE/issues.csv?status=done'

# the paginated routes are unchanged
expect_eq "$(get '/p/BASE/issues.txt?status=open' | wc -l | tr -d ' ')" 20 "txt still paginated"
expect_eq "$(get '/p/BASE/issues.txt?status=open&page=2')" $'BASE-22 open P3 - [] n 20\nBASE-23 open P2 - [ui] under_score test' "txt page 2"
expect_has "$(get /p/BASE)" 'Page 1 of 2 (23 issues)' "pager"

bb_restart
csv_is "?status=closed" "$H" "$L1"
csv_is "?q=_" "$H" "$L23"
csv_is "" "$H" "$L1" "$L2" "${ALLN[@]}" "$L23"
bb_unit
echo PASS
