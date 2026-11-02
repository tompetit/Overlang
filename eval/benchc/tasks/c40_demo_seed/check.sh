#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"

# start_with ARGS...: like bb_start, but the app gets these arguments (@PORT@ becomes a free port).
start_with() {
  PORT="$(python3 -c 'import socket;s=socket.socket();s.bind(("127.0.0.1",0));print(s.getsockname()[1])')"
  local a=("${@//@PORT@/$PORT}")
  if [ "$LANG_" = python ]; then set -- python3 -B "$APP/main.py"; else set -- ./app; fi
  (cd "$RUN" && exec "$@" "${a[@]}") >>"$RUN/server.log" 2>&1 &
  PID=$!; URL="http://127.0.0.1:$PORT"
  for _ in $(seq 100); do
    curl -s -o /dev/null --max-time 2 "$URL/health" && return 0
    kill -0 "$PID" 2>/dev/null || fail "server exited early (${a[*]})"
    sleep 0.1
  done
  fail "server did not come up (${a[*]})"
}
# lines LINE: how many lines of the server log are exactly LINE.
lines() { grep -cx -- "$1" "$RUN/server.log" || true; }
# next_after LINE: the log line right after the last LINE.
next_after() { grep -A1 -x -- "$1" "$RUN/server.log" | tail -1; }

# Without the flag nothing is seeded.
expect_eq "$(get /users.txt)" "" "fresh database has no users"
expect_eq "$(get /projects.txt)" "" "fresh database has no projects"
bb_baseline
expect_eq "$(lines 'seeded demo data')" 0 "no seeding without the flag"

# Seed a new database (flag first).
bb_stop
start_with --demo demo.db @PORT@
expect_eq "$(lines 'seeded demo data')" 1 "seeded line"
expect_eq "$(next_after 'seeded demo data')" "listening on http://127.0.0.1:$PORT" "seeded line before listening"
[ -f "$RUN/demo.db" ] || fail "demo.db not created"
expect_eq "$(get /users.txt)" $'1 alice\n2 bob\n3 carol' "users"
expect_eq "$(get /projects.txt)" $'DEMO Demo project\nOPS Operations' "projects"
expect_eq "$(get /p/DEMO/labels.txt)" $'bug\ndocs\nui' "DEMO labels"
expect_eq "$(get /p/OPS/labels.txt)" 'infra' "OPS labels"
demo_txt=$'DEMO-1 open P1 alice [bug,ui] Login button does nothing\nDEMO-2 open P3 carol [docs] Write the README\nDEMO-3 closed P4 bob [] Typo in footer'
expect_eq "$(get /p/DEMO/issues.txt)" "$demo_txt" "DEMO issues"
expect_eq "$(get /p/OPS/issues.txt)" 'OPS-1 open P2 - [infra] Rotate backups' "OPS issues"
expect_eq "$(get /p/DEMO/1.json)" '{"key":"DEMO-1","project":"DEMO","number":1,"title":"Login button does nothing","body":"Click **Login** on the home page.\n\nNothing happens.","status":"open","priority":1,"assignee":"alice","labels":["bug","ui"],"comments":[{"author":"bob","body":"Reproduced in `Firefox`."}],"activity":[{"kind":"created","detail":""},{"kind":"commented","detail":"bob"}]}' "DEMO-1 json"
expect_eq "$(get /p/DEMO/2.json)" '{"key":"DEMO-2","project":"DEMO","number":2,"title":"Write the README","body":"","status":"open","priority":3,"assignee":"carol","labels":["docs"],"comments":[],"activity":[{"kind":"created","detail":""},{"kind":"assigned","detail":"carol"}]}' "DEMO-2 json"
expect_eq "$(get /p/DEMO/3.json)" '{"key":"DEMO-3","project":"DEMO","number":3,"title":"Typo in footer","body":"\"Copyright\" is misspelled.","status":"closed","priority":4,"assignee":"bob","labels":[],"comments":[],"activity":[{"kind":"created","detail":""},{"kind":"status","detail":"closed"}]}' "DEMO-3 json"
expect_eq "$(get /p/OPS/1/activity.txt)" 'created' "OPS-1 activity"
page="$(get /p/DEMO/1)"
[[ "$page" == *'<div class="body"><p>Click <strong>Login</strong> on the home page.</p>'$'\n''<p>Nothing happens.</p></div>'* ]] || fail "DEMO-1 page body: $page"
expect_has "$page" '<li><b>bob</b> <p>Reproduced in <code>Firefox</code>.</p></li>' "DEMO-1 comment"
expect_has "$(get /p/DEMO)" 'Page 1 of 1 (3 issues)' "DEMO page"
# Row ids, as if made by the requests in order.
expect_eq "$(bb_sql 'select id, name from users order by id' demo.db)" $'1|alice\n2|bob\n3|carol' "user rows"
expect_eq "$(bb_sql 'select id, key, name from projects order by id' demo.db)" $'1|DEMO|Demo project\n2|OPS|Operations' "project rows"
expect_eq "$(bb_sql 'select id, project_id, name from labels order by id' demo.db)" $'1|1|bug\n2|1|docs\n3|1|ui\n4|2|infra' "label rows"
expect_eq "$(bb_sql 'select id, project_id, number, status, priority, coalesce(assignee_id, 0) from issues order by id' demo.db)" $'1|1|1|open|1|1\n2|1|2|open|3|3\n3|1|3|closed|4|2\n4|2|1|open|2|0' "issue rows"
expect_eq "$(bb_sql 'select issue_id, label_id from issue_labels order by issue_id, label_id' demo.db)" $'1|1\n1|3\n2|2\n4|4' "issue_labels rows"
expect_eq "$(bb_sql 'select id, issue_id, author_id, body from comments order by id' demo.db)" '1|1|2|Reproduced in `Firefox`.' "comment rows"
expect_eq "$(bb_sql 'select id, issue_id, kind, detail from activity order by id' demo.db)" $'1|1|created|\n2|2|created|\n3|3|created|\n4|1|commented|bob\n5|3|status|closed\n6|2|assigned|carol\n7|4|created|' "activity rows"
# The server works normally afterwards.
expect_has "$(post /p/DEMO/issues 'title=Mine&labels=ui')" '<h1>DEMO-4: Mine</h1>' "create after seeding"
expect_code 200 /p/OPS/1/comments 'author=carol&body=done'
expect_err 422 "user exists" /users name=alice
expect_err 404 "no such project: NOPE" /p/NOPE
bb_stop

