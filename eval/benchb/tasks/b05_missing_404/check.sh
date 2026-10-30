#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
bb_baseline
add "one"; add "two"
expect_code 404 /toggle id=99
expect_eq "$(post /toggle id=99)" "no such todo" "404 body"
expect_code 404 /delete id=99
expect_eq "$(post /delete id=99)" "no such todo" "404 body"
expect_code 400 /toggle id=0
expect_code 400 /delete id=-3
expect_code 400 /toggle id=x
expect_list $'2 [ ] write & ship!\n3 [ ] one\n4 [ ] two'
expect_code 200 /toggle id=3
expect_code 200 /delete id=4
expect_code 404 /toggle id=4
expect_list $'2 [ ] write & ship!\n3 [x] one'
bb_unit
echo PASS
