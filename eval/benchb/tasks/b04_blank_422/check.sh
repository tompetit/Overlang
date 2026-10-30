#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
BB_BLANK_CODE=422
bb_baseline
expect_code 422 /add title=
expect_code 422 /add title=+++
expect_eq "$(post /add title=%20)" "title must not be blank" "422 body"
expect_code 400 /toggle id=abc
expect_eq "$(post /toggle id=0)" "bad id" "bad id body"
add "kept"
expect_has "$(list)" "[ ] kept"
bb_unit
echo PASS
