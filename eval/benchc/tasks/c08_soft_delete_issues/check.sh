#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
bb_baseline
post /projects 'key=WEB&name=Web' >/dev/null   # project id 2 (BASE is 1)
post /p/WEB/labels name=bug >/dev/null
post /p/WEB/issues 'title=One&labels=bug&assignee=alice&priority=1' >/dev/null
post /p/WEB/issues 'title=Two' >/dev/null
post /p/WEB/issues 'title=Three&labels=bug' >/dev/null
post /p/WEB/2/comments 'author=bob&body=hi' >/dev/null
expect_eq "$(get /p/WEB/trash.txt)" "" "empty trash"
expect_eq "$(ctype /p/WEB/trash.txt)" 'text/plain; charset=utf-8' "trash content type"

# Delete WEB-2.
r="$(post /p/WEB/2/delete '')"
expect_has "$r" '<h1>WEB: Web</h1>' "delete answers the project page"
expect_has "$r" 'Page 1 of 1 (2 issues)' "count leaves out deleted"
expect_lacks "$r" '>WEB-2</a>' "row gone"
expect_eq "$(get /p/WEB/issues.txt)" $'WEB-1 open P1 alice [bug] One\nWEB-3 open P3 - [bug] Three' "issues.txt"
getf /p/WEB/issues.csv "$RUN/web.csv"
expect_file "$RUN/web.csv" 'key,title,status,priority,assignee,labels,body\nWEB-1,One,open,1,alice,bug,\nWEB-3,Three,open,3,,bug,\n'
for p in "" .json .txt /comments.txt /activity.txt; do expect_err 404 "no such issue: WEB-2" "/p/WEB/2$p"; done
expect_err 404 "no such issue: WEB-2" /p/WEB/2/edit title=x
expect_err 404 "no such issue: WEB-2" /p/WEB/2/status status=closed
expect_err 404 "no such issue: WEB-2" /p/WEB/2/assign assignee=bob
expect_err 404 "no such issue: WEB-2" /p/WEB/2/comments 'author=bob&body=x'
expect_err 404 "no such issue: WEB-2" /p/WEB/2/labels label=bug
expect_err 404 "no such issue: WEB-2" /p/WEB/2/labels/remove label=bug
expect_err 404 "no such issue: WEB-2" /p/WEB/2/delete ''
expect_err 404 "no such issue: WEB-9" /p/WEB/9/delete ''
expect_err 404 "no such project: NOPE" /p/NOPE/1/delete ''
expect_err 404 "no such project: NOPE" /p/NOPE/trash.txt
expect_err 404 "not found" /p/WEB/1/delete
expect_eq "$(get /p/WEB/trash.txt)" 'WEB-2 open P3 - [] Two' "trash"
expect_eq "$(bb_sql 'select number, deleted from issues where project_id = 2 order by number')" $'1|0\n2|1\n3|0' "row kept, flag set"
expect_eq "$(bb_sql 'select count(*) from comments c join issues i on i.id = c.issue_id where i.project_id = 2 and i.number = 2')" 1 "comment kept"

# Filters and counts.
post /p/WEB/3/delete '' >/dev/null
expect_eq "$(get '/p/WEB/issues.txt?label=bug')" 'WEB-1 open P1 alice [bug] One' "label filter"
expect_has "$(get '/p/WEB?label=bug')" 'Page 1 of 1 (1 issues)' "filtered count"
expect_eq "$(get '/p/WEB/issues.txt?q=T')" "" "search leaves out deleted"

# Numbers are not reused.
expect_has "$(post /p/WEB/issues title=Four)" '<h1>WEB-4: Four</h1>' "next number"
post /p/WEB/4/delete '' >/dev/null
expect_has "$(post /p/WEB/issues title=Five)" '<h1>WEB-5: Five</h1>' "deleted max still counts"

