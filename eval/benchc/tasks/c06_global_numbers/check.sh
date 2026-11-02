#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
# Old behavior on BASE (fresh db: BASE-1, BASE-2), so the sequence continues at 3.
bb_baseline
post /projects 'key=WEB&name=Web' >/dev/null
post /projects 'key=API&name=Api' >/dev/null
post /p/WEB/labels name=bug >/dev/null
post /p/API/labels name=bug >/dev/null
expect_has "$(post /p/WEB/issues 'title=Alpha&labels=bug&assignee=alice&priority=2')" '<h1>WEB-3: Alpha</h1>' "WEB continues the global sequence"
expect_has "$(post /p/API/issues 'title=Beta&body=b+%22q%22')" '<h1>API-4: Beta</h1>' "API gets the next global number"
post /p/WEB/issues 'title=Gamma' >/dev/null
post /p/API/issues 'title=Delta&labels=bug' >/dev/null
expect_eq "$(get /p/WEB/issues.txt)" $'WEB-3 open P2 alice [bug] Alpha\nWEB-5 open P3 - [] Gamma' "WEB issues"
expect_eq "$(get /p/API/issues.txt)" $'API-4 open P3 - [] Beta\nAPI-6 open P3 - [bug] Delta' "API issues"
expect_eq "$(get /p/BASE/issues.txt)" $'BASE-1 closed P1 alice [bug,ui] Login fails\nBASE-2 open P3 bob [] Typo on home' "BASE issues"
expect_eq "$(get '/p/WEB/issues.txt?label=bug')" 'WEB-3 open P2 alice [bug] Alpha' "label filter"

# A number of another project is not found under this project, for every route.
expect_err 404 "no such issue: API-3" /p/API/3
expect_err 404 "no such issue: WEB-4" /p/WEB/4.json
expect_err 404 "no such issue: WEB-4" /p/WEB/4.txt
expect_err 404 "no such issue: WEB-4" /p/WEB/4/activity.txt
expect_err 404 "no such issue: WEB-4" /p/WEB/4/status status=closed
expect_err 404 "no such issue: BASE-3" /p/BASE/3/comments 'author=bob&body=x'
expect_eq "$(get /p/API/4.txt)" 'API-4 open P3 - [] Beta' "API-4 untouched by the refused post"
expect_eq "$(get /p/API/4.json)" '{"key":"API-4","project":"API","number":4,"title":"Beta","body":"b \"q\"","status":"open","priority":3,"assignee":null,"labels":[],"comments":[],"activity":[{"kind":"created","detail":""}]}' "json"

# /i routes.
expect_eq "$(get /i/5.txt)" 'WEB-5 open P3 - [] Gamma' "/i/5.txt"
expect_eq "$(get /i/1.txt)" 'BASE-1 closed P1 alice [bug,ui] Login fails' "/i/1.txt"
expect_eq "$(ctype /i/5.txt)" 'text/plain; charset=utf-8' "/i/N.txt content type"
page="$(get /i/4)"
expect_has "$page" '<h1>API-4: Beta</h1>' "/i/4 page"
expect_has "$page" 'action="/p/API/4/status"' "/i/4 page forms"
expect_eq "$(ctype /i/4)" 'text/html; charset=utf-8' "/i/N content type"
expect_err 404 "no such issue: 99" /i/99
expect_err 404 "no such issue: 99" /i/99.txt
expect_err 404 "not found" /i/0
expect_err 404 "not found" /i/abc
expect_err 404 "not found" /i/5.json
expect_err 404 "not found" /i
expect_err 404 "not found" /i/5 x=1
post /p/WEB/5/status status=closed >/dev/null
expect_eq "$(get /i/5.txt)" 'WEB-5 closed P3 - [] Gamma' "/i sees changes"
expect_eq "$(get /p/WEB/5/activity.txt)" $'created\nstatus closed' "activity"

# The database rejects a duplicate number in another project (BASE=1, WEB=2, API=3).
if bb_sql "insert into issues (project_id, number, title) values (3, 5, 'dup')" >/dev/null 2>&1; then
  fail "issues table accepted a second issue number 5"
fi
expect_eq "$(bb_sql 'select count(*) from issues where number = 5')" 1 "one issue 5"

# A project with no issues yet still continues the sequence; failed creations use no number.
post /projects 'key=MOB&name=Mobile' >/dev/null
expect_err 422 "title required" /p/MOB/issues title=
expect_err 404 "no such project: NOPE" /p/NOPE/issues title=x
expect_has "$(post /p/MOB/issues title=First)" '<h1>MOB-7: First</h1>' "new project's first issue"

bb_restart
expect_eq "$(get /i/7.txt)" 'MOB-7 open P3 - [] First' "persisted"
post /p/API/issues title=Eps >/dev/null
expect_eq "$(get /p/API/issues.txt)" $'API-4 open P3 - [] Beta\nAPI-6 open P3 - [bug] Delta\nAPI-8 open P3 - [] Eps' "sequence survives restart"
getf /p/API/issues.csv "$RUN/api.csv"
expect_file "$RUN/api.csv" 'key,title,status,priority,assignee,labels,body\nAPI-4,Beta,open,3,,,"b ""q"""\nAPI-6,Delta,open,3,,bug,\nAPI-8,Eps,open,3,,,\n'
for i in $(seq 19); do post /p/WEB/issues "title=n+$i" >/dev/null; done
expect_eq "$(get /p/WEB/issues.txt | wc -l | tr -d ' ')" 20 "page 1 size"
expect_eq "$(get '/p/WEB/issues.txt?page=2')" 'WEB-27 open P3 - [] n 19' "page 2"
expect_has "$(get /p/WEB)" 'Page 1 of 2 (21 issues)' "pager"
expect_eq "$(get /i/26.txt)" 'WEB-26 open P3 - [] n 18' "/i on a later issue"
bb_unit
echo PASS
