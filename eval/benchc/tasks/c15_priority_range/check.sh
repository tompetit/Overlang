#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
# local helpers (lib.sh has none for these)
ok_post() { local c; c="$(code "$1" "$2")"; [ "$c" = 200 ] || fail "POST $1 [$2]: want 200, got $c"; }
expect_get() { expect_eq "$(get "$1")" "$2" "GET $1"; }
bb_baseline
ok_post /p/BASE/issues 'title=Two&priority=2'
ok_post /p/BASE/issues 'title=Four&priority=4'
ok_post /p/BASE/issues 'title=Two+again&priority=2&labels=bug'
L=("" 'BASE-1 closed P1 alice [bug,ui] Login fails' 'BASE-2 open P3 bob [] Typo on home' 'BASE-3 open P2 - [] Two'
   'BASE-4 open P4 - [] Four' 'BASE-5 open P2 - [bug] Two again')
for n in $(seq 6 25); do L+=("BASE-$n open P1 - [] p $n"); done
lines() { local out="" n; for n in "$@"; do out+="${L[$n]}"$'\n'; done; printf '%s' "${out%$'\n'}"; }
# single priority unchanged
expect_get '/p/BASE/issues.txt?priority=2' "$(lines 3 5)"
expect_get '/p/BASE/issues.txt?priority=' "$(lines 1 2 3 4 5)"
expect_has "$(get '/p/BASE?priority=2')" '<input name="priority" value="2" placeholder="priority">' "single value"
# ranges
expect_get '/p/BASE/issues.txt?priority=1-2' "$(lines 1 3 5)"
expect_get '/p/BASE/issues.txt?priority=2-3' "$(lines 2 3 5)"
expect_get '/p/BASE/issues.txt?priority=3-4' "$(lines 2 4)"
expect_get '/p/BASE/issues.txt?priority=1-4' "$(lines 1 2 3 4 5)"
expect_get '/p/BASE/issues.txt?priority=4-4' "$(lines 4)"
expect_get '/p/BASE/issues.txt?priority=+1-2+' "$(lines 1 3 5)"
expect_get '/p/BASE/issues.txt?priority=1%2D2' "$(lines 1 3 5)"
expect_get '/p/BASE/issues.txt?priority=1-2&status=open' "$(lines 3 5)"
expect_get '/p/BASE/issues.txt?priority=1-2&label=bug' "$(lines 1 5)"
expect_get '/p/BASE/issues.txt?priority=2-4&q=two' "$(lines 3 5)"
expect_get '/p/BASE/issues.txt?priority=2-4&assignee=alice' ""
expect_has "$(get '/p/BASE?priority=1-2')" 'Page 1 of 1 (3 issues)' "pager total"
# errors
for bad in 3-1 0-2 1-5 1- -2 1-2-3 '1+-+2' 0 5 a-b 1-x 1--2 '2-%201' '1,2' 12 '-'; do
  expect_err 400 "bad priority" "/p/BASE/issues.txt?priority=$bad"
done
expect_err 400 "bad priority" '/p/BASE?priority=4-1'
expect_err 400 "bad status" '/p/BASE?status=done&priority=3-1'
expect_err 400 "bad priority" '/p/BASE?priority=3-1&page=0'
expect_err 404 "no such project: NOPE" '/p/NOPE?priority=3-1'
# writes still take one priority
expect_err 400 "bad priority" /p/BASE/issues 'title=zz&priority=1-2'
expect_err 400 "bad priority" /p/BASE/issues 'title=zz&priority=2-2'
expect_err 400 "bad priority" /p/BASE/2/edit 'priority=1-2'
expect_err 422 "bad priority" /p/BASE/issues 'title=zz&priority=9'
expect_get '/p/BASE/issues.txt?q=zz' ""
expect_get /p/BASE/2.txt "$(lines 2)"
# pagination and write-back
for n in $(seq 6 25); do ok_post /p/BASE/issues "title=p+$n&priority=1"; done
page="$(get '/p/BASE?priority=1-2')"
expect_has "$page" '<input name="priority" value="1-2" placeholder="priority">' "range value"
expect_has "$page" 'Page 1 of 2 (23 issues)' "pager"
expect_has "$page" ' <a href="/p/BASE?priority=1-2&amp;page=2">next</a>' "next link"
expect_get '/p/BASE/issues.txt?priority=1-2&page=2' "$(lines 23 24 25)"
page="$(get '/p/BASE?priority=1-1&page=2')"
expect_has "$page" '<input name="priority" value="1" placeholder="priority">' "A==B value"
expect_has "$page" 'Page 2 of 2 (21 issues)' "pager A==B"
expect_has "$page" ' <a href="/p/BASE?priority=1&amp;page=1">prev</a>' "A==B link"
expect_get '/p/BASE/issues.txt?priority=1-1&page=2' "$(lines 25)"
page="$(get '/p/BASE?status=open&priority=1-3&q=p')"
expect_has "$page" 'Page 1 of 2 (21 issues)' "pager combined"
expect_has "$page" ' <a href="/p/BASE?status=open&amp;priority=1-3&amp;q=p&amp;page=2">next</a>' "combined link"
expect_has "$(get '/p/BASE?priority=3-3')" '<input name="priority" value="3" placeholder="priority">' "3-3 value"
bb_restart
expect_get '/p/BASE/issues.txt?priority=2-3' "$(lines 2 3 5)"
expect_get '/p/BASE/issues.txt?priority=1-2&page=2' "$(lines 23 24 25)"
bb_unit
echo PASS