# Restart with the flag last: skipped, nothing changes.
start_with demo.db @PORT@ --demo
expect_eq "$(lines 'seeded demo data')" 1 "not seeded twice"
expect_eq "$(lines 'demo skipped: database not empty')" 1 "skipped line"
expect_eq "$(next_after 'demo skipped: database not empty')" "listening on http://127.0.0.1:$PORT" "skipped line before listening"
expect_eq "$(get /users.txt)" $'1 alice\n2 bob\n3 carol' "users after a second --demo"
expect_eq "$(get /p/DEMO/issues.txt)" "$demo_txt"$'\nDEMO-4 open P3 - [ui] Mine' "issues after a second --demo"
expect_eq "$(get /p/OPS/1/comments.txt)" 'carol: done' "comment kept"
bb_stop

# Not empty: the baseline database, a database with only a user, one with only a project.
start_with tracker.db --demo @PORT@
expect_eq "$(lines 'demo skipped: database not empty')" 2 "skipped on the baseline database"
expect_eq "$(get /users.txt)" $'1 alice\n2 bob' "baseline users untouched"
expect_eq "$(get /projects.txt)" 'BASE Base line' "baseline projects untouched"
expect_eq "$(get /p/BASE/issues.txt)" $'BASE-1 closed P1 alice [bug,ui] Login fails\nBASE-2 open P3 bob [] Typo on home' "baseline issues untouched"
bb_stop
python3 -c 'import sqlite3,sys;c=sqlite3.connect(sys.argv[1]);c.execute("create table users (id integer primary key autoincrement, name text not null unique)");c.execute("insert into users (name) values (?)", ("zoe",));c.commit()' "$RUN/one.db"
start_with --demo one.db @PORT@
expect_eq "$(lines 'demo skipped: database not empty')" 3 "skipped with only a user"
expect_eq "$(get /users.txt)" '1 zoe' "only user kept"
expect_eq "$(get /projects.txt)" '' "no projects added"
bb_stop
python3 -c 'import sqlite3,sys;c=sqlite3.connect(sys.argv[1]);c.execute("create table projects (id integer primary key autoincrement, key text not null unique, name text not null)");c.execute("insert into projects (key, name) values (?, ?)", ("SOLO", "Solo"));c.commit()' "$RUN/two.db"
start_with two.db @PORT@ --demo
expect_eq "$(lines 'demo skipped: database not empty')" 4 "skipped with only a project"
expect_eq "$(get /users.txt)" '' "no users added"
expect_eq "$(get /projects.txt)" 'SOLO Solo' "only project kept"
bb_stop
expect_eq "$(lines 'seeded demo data')" 1 "seeded exactly once overall"

# Persistence of the seeded database without the flag.
start_with demo.db @PORT@
expect_eq "$(get /p/DEMO/4.txt)" 'DEMO-4 open P3 - [ui] Mine' "seeded database after restart"
expect_eq "$(get /p/DEMO/2/activity.txt)" $'created\nassigned carol' "activity after restart"
expect_eq "$(lines 'demo skipped: database not empty')" 4 "nothing printed without the flag"
bb_stop
bb_start
expect_eq "$(get /p/BASE/1.txt)" 'BASE-1 closed P1 alice [bug,ui] Login fails' "baseline database after restart"
bb_unit
echo PASS
