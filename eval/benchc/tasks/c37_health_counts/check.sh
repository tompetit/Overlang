#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"

# Empty database.
getf /health "$RUN/h0"
expect_file "$RUN/h0" 'ok\nusers 0\nprojects 0\nissues 0\nopen 0\nclosed 0\nlabels 0\ncomments 0\n'
expect_eq "$(ctype /health)" "text/plain; charset=utf-8" "health content type"
getf /health.json "$RUN/j0"
expect_file "$RUN/j0" '{"status":"ok","users":0,"projects":0,"issues":0,"open":0,"closed":0,"labels":0,"comments":0,"by_project":[]}\n'
expect_eq "$(ctype /health.json)" "application/json" "health.json content type"

# Old behaviour (bb_baseline expects the old /health body, so its steps are here without it).
post /users name=alice >/dev/null; post /users name=bob >/dev/null
expect_eq "$(get /users.txt)" $'1 alice\n2 bob' "users.txt"
expect_err 422 "user exists" /users name=alice
expect_err 422 "bad name" /users "name=a+b"
post /projects 'key=BASE&name=Base+line' >/dev/null
expect_eq "$(get /projects.txt)" "BASE Base line" "projects.txt"
expect_err 422 "bad key" /projects 'key=base&name=x'
expect_has "$(get /)" '<a href="/p/BASE">BASE</a>' "home page"
post /p/BASE/labels name=bug >/dev/null; post /p/BASE/labels name=ui >/dev/null
expect_eq "$(get /p/BASE/labels.txt)" $'bug\nui' "labels.txt"
post /p/BASE/issues 'title=Login+fails&body=Steps%3A+**click**&priority=1&assignee=alice&labels=bug,ui' >/dev/null
post /p/BASE/issues 'title=Typo+on+home&body=50%25+of+the+text' >/dev/null
expect_eq "$(get /p/BASE/issues.txt)" $'BASE-1 open P1 alice [bug,ui] Login fails\nBASE-2 open P3 - [] Typo on home' "issues.txt"
expect_has "$(get /p/BASE/1)" '<strong>click</strong>' "issue page markdown"
post /p/BASE/1/status status=closed >/dev/null
post /p/BASE/2/assign assignee=bob >/dev/null
post /p/BASE/1/comments 'author=bob&body=Fixed+in+%3Cb%3Ev2%3C%2Fb%3E' >/dev/null
expect_eq "$(get /p/BASE/1/activity.txt)" $'created\nstatus closed\ncommented bob' "activity.txt"
expect_eq "$(get /p/BASE/1/comments.txt)" 'bob: Fixed in <b>v2</b>' "comments.txt"
expect_eq "$(get '/p/BASE/issues.txt?status=open')" 'BASE-2 open P3 bob [] Typo on home' "status filter"
expect_has "$(get /p/BASE/1.json)" '{"key":"BASE-1","project":"BASE","number":1,' "issue json"
expect_err 404 "no such project: NOPE" /p/NOPE
expect_err 404 "no such issue: BASE-99" /p/BASE/99
expect_err 400 "bad page" '/p/BASE?page=abc'
expect_err 422 "title required" /p/BASE/issues title=
expect_err 422 "bad status" /p/BASE/1/status status=done
expect_err 404 "not found" /nope

# Counts.
expect_eq "$(get /health)" $'ok\nusers 2\nprojects 1\nissues 2\nopen 1\nclosed 1\nlabels 2\ncomments 1\nproject BASE 1 1' "health after baseline"
post /projects 'key=API&name=Api' >/dev/null
post /projects 'key=ZED&name=Last' >/dev/null
post /users name=carol >/dev/null
post /p/ZED/labels name=docs >/dev/null
post /p/ZED/issues 'title=One' >/dev/null
post /p/ZED/issues 'title=Two' >/dev/null
post /p/ZED/issues 'title=Three' >/dev/null
post /p/ZED/2/status status=closed >/dev/null
post /p/ZED/3/status status=closed >/dev/null
post /p/ZED/3/status status=open >/dev/null
post /p/ZED/1/comments 'author=carol&body=a' >/dev/null
post /p/ZED/1/comments 'author=alice&body=b' >/dev/null
post /p/BASE/2/labels label=bug >/dev/null
post /p/BASE/2/edit title=Renamed >/dev/null
getf /health "$RUN/h1"
expect_file "$RUN/h1" 'ok\nusers 3\nprojects 3\nissues 5\nopen 3\nclosed 2\nlabels 3\ncomments 3\nproject API 0 0\nproject BASE 1 1\nproject ZED 2 1\n'
getf /health.json "$RUN/j1"
expect_file "$RUN/j1" '{"status":"ok","users":3,"projects":3,"issues":5,"open":3,"closed":2,"labels":3,"comments":3,"by_project":[{"key":"API","open":0,"closed":0},{"key":"BASE","open":1,"closed":1},{"key":"ZED","open":2,"closed":1}]}\n'
expect_eq "$(get '/health?verbose=1')" "$(get /health)" "query ignored"
expect_eq "$(get '/health.json?x=y')" "$(get /health.json)" "json query ignored"
expect_eq "$(get /health/)" "$(get /health)" "trailing slash"
expect_err 404 "not found" /health x=1
expect_err 404 "not found" /health.json x=1
expect_err 404 "not found" /health.txt
expect_err 404 "not found" /health/x

# Persistence.
bb_restart
getf /health "$RUN/h2"; cmp -s "$RUN/h1" "$RUN/h2" || fail "health after restart: $(cat "$RUN/h2")"
expect_eq "$(get /health.json)" '{"status":"ok","users":3,"projects":3,"issues":5,"open":3,"closed":2,"labels":3,"comments":3,"by_project":[{"key":"API","open":0,"closed":0},{"key":"BASE","open":1,"closed":1},{"key":"ZED","open":2,"closed":1}]}' "json after restart"

# A broken database: 503, never partial counts; other routes unaffected.
bb_sql 'drop table comments' >/dev/null
expect_err 503 "db error" /health
expect_err 503 "db error" /health.json
expect_eq "$(ctype /health)" "text/plain; charset=utf-8" "503 content type"
expect_eq "$(ctype /health.json)" "text/plain; charset=utf-8" "503 json route content type"
expect_eq "$(get /p/BASE/issues.txt)" $'BASE-1 closed P1 alice [bug,ui] Login fails\nBASE-2 open P3 bob [bug] Renamed' "issues.txt still served"
expect_eq "$(get /projects.txt)" $'API Api\nBASE Base line\nZED Last' "projects.txt still served"
bb_restart
expect_eq "$(get /health)" $'ok\nusers 3\nprojects 3\nissues 5\nopen 3\nclosed 2\nlabels 3\ncomments 0\nproject API 0 0\nproject BASE 1 1\nproject ZED 2 1' "health after the table is recreated"
bb_sql 'drop table labels' >/dev/null
expect_err 503 "db error" /health
expect_err 503 "db error" /health.json
bb_unit
echo PASS
