#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
add "buy milk"; add "write & ship!"
expect_list $'1\topen\tbuy milk\n2\topen\twrite & ship!'
post /toggle id=1 >/dev/null
expect_list $'1\tdone\tbuy milk\n2\topen\twrite & ship!'
expect_has "$(get /)" "[x] buy milk" "page keeps marks"
expect_has "$(get /)" "[ ] write &amp; ship!" "page keeps marks"
post /delete id=1 >/dev/null
expect_list $'2\topen\twrite & ship!'
expect_code 404 /nope
expect_code 400 /add title=
case "$(ctype /todos.txt)" in text/plain*) ;; *) fail "content type";; esac
bb_restart
expect_list $'2\topen\twrite & ship!'
bb_unit
echo PASS
