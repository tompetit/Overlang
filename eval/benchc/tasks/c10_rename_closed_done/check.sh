#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
# Old behavior that does not involve the status value (bb_baseline closes an issue, so it is not used).
expect_eq "$(get /health)" "ok" "health"
post /users name=alice >/dev/null; post /users name=bob >/dev/null
expect_eq "$(get /users.txt)" $'1 alice\n2 bob' "users.txt"
expect_err 422 "name required" /users name=
post /projects 'key=WEB&name=Web+site' >/dev/null
expect_eq "$(get /projects.txt)" "WEB Web site" "projects.txt"
expect_has "$(get /)" '<a href="/p/WEB">WEB</a>' "home page"
post /p/WEB/labels name=bug >/dev/null; post /p/WEB/labels name=ui >/dev/null
post /p/WEB/issues 'title=Login+fails&body=Steps&priority=1&assignee=alice&labels=bug,ui' >/dev/null
post /p/WEB/issues 'title=Typo' >/dev/null
expect_eq "$(get /p/WEB/issues.txt)" $'WEB-1 open P1 alice [bug,ui] Login fails\nWEB-2 open P3 - [] Typo' "issues.txt"
post /p/WEB/2/assign assignee=bob >/dev/null
post /p/WEB/1/comments 'author=bob&body=on+it' >/dev/null
expect_eq "$(get /p/WEB/1/comments.txt)" 'bob: on it' "comments"
expect_eq "$(get '/p/WEB/issues.txt?assignee=bob')" 'WEB-2 open P3 bob [] Typo' "assignee filter"
expect_err 422 "title required" /p/WEB/issues title=
expect_err 404 "no such issue: WEB-99" /p/WEB/99
expect_err 404 "not found" /nope

# The new value.
page="$(get /p/WEB/1)"
expect_has "$page" '<form method="post" action="/p/WEB/1/status"><input type="hidden" name="status" value="done"><button>done</button></form>' "toggle form of an open issue"
expect_lacks "$page" 'closed' "no closed on the page"
expect_err 422 "bad status" /p/WEB/1/status status=closed
expect_err 422 "bad status" /p/WEB/1/status status=Done
expect_err 400 "bad status" '/p/WEB?status=closed'
expect_err 400 "bad status" '/p/WEB/issues.txt?status=closed'
r="$(post /p/WEB/1/status status=done)"
expect_has "$r" '<span class="status">done</span>' "issue page"
expect_has "$r" '<input type="hidden" name="status" value="open"><button>reopen</button></form>' "reopen form"
expect_eq "$(get /p/WEB/issues.txt)" $'WEB-1 done P1 alice [bug,ui] Login fails\nWEB-2 open P3 bob [] Typo' "issue lines"
expect_eq "$(get '/p/WEB/issues.txt?status=done')" 'WEB-1 done P1 alice [bug,ui] Login fails' "status=done"
expect_eq "$(get '/p/WEB/issues.txt?status=open')" 'WEB-2 open P3 bob [] Typo' "status=open"
expect_has "$(get '/p/WEB?status=done')" '<td>done</td>' "project row"
expect_has "$(get '/p/WEB?status=done')" '<input name="status" value="done" placeholder="status">' "filter form"
expect_eq "$(get /p/WEB/1/activity.txt)" $'created\ncommented bob\nstatus done' "activity"
expect_eq "$(get /p/WEB/1.json)" '{"key":"WEB-1","project":"WEB","number":1,"title":"Login fails","body":"Steps","status":"done","priority":1,"assignee":"alice","labels":["bug","ui"],"comments":[{"author":"bob","body":"on it"}],"activity":[{"kind":"created","detail":""},{"kind":"commented","detail":"bob"},{"kind":"status","detail":"done"}]}' "json"
getf /p/WEB/issues.csv "$RUN/web.csv"
expect_file "$RUN/web.csv" 'key,title,status,priority,assignee,labels,body\nWEB-1,Login fails,done,1,alice,bug;ui,Steps\nWEB-2,Typo,open,3,bob,,\n'
expect_eq "$(bb_sql 'select status from issues order by id')" $'done\nopen' "stored value"
if bb_sql "insert into issues (project_id, number, title, status) values (1, 90, 'x', 'closed')" >/dev/null 2>&1; then
  fail "issues table accepted status closed"
fi
bb_sql "insert into issues (project_id, number, title, status) values (1, 91, 'x', 'done')" >/dev/null 2>&1 \
  || fail "issues table rejected status done"
bb_sql "delete from issues where number = 91" >/dev/null
post /p/WEB/1/status status=open >/dev/null
post /p/WEB/1/status status=done >/dev/null
expect_eq "$(get /p/WEB/1/activity.txt)" $'created\ncommented bob\nstatus done\nstatus open\nstatus done' "reopen and done"
bb_restart
expect_eq "$(get /p/WEB/issues.txt)" $'WEB-1 done P1 alice [bug,ui] Login fails\nWEB-2 open P3 bob [] Typo' "persisted"
expect_has "$(post /p/WEB/issues title=Three)" '<h1>WEB-3: Three</h1>' "numbering after restart"

