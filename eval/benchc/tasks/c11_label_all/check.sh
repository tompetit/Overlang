#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
# local helpers (lib.sh has none for these)
ok_post() { local c; c="$(code "$1" "$2")"; [ "$c" = 200 ] || fail "POST $1 [$2]: want 200, got $c"; }
expect_get() { expect_eq "$(get "$1")" "$2" "GET $1"; }
bb_baseline
ok_post /p/BASE/labels name=docs
ok_post /p/BASE/issues 'title=Button+color&priority=2&labels=ui'
ok_post /p/BASE/issues 'title=Crash+on+save&priority=2&labels=bug'
ok_post /p/BASE/issues 'title=Broken+docs+link&priority=4&assignee=alice&labels=ui,docs,bug'
ok_post /p/BASE/issues 'title=Docs+typo&labels=docs'
I1='BASE-1 closed P1 alice [bug,ui] Login fails'
I2='BASE-2 open P3 bob [] Typo on home'
I3='BASE-3 open P2 - [ui] Button color'
I4='BASE-4 open P2 - [bug] Crash on save'
I5='BASE-5 open P4 alice [bug,docs,ui] Broken docs link'
I6='BASE-6 open P3 - [docs] Docs typo'
ALL="$I1"$'\n'"$I2"$'\n'"$I3"$'\n'"$I4"$'\n'"$I5"$'\n'"$I6"
expect_get /p/BASE/issues.txt "$ALL"
# one label: unchanged
expect_get '/p/BASE/issues.txt?label=bug' "$I1"$'\n'"$I4"$'\n'"$I5"
expect_get '/p/BASE/issues.txt?label=docs' "$I5"$'\n'"$I6"
# several labels: AND
expect_get '/p/BASE/issues.txt?label=bug,ui' "$I1"$'\n'"$I5"
expect_get '/p/BASE/issues.txt?label=ui,bug' "$I1"$'\n'"$I5"
expect_get '/p/BASE/issues.txt?label=bug%2Cui' "$I1"$'\n'"$I5"
expect_get '/p/BASE/issues.txt?label=+bug+,+,ui+' "$I1"$'\n'"$I5"
expect_get '/p/BASE/issues.txt?label=bug,docs,ui' "$I5"
expect_get '/p/BASE/issues.txt?label=docs,ui' "$I5"
expect_get '/p/BASE/issues.txt?label=bug,bug' "$I1"$'\n'"$I4"$'\n'"$I5"
expect_get '/p/BASE/issues.txt?label=bug,bug,ui,ui,bug' "$I1"$'\n'"$I5"
expect_get '/p/BASE/issues.txt?label=bug,ui&status=open' "$I5"
expect_get '/p/BASE/issues.txt?label=bug,ui&assignee=alice&priority=1' "$I1"
expect_get '/p/BASE/issues.txt?label=bug,nope' ""
expect_code 200 '/p/BASE/issues.txt?label=bug,nope'
expect_get '/p/BASE/issues.txt?label=,' "$ALL"
expect_get '/p/BASE/issues.txt?label=+' "$ALL"
expect_has "$(get '/p/BASE?label=bug,ui')" 'Page 1 of 1 (2 issues)' "pager total"
# errors
expect_err 400 "bad label" '/p/BASE/issues.txt?label=Bug'
expect_err 400 "bad label" '/p/BASE?label=bug,a+b'
expect_err 400 "bad label" '/p/BASE/issues.txt?label=bug,aaaaaaaaaaaaaaaaaaaaa'
expect_err 400 "too many labels" '/p/BASE/issues.txt?label=a,b,c,d'
expect_err 400 "too many labels" '/p/BASE?label=bug,ui,docs,nope'
expect_err 400 "bad label" '/p/BASE?label=a,b,c,Bad'
expect_err 400 "bad status" '/p/BASE?status=done&label=Bug'
expect_err 400 "bad priority" '/p/BASE/issues.txt?label=a,b,c,d&priority=9'
expect_err 400 "bad page" '/p/BASE?label=Bug&page=0'
expect_err 404 "no such project: NOPE" '/p/NOPE?label=Bug'
# issue label routes still take one name
expect_err 422 "no such label: bug,ui" /p/BASE/3/labels 'label=bug,ui'
expect_err 422 "no such label: bug,ui" /p/BASE/3/labels/remove 'label=bug,ui'
# pagination and write-back
for n in $(seq 7 26); do ok_post /p/BASE/issues "title=m+$n&labels=bug,ui"; done
page="$(get '/p/BASE?label=ui,+bug,ui')"
expect_has "$page" '<input name="label" value="ui,bug" placeholder="label">' "label input"
expect_has "$page" 'Page 1 of 2 (22 issues)' "pager"
expect_has "$page" ' <a href="/p/BASE?label=ui%2Cbug&amp;page=2">next</a>' "next link"
expect_has "$page" '<a href="/p/BASE/24">BASE-24</a>' "page 1 rows"
expect_lacks "$page" '<a href="/p/BASE/25">BASE-25</a>' "page 1 stops at 20"
expect_lacks "$page" '<a href="/p/BASE/4">BASE-4</a>' "AND filter on page"
page="$(get '/p/BASE?status=open&label=bug,ui&page=2')"
expect_has "$page" 'Page 2 of 2 (21 issues)' "pager with status"
expect_has "$page" ' <a href="/p/BASE?status=open&amp;label=bug%2Cui&amp;page=1">prev</a>' "prev link"
expect_get '/p/BASE/issues.txt?label=ui,bug&page=2' $'BASE-25 open P3 - [bug,ui] m 25\nBASE-26 open P3 - [bug,ui] m 26'
expect_has "$(get /p/BASE/issues.csv)" 'BASE-6,Docs typo,open,3,,docs,' "csv"
expect_eq "$(get /p/BASE/issues.csv | wc -l | tr -d ' ')" 27 "csv rows"
bb_restart
expect_get '/p/BASE/issues.txt?label=docs,bug' "$I5"
expect_get '/p/BASE/issues.txt?label=docs' "$I5"$'\n'"$I6"
bb_unit
echo PASS