# Restore.
expect_err 404 "no such issue: WEB-9" /p/WEB/9/restore ''
expect_err 404 "no such project: NOPE" /p/NOPE/2/restore ''
expect_err 404 "not found" /p/WEB/2/restore
expect_has "$(post /p/WEB/1/restore '')" '<h1>WEB-1: One</h1>' "restore of a live issue"
expect_eq "$(get /p/WEB/1/activity.txt)" 'created' "nothing logged for a live issue"
expect_has "$(post /p/WEB/2/restore '')" '<h1>WEB-2: Two</h1>' "restore answers the issue page"
expect_eq "$(get /p/WEB/2/comments.txt)" 'bob: hi' "comments back"
expect_eq "$(get /p/WEB/2/activity.txt)" $'created\ncommented bob\ndeleted\nrestored' "activity"
expect_eq "$(get /p/WEB/2.json)" '{"key":"WEB-2","project":"WEB","number":2,"title":"Two","body":"","status":"open","priority":3,"assignee":null,"labels":[],"comments":[{"author":"bob","body":"hi"}],"activity":[{"kind":"created","detail":""},{"kind":"commented","detail":"bob"},{"kind":"deleted","detail":""},{"kind":"restored","detail":""}]}' "json"
post /p/WEB/2/restore '' >/dev/null
expect_eq "$(get /p/WEB/2/activity.txt)" $'created\ncommented bob\ndeleted\nrestored' "second restore logs nothing"
expect_eq "$(get /p/WEB/issues.txt)" $'WEB-1 open P1 alice [bug] One\nWEB-2 open P3 - [] Two\nWEB-5 open P3 - [] Five' "after restore"
expect_eq "$(get /p/WEB/trash.txt)" $'WEB-3 open P3 - [bug] Three\nWEB-4 open P3 - [] Four' "trash after restore"
expect_eq "$(get /p/BASE/trash.txt)" "" "other project's trash"
post /p/WEB/5/delete '' >/dev/null

bb_restart
expect_eq "$(get /p/WEB/issues.txt)" $'WEB-1 open P1 alice [bug] One\nWEB-2 open P3 - [] Two' "persisted"
expect_eq "$(get /p/WEB/trash.txt)" $'WEB-3 open P3 - [bug] Three\nWEB-4 open P3 - [] Four\nWEB-5 open P3 - [] Five' "trash persisted"
expect_err 404 "no such issue: WEB-3" /p/WEB/3.json
expect_has "$(post /p/WEB/issues title=Six)" '<h1>WEB-6: Six</h1>' "no reuse after restart"
post /p/WEB/3/restore '' >/dev/null
expect_eq "$(get /p/WEB/3.txt)" 'WEB-3 open P3 - [bug] Three' "labels back"
expect_eq "$(get /p/WEB/3/activity.txt)" $'created\ndeleted\nrestored' "activity of WEB-3"

# Pagination counts only live issues: WEB-1,2,3,6 plus n 1..20 (WEB-7..26), minus WEB-7.
for i in $(seq 20); do post /p/WEB/issues "title=n+$i" >/dev/null; done
post /p/WEB/7/delete '' >/dev/null
expect_has "$(get /p/WEB)" 'Page 1 of 2 (23 issues)' "pager total"
expect_eq "$(get /p/WEB/issues.txt | wc -l | tr -d ' ')" 20 "page 1 size"
expect_eq "$(get '/p/WEB/issues.txt?page=2')" $'WEB-24 open P3 - [] n 18\nWEB-25 open P3 - [] n 19\nWEB-26 open P3 - [] n 20' "page 2"

# Deleted closed issues leave the status filter too.
post /p/BASE/1/delete '' >/dev/null
expect_eq "$(get '/p/BASE/issues.txt?status=closed')" "" "status filter"
expect_has "$(get '/p/BASE?status=closed')" 'Page 1 of 1 (0 issues)' "status filter count"
expect_eq "$(get /p/BASE/issues.txt)" 'BASE-2 open P3 bob [] Typo on home' "BASE after delete"
bb_unit
echo PASS
