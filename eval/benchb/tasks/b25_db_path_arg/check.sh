#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
bb_baseline
bb_stop
rm -f "$RUN/todo.db"
bb_start custom.db
add "in custom"
expect_list '1 [ ] in custom'
[ -f "$RUN/custom.db" ] || fail "custom.db not created"
[ ! -e "$RUN/todo.db" ] || fail "todo.db should not be created when a path is given"
expect_eq "$(bb_sql 'select title from todos' custom.db)" "in custom" "row in custom.db"
bb_restart custom.db
expect_list '1 [ ] in custom'
bb_stop
bb_start
expect_list ''
[ -f "$RUN/todo.db" ] || fail "default todo.db not created"
bb_unit
bb_probe
echo PASS
