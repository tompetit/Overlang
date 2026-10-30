#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
bb_baseline
add "   buy   milk  "
add "	tabbed	"
expect_list $'2 [ ] write & ship!\n3 [ ] buy   milk\n4 [ ] tabbed'
expect_eq "$(bb_sql "select title from todos where id = 3")" "buy   milk" "stored trimmed"
expect_code 400 /add "title=%20%20"
bb_restart
expect_list $'2 [ ] write & ship!\n3 [ ] buy   milk\n4 [ ] tabbed'
bb_unit
echo PASS
