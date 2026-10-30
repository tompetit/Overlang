#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
add "a"; add "b"
expect_list $'1 [ ] a\n2 [ ] b'
post /toggle id=1 >/dev/null
expect_list $'1 [~] a\n2 [ ] b'
expect_eq "$(bb_sql 'select status from todos where id = 1')" doing "status column"
expect_has "$(get /)" "[~] a" "page"
bb_restart
expect_list $'1 [~] a\n2 [ ] b'
post /toggle id=1 >/dev/null
expect_list $'1 [x] a\n2 [ ] b'
expect_has "$(get /)" "[x] a" "page"
expect_eq "$(bb_sql 'select status from todos where id = 1')" done "status column"
post /toggle id=1 >/dev/null
expect_list $'1 [ ] a\n2 [ ] b'
expect_eq "$(bb_sql 'select status from todos where id = 2')" open "default status"
post /toggle id=2 >/dev/null; post /toggle id=2 >/dev/null
expect_list $'1 [ ] a\n2 [x] b'
post /delete id=1 >/dev/null
expect_list '2 [x] b'
expect_code 400 /toggle id=0
expect_code 400 /add title=
expect_code 404 /nope
bb_unit
echo PASS
