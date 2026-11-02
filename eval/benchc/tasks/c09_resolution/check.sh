#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
# Old behavior that does not show a closed status (bb_baseline does, so it is not used here).
expect_eq "$(get /health)" "ok" "health"
post /users name=alice >/dev/null; post /users name=bob >/dev/null
expect_eq "$(get /users.txt)" $'1 alice\n2 bob' "users.txt"
expect_err 422 "user exists" /users name=alice
post /projects 'key=WEB&name=Web+site' >/dev/null
expect_eq "$(get /projects.txt)" "WEB Web site" "projects.txt"
expect_err 422 "bad key" /projects 'key=web&name=x'
post /p/WEB/labels name=bug >/dev/null; post /p/WEB/labels name=ui >/dev/null
post /p/WEB/issues 'title=Login+fails&body=Steps%3A+**click**&priority=1&assignee=alice&labels=bug,ui' >/dev/null
post /p/WEB/issues 'title=Typo' >/dev/null
post /p/WEB/issues 'title=Dup+of+1' >/dev/null
expect_eq "$(get /p/WEB/issues.txt)" $'WEB-1 open P1 alice [bug,ui] Login fails\nWEB-2 open P3 - [] Typo\nWEB-3 open P3 - [] Dup of 1' "issues.txt"
expect_has "$(get /p/WEB/1)" '<strong>click</strong>' "markdown"
post /p/WEB/1/comments 'author=bob&body=on+it' >/dev/null
expect_eq "$(get /p/WEB/1/comments.txt)" 'bob: on it' "comments"
expect_err 422 "title required" /p/WEB/issues title=
expect_err 422 "no such label: nope" /p/WEB/issues 'title=x&labels=nope'
expect_err 404 "no such issue: WEB-99" /p/WEB/99
expect_err 400 "bad status" '/p/WEB?status=done'
expect_err 404 "not found" /nope

# Closing.
r="$(post /p/WEB/1/status status=closed)"
expect_has "$r" '<span class="status">closed:fixed</span>' "issue page token"
expect_has "$r" '<input type="hidden" name="status" value="open"><button>reopen</button>' "reopen form"
expect_eq "$(get /p/WEB/1.txt)" 'WEB-1 closed:fixed P1 alice [bug,ui] Login fails' "issue line"
expect_eq "$(bb_sql 'select status, resolution from issues order by number')" $'closed|fixed\nopen|\nopen|' "stored columns"
post /p/WEB/3/status 'status=closed&resolution=duplicate' >/dev/null
post /p/WEB/2/status 'status=closed&resolution=wontfix' >/dev/null
expect_err 422 "bad resolution" /p/WEB/2/status 'status=open&resolution=fixed'
expect_err 422 "bad resolution" /p/WEB/2/status 'status=closed&resolution=Fixed'
expect_err 422 "bad resolution" /p/WEB/2/status 'status=closed&resolution=invalid'
expect_err 422 "bad status" /p/WEB/2/status 'status=done&resolution=bogus'
expect_err 422 "bad status" /p/WEB/2/status 'status=&resolution=fixed'
expect_err 404 "no such issue: WEB-9" /p/WEB/9/status 'status=closed&resolution=bogus'
expect_eq "$(get /p/WEB/2/activity.txt)" $'created\nstatus closed:wontfix' "refused requests change nothing"

