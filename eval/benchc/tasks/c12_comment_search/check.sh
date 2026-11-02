#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
# local helpers (lib.sh has none for these)
ok_post() { local c; c="$(code "$1" "$2")"; [ "$c" = 200 ] || fail "POST $1 [$2]: want 200, got $c"; }
expect_get() { expect_eq "$(get "$1")" "$2" "GET $1"; }
bb_baseline
ok_post /p/BASE/issues 'title=Slow+search&body=Takes+5s'
ok_post /p/BASE/issues 'title=Mobile+layout'
ok_post /p/BASE/2/comments 'author=alice&body=Seen+on+mobile+too'
ok_post /p/BASE/3/comments 'author=bob&body=Mobile+only%3F+check+MOBILE+app'
ok_post /p/BASE/3/comments 'author=alice&body=mobile+again'
ok_post /p/BASE/4/comments 'author=bob&body=first+line%0Amobile+here'
ok_post /p/BASE/3/comments 'author=alice&body=use+snake_case'
ok_post /projects 'key=API&name=Api'
ok_post /p/API/issues 'title=Api+thing'
ok_post /p/API/1/comments 'author=alice&body=mobile+api'
ok_post /p/BASE/1/comments 'author=bob&body=100%25+sure'
I1='BASE-1 closed P1 alice [bug,ui] Login fails'
I2='BASE-2 open P3 bob [] Typo on home'
I3='BASE-3 open P3 - [] Slow search'
I4='BASE-4 open P3 - [] Mobile layout'
# old matches still work
expect_get '/p/BASE/issues.txt?q=50%25' "$I2"
expect_get '/p/BASE/issues.txt?q=LOGIN' "$I1"
expect_get '/p/BASE/issues.txt?q=5s' "$I3"
expect_get /p/BASE/3/comments.txt $'bob: Mobile only? check MOBILE app\nalice: mobile again\nalice: use snake_case'
expect_get /p/BASE/4/comments.txt 'bob: first line mobile here'
# comment bodies are searched, each issue once
expect_get '/p/BASE/issues.txt?q=mobile' "$I2"$'\n'"$I3"$'\n'"$I4"
expect_get '/p/BASE/issues.txt?q=MOBILE' "$I2"$'\n'"$I3"$'\n'"$I4"
expect_get '/p/BASE/issues.txt?q=v2' "$I1"
expect_get '/p/BASE/issues.txt?q=fixed+IN' "$I1"
expect_get '/p/BASE/issues.txt?q=first+line' "$I4"
expect_get '/p/BASE/issues.txt?q=%25' "$I1"$'\n'"$I2"
expect_get '/p/BASE/issues.txt?q=_' "$I3"
expect_get '/p/BASE/issues.txt?q=e_c' "$I3"
expect_get '/p/BASE/issues.txt?q=bob' ""
expect_get '/p/BASE/issues.txt?q=alice' ""
expect_get '/p/BASE/issues.txt?q=api' ""
expect_get '/p/BASE/issues.txt?q=mobile&assignee=bob' "$I2"
expect_get '/p/BASE/issues.txt?q=v2&status=open' ""
expect_get '/p/BASE/issues.txt?q=v2&status=closed&label=bug' "$I1"
expect_get '/p/API/issues.txt?q=mobile' 'API-1 open P3 - [] Api thing'
page="$(get '/p/BASE?q=mobile')"
expect_has "$page" 'Page 1 of 1 (3 issues)' "pager counts issues once"
expect_has "$page" '<input name="q" value="mobile" placeholder="search">' "q input"
expect_has "$(get '/p/BASE?q=again')" 'Page 1 of 1 (1 issues)' "pager total"
# project comment search
expect_get '/p/BASE/comments.txt?q=MOBILE' $'BASE-2 alice: Seen on mobile too\nBASE-3 bob: Mobile only? check MOBILE app\nBASE-3 alice: mobile again\nBASE-4 bob: first line mobile here'
expect_eq "$(ctype '/p/BASE/comments.txt?q=mobile')" 'text/plain; charset=utf-8' "comments.txt content type"
expect_get '/p/BASE/comments.txt?q=_' 'BASE-3 alice: use snake_case'
expect_get '/p/BASE/comments.txt?q=%25' 'BASE-1 bob: 100% sure'
expect_get '/p/BASE/comments.txt?q=IN' $'BASE-1 bob: Fixed in <b>v2</b>\nBASE-3 alice: mobile again\nBASE-4 bob: first line mobile here'
expect_get '/p/BASE/comments.txt?q=u' $'BASE-3 alice: use snake_case\nBASE-1 bob: 100% sure'
expect_get '/p/BASE/comments.txt?q=Login' ""
expect_get '/p/BASE/comments.txt?q=bob' ""
expect_code 200 '/p/BASE/comments.txt?q=zzz'
expect_get '/p/API/comments.txt?q=mobile' 'API-1 alice: mobile api'
expect_get '/p/API/comments.txt?q=v2' ""
expect_err 400 "q required" /p/BASE/comments.txt
expect_err 400 "q required" '/p/BASE/comments.txt?q=+++'
expect_err 400 "q required" '/p/BASE/comments.txt?status=open'
expect_err 404 "no such project: NOPE" '/p/NOPE/comments.txt?q=x'
expect_err 404 "no such project: NOPE" /p/NOPE/comments.txt
expect_err 404 "not found" /p/BASE/comments.txt 'q=x'
bb_restart
expect_get '/p/BASE/issues.txt?q=mobile' "$I2"$'\n'"$I3"$'\n'"$I4"
expect_get '/p/BASE/comments.txt?q=snake' 'BASE-3 alice: use snake_case'
bb_unit
echo PASS