# Upgrade a database written by the previous version.
bb_stop
rm -f "$RUN/tracker.db"
python3 - "$RUN/tracker.db" <<'PY' || fail "could not create the old database"
import sqlite3, sys
c = sqlite3.connect(sys.argv[1])
c.executescript("""
create table if not exists users (id integer primary key autoincrement, name text not null unique);
create table if not exists projects (id integer primary key autoincrement, key text not null unique, name text not null);
create table if not exists issues (
  id integer primary key autoincrement,
  project_id integer not null references projects(id),
  number integer not null,
  title text not null,
  body text not null default '',
  status text not null default 'open' check (status in ('open', 'closed')),
  priority integer not null default 3 check (priority between 1 and 4),
  assignee_id integer references users(id),
  unique (project_id, number)
);
create table if not exists labels (id integer primary key autoincrement, project_id integer not null references projects(id), name text not null, unique (project_id, name));
create table if not exists issue_labels (issue_id integer not null references issues(id), label_id integer not null references labels(id), primary key (issue_id, label_id));
create table if not exists comments (id integer primary key autoincrement, issue_id integer not null references issues(id), author_id integer not null references users(id), body text not null);
create table if not exists activity (id integer primary key autoincrement, issue_id integer not null references issues(id), kind text not null check (kind in ('created', 'status', 'assigned', 'commented')), detail text not null default '');
insert into users (name) values ('alice'), ('bob'), ('closed');
insert into projects (key, name) values ('OLD', 'Old project');
insert into labels (project_id, name) values (1, 'bug');
insert into issues (project_id, number, title, body, status, priority, assignee_id) values
  (1, 1, 'Crash on save', 'it is closed now', 'closed', 2, 1),
  (1, 2, 'Slow when closed', '', 'open', 3, 2),
  (1, 3, 'Typo', '', 'closed', 4, null);
insert into issue_labels (issue_id, label_id) values (1, 1);
insert into comments (issue_id, author_id, body) values (1, 2, 'seen it');
insert into activity (issue_id, kind, detail) values
  (1, 'created', ''), (2, 'created', ''), (3, 'created', ''),
  (1, 'status', 'closed'), (2, 'assigned', 'closed'), (2, 'assigned', 'bob'),
  (3, 'status', 'closed'), (3, 'status', 'open'), (3, 'status', 'closed'), (1, 'commented', 'bob');
""")
c.commit()
PY
bb_start
expect_eq "$(get /p/OLD/issues.txt)" $'OLD-1 done P2 alice [bug] Crash on save\nOLD-2 open P3 bob [] Slow when closed\nOLD-3 done P4 - [] Typo' "upgraded issues"
expect_eq "$(get /p/OLD/1/activity.txt)" $'created\nstatus done\ncommented bob' "upgraded activity 1"
expect_eq "$(get /p/OLD/2/activity.txt)" $'created\nassigned closed\nassigned bob' "assigned rows untouched"
expect_eq "$(get /p/OLD/3/activity.txt)" $'created\nstatus done\nstatus open\nstatus done' "upgraded activity 3"
expect_eq "$(get /p/OLD/1.json)" '{"key":"OLD-1","project":"OLD","number":1,"title":"Crash on save","body":"it is closed now","status":"done","priority":2,"assignee":"alice","labels":["bug"],"comments":[{"author":"bob","body":"seen it"}],"activity":[{"kind":"created","detail":""},{"kind":"status","detail":"done"},{"kind":"commented","detail":"bob"}]}' "upgraded json"
expect_eq "$(get /users.txt)" $'1 alice\n2 bob\n3 closed' "users kept"
expect_eq "$(get '/p/OLD/issues.txt?status=done')" $'OLD-1 done P2 alice [bug] Crash on save\nOLD-3 done P4 - [] Typo' "filter on upgraded rows"
expect_eq "$(bb_sql 'select id, number from issues order by id')" $'1|1\n2|2\n3|3' "ids kept"
expect_eq "$(bb_sql "select count(*) from issues where status = 'closed'")" 0 "no closed issues"
expect_eq "$(bb_sql "select count(*) from activity where kind = 'status' and detail = 'closed'")" 0 "no closed activity"
if bb_sql "insert into issues (project_id, number, title, status) values (1, 90, 'x', 'closed')" >/dev/null 2>&1; then
  fail "upgraded issues table accepted status closed"
fi
bb_sql "insert into issues (project_id, number, title, status) values (1, 91, 'x', 'done')" >/dev/null 2>&1 \
  || fail "upgraded issues table rejected status done"
bb_sql "delete from issues where number = 91" >/dev/null
expect_has "$(post /p/OLD/issues 'title=New&labels=bug')" '<h1>OLD-4: New</h1>' "numbering continues"
expect_eq "$(get /p/OLD/4.txt)" 'OLD-4 open P3 - [bug] New' "labels on a new issue"
post /p/OLD/2/status status=done >/dev/null
expect_eq "$(get /p/OLD/2/activity.txt)" $'created\nassigned closed\nassigned bob\nstatus done' "status on an upgraded db"
expect_err 422 "bad status" /p/OLD/3/status status=closed
bb_restart
expect_eq "$(get /p/OLD/issues.txt)" $'OLD-1 done P2 alice [bug] Crash on save\nOLD-2 done P3 bob [] Slow when closed\nOLD-3 done P4 - [] Typo\nOLD-4 open P3 - [bug] New' "second start changes nothing"
expect_eq "$(get /p/OLD/1/comments.txt)" 'bob: seen it' "comments kept"
expect_eq "$(get /p/OLD/3/activity.txt)" $'created\nstatus done\nstatus open\nstatus done' "activity after second start"
bb_unit
echo PASS
