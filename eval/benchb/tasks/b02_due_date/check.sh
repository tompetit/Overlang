#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
add "pay rent" due=2026-01-31
add "no due"
add "empty due" due=
expect_list $'1 [ ] pay rent (due 2026-01-31)\n2 [ ] no due\n3 [ ] empty due'
for bad in tomorrow 2026-1-31 20260131 2026-01-3x 26-01-31 2026-01-311; do
  expect_code 400 /add "title=x&due=$bad"
done
expect_eq "$(post /add "title=x&due=nope")" "bad due date" "body"
expect_list $'1 [ ] pay rent (due 2026-01-31)\n2 [ ] no due\n3 [ ] empty due'
post /toggle id=1 >/dev/null
expect_list $'1 [x] pay rent (due 2026-01-31)\n2 [ ] no due\n3 [ ] empty due'
page="$(get /)"
expect_has "$page" "<small>due 2026-01-31</small>" "page"
expect_eq "$(grep -o '<small>' <<<"$page" | wc -l)" 1 "only the todo with a date has a <small>"
post /delete id=2 >/dev/null
expect_code 404 /nope
expect_code 400 /add title=
bb_restart
expect_list $'1 [x] pay rent (due 2026-01-31)\n3 [ ] empty due'
bb_unit
echo PASS
