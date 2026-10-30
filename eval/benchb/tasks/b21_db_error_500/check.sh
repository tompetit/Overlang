#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
bb_baseline
expect_code 400 /add title=
expect_code 400 /toggle id=0
bb_sql 'drop table todos' >/dev/null
expect_code 500 /add title=x
expect_eq "$(post /add title=x)" "db error" "add body"
expect_code 500 /toggle id=2
expect_eq "$(post /toggle id=2)" "db error" "toggle body"
expect_code 500 /delete id=2
expect_eq "$(post /delete id=2)" "db error" "delete body"
expect_code 400 /add title=
expect_code 400 /toggle id=0
expect_code 200 /todos.txt
expect_code 200 /
expect_code 404 /nope
bb_unit
echo PASS
