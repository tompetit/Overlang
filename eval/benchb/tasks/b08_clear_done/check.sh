#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
bb_baseline
add "a"; add "b"; add "c"
post /toggle id=2 >/dev/null; post /toggle id=4 >/dev/null
expect_code 200 /clear-done x=1
expect_list $'3 [ ] a\n5 [ ] c'
expect_has "$(post /clear-done x=1)" "<h1>Todos</h1>" "page body"
expect_list $'3 [ ] a\n5 [ ] c'
add "d"
expect_list $'3 [ ] a\n5 [ ] c\n6 [ ] d'
bb_restart
expect_list $'3 [ ] a\n5 [ ] c\n6 [ ] d'
expect_code 404 /clear-done
bb_unit
echo PASS
