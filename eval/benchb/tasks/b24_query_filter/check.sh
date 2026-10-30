#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
bb_baseline
add "a"; add "b"; add "c"
post /toggle id=4 >/dev/null
expect_eq "$(get '/todos.txt?done=1')" "4 [x] b" "done only"
expect_eq "$(get '/todos.txt?done=0')" $'2 [ ] write & ship!\n3 [ ] a\n5 [ ] c' "open only"
expect_eq "$(get '/todos.txt?done=2')" $'2 [ ] write & ship!\n3 [ ] a\n4 [x] b\n5 [ ] c' "other value"
expect_eq "$(get '/todos.txt?x=1&done=0')" $'2 [ ] write & ship!\n3 [ ] a\n5 [ ] c' "extra params"
expect_eq "$(get '/todos.txt?')" $'2 [ ] write & ship!\n3 [ ] a\n4 [x] b\n5 [ ] c' "empty query"
expect_code 200 '/?done=1'
expect_has "$(get '/?done=1')" "<h1>Todos</h1>" "page ignores query"
expect_code 404 '/nope?done=1'
expect_code 200 '/add?x=1' title=viaquery
expect_has "$(list)" "viaquery" "post with query"
bb_unit
echo PASS
