#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
bb_baseline
add "a"; add "b"
post /toggle id=3 >/dev/null
expect_list $'2 [ ] write & ship!\n3 [x] a\n4 [ ] b'
post /toggle id=3 >/dev/null
expect_list $'2 [ ] write & ship!\n3 [ ] a\n4 [ ] b'
post /toggle id=3 >/dev/null
post /toggle id=4 >/dev/null
post /toggle id=2 >/dev/null
post /toggle id=2 >/dev/null
expect_list $'2 [ ] write & ship!\n3 [x] a\n4 [x] b'
bb_restart
expect_list $'2 [ ] write & ship!\n3 [x] a\n4 [x] b'
bb_unit
echo PASS
