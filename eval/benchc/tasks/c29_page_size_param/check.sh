#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
bb_baseline
for i in $(seq 19); do post /p/BASE/issues "title=n+$i" >/dev/null; done   # BASE-3 .. BASE-21: "n 1" .. "n 19"
row() { local k; for k in "$@"; do echo "BASE-$k open P3 - [] n $((k - 2))"; done; }
txt() { get "/p/BASE/issues.txt$1"; }
nlines() { txt "$1" | wc -l | tr -d ' '; }

# default size unchanged
expect_eq "$(nlines '')" 20 "default page"
expect_eq "$(txt '?page=2')" "$(row 21)" "default page 2"
page="$(get /p/BASE)"
expect_has "$page" 'Page 1 of 2 (21 issues)' "default pager"
expect_has "$page" 'href="/p/BASE?page=2">next</a>' "default next"
expect_lacks "$page" 'per_page' "default page mentions per_page"
page="$(get '/p/BASE?per_page=20')"
expect_has "$page" 'href="/p/BASE?page=2">next</a>' "explicit 20"
expect_lacks "$page" 'per_page' "explicit 20 mentions per_page"
expect_eq "$(nlines '?per_page=%20')" 20 "blank per_page"

# chosen sizes
expect_eq "$(nlines '?per_page=10')" 10 "per_page=10"
expect_eq "$(txt '?per_page=10' | tail -n 1)" "$(row 10)" "per_page=10 last"
expect_eq "$(txt '?per_page=10&page=3')" "$(row 21)" "per_page=10 page 3"
expect_code 200 '/p/BASE/issues.txt?per_page=10&page=4'
expect_eq "$(txt '?per_page=10&page=4')" "" "past the end"
expect_eq "$(nlines '?per_page=007')" 7 "leading zeros"
page="$(get '/p/BASE?per_page=10&page=2')"
expect_has "$page" 'Page 2 of 3 (21 issues)' "pager 10"
expect_has "$page" 'href="/p/BASE?per_page=10&amp;page=1">prev</a>' "prev 10"
expect_has "$page" 'href="/p/BASE?per_page=10&amp;page=3">next</a>' "next 10"
expect_has "$page" '<a href="/p/BASE/11">BASE-11</a>' "rows of page 2"
expect_has "$page" '<a href="/p/BASE/20">BASE-20</a>' "rows of page 2"
expect_lacks "$page" '<a href="/p/BASE/10">BASE-10</a>' "rows of page 2"
expect_lacks "$page" '<a href="/p/BASE/21">BASE-21</a>' "rows of page 2"
expect_has "$(get '/p/BASE?per_page=10&page=3')" 'Page 3 of 3 (21 issues)' "pager last"
expect_eq "$(nlines '?per_page=100')" 21 "per_page=100"
page="$(get '/p/BASE?per_page=100')"
expect_has "$page" 'Page 1 of 1 (21 issues)' "pager 100"
expect_lacks "$page" '>next</a>' "no next"
expect_lacks "$page" '>prev</a>' "no prev"
expect_eq "$(txt '?per_page=1&page=21')" "$(row 21)" "per_page=1"
page="$(get '/p/BASE?page=21&per_page=1')"
expect_has "$page" 'Page 21 of 21 (21 issues)' "pager 1"
expect_has "$page" 'href="/p/BASE?per_page=1&amp;page=20">prev</a>' "prev 1"
expect_lacks "$page" '>next</a>' "no next at end"

# with filters
expect_eq "$(txt '?status=open&per_page=5&page=2')" "$(row 7 8 9 10 11)" "filtered page"
page="$(get '/p/BASE?status=open&per_page=5&page=2')"
expect_has "$page" 'Page 2 of 4 (20 issues)' "filtered pager"
expect_has "$page" 'href="/p/BASE?status=open&amp;per_page=5&amp;page=1">prev</a>' "filtered prev"
expect_has "$page" 'href="/p/BASE?status=open&amp;per_page=5&amp;page=3">next</a>' "filtered next"
expect_eq "$(txt '?q=n+1&per_page=3')" "$(row 3 12 13)" "search page"
page="$(get '/p/BASE?q=n+1&per_page=3')"
expect_has "$page" 'Page 1 of 4 (11 issues)' "search pager"
expect_has "$page" 'href="/p/BASE?q=n%201&amp;per_page=3&amp;page=2">next</a>' "search next"

# errors, on both routes, in order
for path in /p/BASE /p/BASE/issues.txt; do
  for bad in 0 101 abc -1 %2B5 1234567890 5x; do
    expect_err 400 "bad per_page" "$path?per_page=$bad"
  done
  expect_err 400 "bad per_page" "$path?per_page=0&page=2"
  expect_err 400 "bad page" "$path?page=0&per_page=0"
  expect_err 400 "bad status" "$path?per_page=0&status=x"
  expect_err 400 "bad priority" "$path?priority=9&per_page=abc"
done
expect_err 404 "no such project: NOPE" '/p/NOPE?per_page=0'
expect_eq "$(get '/p/BASE/issues.csv?per_page=1' | wc -l | tr -d ' ')" 22 "csv unpaginated"

bb_restart
expect_eq "$(txt '?per_page=10&page=3')" "$(row 21)" "after restart"
expect_eq "$(nlines '')" 20 "default after restart"
bb_unit
echo PASS
