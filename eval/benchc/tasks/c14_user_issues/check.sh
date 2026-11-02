#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
# local helpers (lib.sh has none for these)
ok_post() { local c; c="$(code "$1" "$2")"; [ "$c" = 200 ] || fail "POST $1 [$2]: want 200, got $c"; }
expect_get() { expect_eq "$(get "$1")" "$2" "GET $1"; }
bb_baseline
ok_post /users name=carol
ok_post /projects 'key=ZED&name=Zed'
ok_post /p/ZED/labels name=ops
ok_post /projects 'key=API&name=Api'
ok_post /p/ZED/issues 'title=Deploy&priority=2&assignee=alice&labels=ops'
ok_post /p/API/issues 'title=Rate+%3Climit%3E&priority=1&assignee=alice'
ok_post /p/API/issues 'title=Docs&assignee=bob'
ok_post /p/ZED/issues 'title=Backup&assignee=alice'
ok_post /p/ZED/2/status status=closed
ok_post /p/BASE/issues 'title=Footer&assignee=alice'
ok_post /p/BASE/3/assign assignee=bob
ok_post /p/BASE/issues 'title=Header&assignee=alice'
ok_post /p/BASE/4/assign assignee=
A1='API-1 open P1 alice [] Rate <limit>'
W1='BASE-1 closed P1 alice [bug,ui] Login fails'
Z1='ZED-1 open P2 alice [ops] Deploy'
Z2='ZED-2 closed P3 alice [] Backup'
A2='API-2 open P3 bob [] Docs'
W2='BASE-2 open P3 bob [] Typo on home'
W3='BASE-3 open P3 bob [] Footer'
# existing behaviour around users
expect_get /users.txt $'1 alice\n2 bob\n3 carol'
expect_get /p/BASE/issues.txt "$W1"$'\n'"$W2"$'\n'"$W3"$'\n''BASE-4 open P3 - [] Header'
expect_get /p/BASE/4/activity.txt $'created\nassigned -'
# text listing
expect_get /users/alice/issues.txt "$A1"$'\n'"$W1"$'\n'"$Z1"$'\n'"$Z2"
expect_eq "$(ctype /users/alice/issues.txt)" 'text/plain; charset=utf-8' "issues.txt content type"
expect_get /users/bob/issues.txt "$A2"$'\n'"$W2"$'\n'"$W3"
expect_get /users/carol/issues.txt ""
expect_code 200 /users/carol/issues.txt
expect_get '/users/alice/issues.txt?status=open' "$A1"$'\n'"$Z1"
expect_get '/users/alice/issues.txt?status=closed' "$W1"$'\n'"$Z2"
expect_get '/users/alice/issues.txt?status=' "$A1"$'\n'"$W1"$'\n'"$Z1"$'\n'"$Z2"
expect_get '/users/bob/issues.txt?status=closed' ""
expect_get /users/alice/issues.txt/ "$A1"$'\n'"$W1"$'\n'"$Z1"$'\n'"$Z2"
# HTML page
HEAD='<table class="issues"><tr><th>Issue</th><th>Title</th><th>Status</th><th>Priority</th><th>Assignee</th><th>Labels</th></tr>'
page="$(get /users/alice)"
expect_eq "$(ctype /users/alice)" 'text/html; charset=utf-8' "page content type"
expect_has "$page" '<title>Issues of alice</title>' "title"
expect_has "$page" '<nav><a href="/">Tracker</a> <a href="/users">Users</a></nav><h1>Issues assigned to alice</h1><p class="count">2 open, 2 closed</p>'"$HEAD"'<tr><td><a href="/p/API/1">API-1</a></td><td>Rate &lt;limit&gt;</td><td>open</td><td>P1</td><td>alice</td><td>-</td></tr><tr><td><a href="/p/BASE/1">BASE-1</a></td><td>Login fails</td><td>closed</td><td>P1</td><td>alice</td><td>bug, ui</td></tr><tr><td><a href="/p/ZED/1">ZED-1</a></td><td>Deploy</td><td>open</td><td>P2</td><td>alice</td><td>ops</td></tr><tr><td><a href="/p/ZED/2">ZED-2</a></td><td>Backup</td><td>closed</td><td>P3</td><td>alice</td><td>-</td></tr></table></body></html>' "alice page"
expect_lacks "$page" 'Rate <limit>' "escaping"
page="$(get '/users/alice?status=open')"
expect_has "$page" '<p class="count">2 open, 2 closed</p>' "count ignores status"
expect_has "$page" '<a href="/p/ZED/1">ZED-1</a>' "open row"
expect_lacks "$page" '<a href="/p/BASE/1">BASE-1</a>' "closed row filtered"
expect_lacks "$page" '<a href="/p/ZED/2">ZED-2</a>' "closed row filtered"
expect_has "$(get '/users/bob?status=closed')" '<p class="count">3 open, 0 closed</p>'"$HEAD"'</table></body></html>' "bob closed"
expect_lacks "$(get /users/bob)" '<a href="/p/BASE/4">' "unassigned issue"
expect_lacks "$(get /users/alice)" '<a href="/p/BASE/3">' "reassigned issue"
expect_get /users/carol '<!doctype html><html><head><meta charset="utf-8"><title>Issues of carol</title></head><body><nav><a href="/">Tracker</a> <a href="/users">Users</a></nav><h1>Issues assigned to carol</h1><p class="count">0 open, 0 closed</p>'"$HEAD"'</table></body></html>'
# users page links
page="$(get /users)"
expect_has "$page" '<ul class="users"><li><a href="/users/alice">alice</a></li><li><a href="/users/bob">bob</a></li><li><a href="/users/carol">carol</a></li></ul>' "users page links"
expect_lacks "$page" '<li>alice</li>' "old users list"
expect_has "$(post /users name=dave)" '<li><a href="/users/dave">dave</a></li>' "POST /users page"
expect_get /users/dave/issues.txt ""
# errors
expect_err 404 "no such user: nobody" /users/nobody
expect_err 404 "no such user: nobody" /users/nobody/issues.txt
expect_err 404 "no such user: Alice" /users/Alice
expect_err 404 "no such user: nobody" '/users/nobody?status=done'
expect_err 400 "bad status" '/users/alice?status=done'
expect_err 400 "bad status" '/users/alice/issues.txt?status=Open'
expect_err 404 "not found" /users/alice 'x=1'
expect_err 404 "not found" /users/alice/issues.txt 'x=1'
expect_err 404 "not found" /users/alice/comments.txt
expect_err 404 "not found" /users/alice/issues.txt/x
expect_err 422 "user exists" /users name=alice
# changes show up, and survive a restart
ok_post /p/API/2/assign assignee=alice
ok_post /p/BASE/1/status status=open
bb_restart
expect_get /users/alice/issues.txt "$A1"$'\n''API-2 open P3 alice [] Docs'$'\n''BASE-1 open P1 alice [bug,ui] Login fails'$'\n'"$Z1"$'\n'"$Z2"
expect_get /users/bob/issues.txt "$W2"$'\n'"$W3"
expect_has "$(get /users/alice)" '<p class="count">4 open, 1 closed</p>' "count after restart"
bb_unit
echo PASS
