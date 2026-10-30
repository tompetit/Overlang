#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
bb_baseline
t80="$(python3 -c 'print("a"*80)')"; t81="$(python3 -c 'print("a"*81)')"
e80="$(python3 -c 'print("\u00e9"*80)')"; e81="$(python3 -c 'print("\u00e9"*81)')"
expect_code 200 /add "title=$t80"
expect_code 400 /add "title=$t81"
expect_eq "$(post /add "title=$t81")" "title too long" "body"
expect_code 200 /add "title=$e80"
expect_code 400 /add "title=$e81"
expect_code 200 /add "title=++$t80++"
expect_code 400 /add "title=++x$t80++"
expect_eq "$(list | wc -l)" 4 "stored: baseline 1 + three accepted adds"
expect_has "$(list)" "$t80"
bb_unit
echo PASS