# Changes and no-ops.
post /p/WEB/2/status 'status=closed&resolution=duplicate' >/dev/null
post /p/WEB/2/status 'status=closed&resolution=duplicate' >/dev/null
post /p/WEB/2/status 'status=closed' >/dev/null
post /p/WEB/2/status 'status=closed&resolution=' >/dev/null
post /p/WEB/2/status 'status=open' >/dev/null
expect_eq "$(bb_sql 'select status, resolution from issues where number = 2')" 'open|' "reopen clears"
post /p/WEB/2/status 'status=open&resolution=' >/dev/null
post /p/WEB/2/status 'status=closed&resolution=wontfix' >/dev/null
expect_eq "$(get /p/WEB/2/activity.txt)" $'created\nstatus closed:wontfix\nstatus closed:duplicate\nstatus closed:fixed\nstatus open\nstatus closed:wontfix' "activity"
expect_eq "$(get /p/WEB/2.json)" '{"key":"WEB-2","project":"WEB","number":2,"title":"Typo","body":"","status":"closed","resolution":"wontfix","priority":3,"assignee":null,"labels":[],"comments":[],"activity":[{"kind":"created","detail":""},{"kind":"status","detail":"closed:wontfix"},{"kind":"status","detail":"closed:duplicate"},{"kind":"status","detail":"closed:fixed"},{"kind":"status","detail":"open"},{"kind":"status","detail":"closed:wontfix"}]}' "json closed"
post /p/WEB/issues 'title=Open+one&body=a%2Cb' >/dev/null
expect_eq "$(get /p/WEB/4.json)" '{"key":"WEB-4","project":"WEB","number":4,"title":"Open one","body":"a,b","status":"open","resolution":null,"priority":3,"assignee":null,"labels":[],"comments":[],"activity":[{"kind":"created","detail":""}]}' "json open"
getf /p/WEB/issues.csv "$RUN/web.csv"
expect_file "$RUN/web.csv" 'key,title,status,resolution,priority,assignee,labels,body\nWEB-1,Login fails,closed,fixed,1,alice,bug;ui,Steps: **click**\nWEB-2,Typo,closed,wontfix,3,,,\nWEB-3,Dup of 1,closed,duplicate,3,,,\nWEB-4,Open one,open,,3,,,"a,b"\n'
page="$(get /p/WEB)"
expect_has "$page" '<td>closed:wontfix</td>' "project row token"
expect_has "$page" '<a href="/p/WEB/4">WEB-4</a></td><td>Open one</td><td>open</td>' "open row"

# Filters.
expect_eq "$(get '/p/WEB/issues.txt?status=closed')" $'WEB-1 closed:fixed P1 alice [bug,ui] Login fails\nWEB-2 closed:wontfix P3 - [] Typo\nWEB-3 closed:duplicate P3 - [] Dup of 1' "status=closed"
expect_eq "$(get '/p/WEB/issues.txt?resolution=wontfix')" 'WEB-2 closed:wontfix P3 - [] Typo' "resolution filter"
expect_eq "$(get '/p/WEB/issues.txt?status=closed&resolution=duplicate')" 'WEB-3 closed:duplicate P3 - [] Dup of 1' "both"
expect_eq "$(get '/p/WEB/issues.txt?status=open&resolution=fixed')" "" "open has none"
expect_eq "$(get '/p/WEB/issues.txt?status=open')" 'WEB-4 open P3 - [] Open one' "status=open"
expect_eq "$(get '/p/WEB/issues.txt?resolution=&label=bug')" 'WEB-1 closed:fixed P1 alice [bug,ui] Login fails' "empty resolution is no filter"
expect_err 400 "bad resolution" '/p/WEB/issues.txt?resolution=nope'
expect_err 400 "bad resolution" '/p/WEB?resolution=Fixed'
expect_err 400 "bad status" '/p/WEB?status=x&resolution=nope'
expect_err 400 "bad resolution" '/p/WEB?resolution=nope&priority=9'
expect_err 400 "bad resolution" '/p/WEB/issues.txt?page=0&resolution=nope'
expect_err 400 "bad priority" '/p/WEB?priority=9&resolution=fixed'

bb_restart
expect_eq "$(get '/p/WEB/issues.txt?status=closed&resolution=fixed')" 'WEB-1 closed:fixed P1 alice [bug,ui] Login fails' "persisted"
expect_has "$(get /p/WEB/3)" '<span class="status">closed:duplicate</span>' "page after restart"

# Pager links carry the resolution right after the status.
for i in $(seq 21); do post /p/WEB/issues "title=n+$i" >/dev/null; post "/p/WEB/$((4 + i))/status" 'status=closed&resolution=wontfix' >/dev/null; done
expect_has "$(get '/p/WEB?resolution=wontfix')" 'Page 1 of 2 (22 issues) <a href="/p/WEB?resolution=wontfix&amp;page=2">next</a>' "pager link"
expect_has "$(get '/p/WEB?page=2&resolution=wontfix&status=closed')" '<a href="/p/WEB?status=closed&amp;resolution=wontfix&amp;page=1">prev</a>' "pager order"
expect_eq "$(get '/p/WEB/issues.txt?resolution=wontfix&page=2')" $'WEB-24 closed:wontfix P3 - [] n 20\nWEB-25 closed:wontfix P3 - [] n 21' "page 2"
expect_eq "$(get /p/WEB/25/activity.txt)" $'created\nstatus closed:wontfix' "activity of a later issue"
bb_unit
echo PASS
