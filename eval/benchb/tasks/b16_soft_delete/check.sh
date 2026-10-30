#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
add "a"; add "b"; add "c"
post /toggle id=2 >/dev/null
post /delete id=2 >/dev/null
expect_list $'1 [ ] a\n3 [ ] c'
expect_lacks "$(get /)" "<li>[x] b"
expect_eq "$(bb_sql 'select count(*) from todos')" 3 "row still there"
expect_eq "$(bb_sql 'select deleted from todos where id = 2')" 1 "deleted flag"
expect_eq "$(bb_sql 'select deleted from todos where id = 1')" 0 "other rows untouched"
expect_code 200 /delete id=2
expect_code 400 /undelete id=0
expect_eq "$(post /undelete id=x)" "bad id" "bad id body"
expect_code 200 /undelete id=99
expect_list $'1 [ ] a\n3 [ ] c'
bb_restart
expect_list $'1 [ ] a\n3 [ ] c'
expect_code 200 /undelete id=2
expect_list $'1 [ ] a\n2 [x] b\n3 [ ] c'
expect_has "$(get /)" "[x] b" "page"
add "d"
post /delete id=4 >/dev/null
expect_list $'1 [ ] a\n2 [x] b\n3 [ ] c'
expect_code 404 /nope
bb_unit
echo PASS
