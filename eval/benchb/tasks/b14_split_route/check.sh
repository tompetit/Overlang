#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
bb_baseline
add "x"
expect_code 404 /todos.txt x=1
expect_code 404 /add
expect_code 200 /
bb_unit
bb_probe
echo PASS
