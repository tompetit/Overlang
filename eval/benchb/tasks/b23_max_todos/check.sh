#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
bb_baseline
for i in $(seq 2 20); do expect_code 200 /add "title=item$i"; done
expect_eq "$(list | wc -l)" 20 "twenty todos"
expect_code 409 /add title=one-too-many
expect_eq "$(post /add title=one-too-many)" "list full" "body"
expect_eq "$(list | wc -l)" 20 "still twenty"
expect_code 400 /add title=
post /toggle id=5 >/dev/null
expect_code 409 /add title=still-full
post /delete id=5 >/dev/null
expect_code 200 /add title=fits-now
expect_code 409 /add title=full-again
expect_has "$(list)" "fits-now" "added after delete"
expect_lacks "$(list)" "too-many" "rejected not stored"
bb_unit
echo PASS
