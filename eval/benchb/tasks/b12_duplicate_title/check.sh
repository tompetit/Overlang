#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
bb_baseline
add "Buy milk"
expect_code 409 /add "title=buy+MILK+"
expect_eq "$(post /add "title=++Buy+milk")" "duplicate title" "body"
expect_code 409 /add "title=Buy+milk"
expect_code 200 /add "title=Buy+milk+2"
expect_code 409 /add "title=write+%26+SHIP%21"
expect_code 400 /add "title=+"
expect_list $'2 [ ] write & ship!\n3 [ ] Buy milk\n4 [ ] Buy milk 2'
post /delete id=3 >/dev/null
expect_code 200 /add "title=buy+milk"
expect_list $'2 [ ] write & ship!\n4 [ ] Buy milk 2\n5 [ ] buy milk'
bb_unit
echo PASS
