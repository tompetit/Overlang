#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
bb_baseline
add "dated"
page="$(get /)"
today="$(date -u +%F)"
yesterday="$(date -u -d yesterday +%F)"
case "$page" in
  *"<small>added $today</small>"*|*"<small>added $yesterday</small>"*) ;;
  *) fail "page lacks <small>added DATE</small>: $page";;
esac
expect_eq "$(grep -o '<small>added' <<<"$page" | wc -l)" 2 "one per todo"
ts="$(bb_sql 'select created_at from todos where id = 3')"
[[ "$ts" =~ ^20[0-9][0-9]-[01][0-9]-[0-3][0-9]\ [0-2][0-9]:[0-5][0-9]:[0-5][0-9]$ ]] || fail "created_at format: [$ts]"
post /toggle id=3 >/dev/null
bb_restart
expect_eq "$(bb_sql 'select created_at from todos where id = 3')" "$ts" "created_at stable"
expect_list $'2 [ ] write & ship!\n3 [x] dated'
bb_unit
echo PASS
