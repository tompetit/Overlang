#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
bb_baseline
add "wrtie report"; add "other"
post /toggle id=3 >/dev/null
expect_code 200 /rename "id=3&title=++write+report++"
expect_list $'2 [ ] write & ship!\n3 [x] write report\n4 [ ] other'
expect_has "$(post /rename "id=4&title=a+%26+b")" "a &amp; b" "page shows new escaped title"
expect_list $'2 [ ] write & ship!\n3 [x] write report\n4 [ ] a & b'
expect_code 400 /rename "id=0&title=x"
expect_eq "$(post /rename "id=0&title=x")" "bad id" "bad id body"
expect_code 400 /rename "title=x"
expect_code 400 /rename "id=3&title=+++"
expect_eq "$(post /rename "id=3&title=")" "title required" "blank body"
expect_code 200 /rename "id=99&title=x"
expect_list $'2 [ ] write & ship!\n3 [x] write report\n4 [ ] a & b'
bb_restart
expect_list $'2 [ ] write & ship!\n3 [x] write report\n4 [ ] a & b'
bb_unit
echo PASS
