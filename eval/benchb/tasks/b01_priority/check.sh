#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
add "buy milk"
add "pay rent" priority=3
add "later" priority=1
add "empty prio" priority=
expect_list $'1 [ ] P2 buy milk\n2 [ ] P3 pay rent\n3 [ ] P1 later\n4 [ ] P2 empty prio'
for bad in 0 4 high -1 2x; do
  expect_code 400 /add "title=x&priority=$bad"
done
expect_eq "$(post /add "title=x&priority=7")" "bad priority" "body"
expect_code 400 /add "title=&priority=3"
expect_list $'1 [ ] P2 buy milk\n2 [ ] P3 pay rent\n3 [ ] P1 later\n4 [ ] P2 empty prio'
post /toggle id=2 >/dev/null
expect_list $'1 [ ] P2 buy milk\n2 [x] P3 pay rent\n3 [ ] P1 later\n4 [ ] P2 empty prio'
page="$(get /)"
expect_has "$page" "[x] P3 pay rent" "page"
expect_has "$page" "[ ] P1 later" "page"
post /delete id=1 >/dev/null
expect_code 404 /nope
bb_restart
expect_list $'2 [x] P3 pay rent\n3 [ ] P1 later\n4 [ ] P2 empty prio'
bb_unit
echo PASS
