#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
# local helpers (lib.sh has none for these)
ok_post() { local c; c="$(code "$1" "$2")"; [ "$c" = 200 ] || fail "POST $1 [$2]: want 200, got $c"; }
expect_get() { expect_eq "$(get "$1")" "$2" "GET $1"; }
bb_baseline
ok_post /users name=Carl
ok_post /p/BASE/issues 'title=apple+pie&priority=2&assignee=bob'
ok_post /p/BASE/issues 'title=Zebra&priority=1'
ok_post /p/BASE/issues 'title=Login+fails&priority=3&assignee=alice'
ok_post /p/BASE/issues 'title=banana&priority=4&assignee=Carl'
ok_post /p/BASE/issues 'title=mango&priority=2'
L=("" 'BASE-1 closed P1 alice [bug,ui] Login fails' 'BASE-2 open P3 bob [] Typo on home' 'BASE-3 open P2 bob [] apple pie'
   'BASE-4 open P1 - [] Zebra' 'BASE-5 open P3 alice [] Login fails' 'BASE-6 open P4 Carl [] banana' 'BASE-7 open P2 - [] mango')
for n in $(seq 8 23); do L+=("BASE-$n open P4 - [] n $n"); done
# lines N...: the expected issues.txt for those issue numbers
lines() { local out="" n; for n in "$@"; do out+="${L[$n]}"$'\n'; done; printf '%s' "${out%$'\n'}"; }
# default order unchanged
expect_get /p/BASE/issues.txt "$(lines 1 2 3 4 5 6 7)"
expect_get '/p/BASE/issues.txt?sort=' "$(lines 1 2 3 4 5 6 7)"
expect_get '/p/BASE/issues.txt?sort=number' "$(lines 1 2 3 4 5 6 7)"
expect_get '/p/BASE/issues.txt?sort=-number' "$(lines 7 6 5 4 3 2 1)"
expect_get '/p/BASE/issues.txt?sort=priority' "$(lines 1 4 3 7 2 5 6)"
expect_get '/p/BASE/issues.txt?sort=-priority' "$(lines 6 2 5 3 7 1 4)"
expect_get '/p/BASE/issues.txt?sort=title' "$(lines 1 5 2 4 3 6 7)"
expect_get '/p/BASE/issues.txt?sort=-title' "$(lines 7 6 3 4 2 1 5)"
expect_get '/p/BASE/issues.txt?sort=assignee' "$(lines 6 1 5 2 3 4 7)"
expect_get '/p/BASE/issues.txt?sort=-assignee' "$(lines 2 3 1 5 6 4 7)"
expect_get '/p/BASE/issues.txt?status=open&sort=-priority' "$(lines 6 2 5 3 7 4)"
expect_get '/p/BASE/issues.txt?sort=title&q=login' "$(lines 1 5)"
expect_get '/p/BASE/issues.txt?sort=-assignee&assignee=alice' "$(lines 1 5)"
# errors
for bad in prio Priority --number -- - 'number,title' '-%20title' 'x'; do
  expect_err 400 "bad sort" "/p/BASE/issues.txt?sort=$bad"
done
expect_err 400 "bad sort" '/p/BASE?sort=desc'
expect_err 400 "bad status" '/p/BASE?status=done&sort=x'
expect_err 400 "bad priority" '/p/BASE/issues.txt?sort=x&priority=9'
expect_err 400 "bad page" '/p/BASE?page=0&sort=x'
expect_err 404 "no such project: NOPE" '/p/NOPE?sort=x'
# pagination and write-back
for n in $(seq 8 23); do ok_post /p/BASE/issues "title=n+$n&priority=4"; done
expect_get '/p/BASE/issues.txt?sort=-priority&page=2' "$(lines 7 1 4)"
expect_get '/p/BASE/issues.txt?sort=priority&page=2' "$(lines 21 22 23)"
expect_eq "$(get '/p/BASE/issues.txt?sort=-priority' | head -n 3)" "$(lines 6 8 9)" "page 1 head"
page="$(get '/p/BASE?sort=-priority')"
expect_has "$page" 'Page 1 of 2 (23 issues)' "pager"
expect_has "$page" ' <a href="/p/BASE?sort=-priority&amp;page=2">next</a>' "next link"
expect_has "$page" '<input name="q" value="" placeholder="search"> <input name="sort" value="-priority" placeholder="sort"> <button>filter</button>' "sort input"
page="$(get '/p/BASE?status=open&sort=-priority&page=2')"
expect_has "$page" 'Page 2 of 2 (22 issues)' "pager with status"
expect_has "$page" ' <a href="/p/BASE?status=open&amp;sort=-priority&amp;page=1">prev</a>' "prev link"
expect_has "$page" '<a href="/p/BASE/4">BASE-4</a>' "page 2 row"
page="$(get '/p/BASE?q=n&sort=title')"
expect_has "$page" ' <a href="/p/BASE?q=n&amp;sort=title&amp;page=2">next</a>' "q before sort"
page="$(get '/p/BASE?sort=number')"
expect_has "$page" '<input name="sort" value="" placeholder="sort">' "default sort input"
expect_has "$page" ' <a href="/p/BASE?page=2">next</a>' "default next link"
page="$(get /p/BASE)"
expect_has "$page" '<input name="q" value="" placeholder="search"> <input name="sort" value="" placeholder="sort"> <button>filter</button>' "form"
expect_has "$page" ' <a href="/p/BASE?page=2">next</a>' "plain next link"
expect_eq "$(get /p/BASE/issues.csv | sed -n 2p | cut -d, -f1)" BASE-1 "csv order"
expect_eq "$(get /p/BASE/issues.csv | tail -n 1 | cut -d, -f1)" BASE-23 "csv order"
bb_restart
expect_eq "$(get '/p/BASE/issues.txt?sort=assignee' | head -n 3)" "$(lines 6 1 5)" "sort after restart"
expect_get '/p/BASE/issues.txt?sort=-title&status=closed' "$(lines 1)"
bb_unit
echo PASS
