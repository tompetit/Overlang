#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
bb_baseline
# Unaffected routes still work.
post /p/BASE/2/labels label=ui >/dev/null
post /p/BASE/2/labels/remove label=ui >/dev/null
post /p/BASE/2/edit 'title=Typo+on+home+page' >/dev/null
expect_eq "$(get /p/BASE/2.txt)" 'BASE-2 open P3 bob [] Typo on home page' "edit"
expect_has "$(get /p/BASE/1.json)" '{"key":"BASE-1","project":"BASE","number":1,' "json"
expect_eq "$(get /p/BASE/issues.csv | head -1)" 'key,title,status,priority,assignee,labels,body' "csv"

# Stored in projects.open_limit, no limit by default.
expect_eq "$(bb_sql "select key, open_limit from projects")" 'BASE|0' "new column"
expect_eq "$(get /p/BASE/limit.txt)" 'open 1, no limit' "limit.txt without a limit"
expect_eq "$(ctype /p/BASE/limit.txt)" "text/plain; charset=utf-8" "limit.txt content type"
page="$(get /p/BASE)"
expect_has "$page" '(2 issues)</p><form method="post" action="/p/BASE/limit"><input name="limit" value="0"> <button>set limit</button></form><h2>Labels</h2>' "page without a limit"
expect_lacks "$page" 'class="limit"' "no limit paragraph"

# Setting it: errors first.
expect_err 400 "bad limit" /p/BASE/limit limit=
expect_err 400 "bad limit" /p/BASE/limit x=1
expect_err 400 "bad limit" /p/BASE/limit limit=abc
expect_err 400 "bad limit" /p/BASE/limit limit=-1
expect_err 400 "bad limit" /p/BASE/limit limit=%2B2
expect_err 400 "bad limit" /p/BASE/limit limit=2.0
expect_err 400 "bad limit" /p/BASE/limit limit=1234567890
expect_err 422 "bad limit" /p/BASE/limit limit=1001
expect_err 422 "bad limit" /p/BASE/limit limit=999999999
expect_err 404 "no such project: NOPE" /p/NOPE/limit limit=x
expect_err 404 "not found" /p/BASE/limit
expect_err 404 "no such project: NOPE" /p/NOPE/limit.txt
expect_eq "$(get /p/BASE/limit.txt)" 'open 1, no limit' "nothing stored on errors"
page="$(post /p/BASE/limit 'limit=+2+')"
expect_has "$page" '<h1>BASE: Base line</h1>' "answers the project page"
expect_has "$page" '(2 issues)</p><p class="limit">Open issues: 1 of 2</p><form method="post" action="/p/BASE/limit"><input name="limit" value="2"> <button>set limit</button></form><h2>Labels</h2>' "page with a limit"
expect_eq "$(get /p/BASE/limit.txt)" 'open 1 of 2' "limit.txt"
expect_eq "$(bb_sql "select open_limit from projects where key = 'BASE'")" 2 "stored"
expect_code 200 /p/BASE/limit limit=1000
expect_code 200 /p/BASE/limit limit=0002

# Creating up to the limit, then blocked; existing checks first.
expect_has "$(post /p/BASE/issues 'title=Third')" '<h1>BASE-3: Third</h1>' "create under the limit"
expect_eq "$(get /p/BASE/limit.txt)" 'open 2 of 2' "at the limit"
expect_err 422 "open issue limit reached: 2" /p/BASE/issues 'title=Fourth'
expect_err 422 "title required" /p/BASE/issues 'title=+'
expect_err 422 "bad priority" /p/BASE/issues 'title=x&priority=7'
expect_err 400 "bad priority" /p/BASE/issues 'title=x&priority=x'
expect_err 422 "no such user: zed" /p/BASE/issues 'title=x&assignee=zed'
expect_err 422 "no such label: nope" /p/BASE/issues 'title=x&labels=bug,nope'
expect_err 422 "open issue limit reached: 2" /p/BASE/issues 'title=x&priority=1&assignee=alice&labels=bug'
expect_err 404 "no such project: NOPE" /p/NOPE/issues 'title=x'
expect_eq "$(get /p/BASE/issues.txt)" $'BASE-1 closed P1 alice [bug,ui] Login fails\nBASE-2 open P3 bob [] Typo on home page\nBASE-3 open P3 - [] Third' "nothing created"

# Reopening is limited; a no-op open and closing are not; other writes are not.
expect_err 422 "open issue limit reached: 2" /p/BASE/1/status status=open
expect_err 422 "bad status" /p/BASE/1/status status=reopen
expect_eq "$(get /p/BASE/1/activity.txt)" $'created\nstatus closed\ncommented bob' "no activity on a blocked reopen"
expect_has "$(post /p/BASE/2/status status=open)" '<span class="status">open</span>' "open on an open issue"
expect_code 200 /p/BASE/3/edit 'title=Third+one&priority=2'
expect_code 200 /p/BASE/3/assign assignee=alice
expect_code 200 /p/BASE/3/comments 'author=bob&body=hi'
expect_code 200 /p/BASE/3/labels label=bug
expect_code 200 /p/BASE/1/comments 'author=alice&body=still+closed'
expect_eq "$(get /p/BASE/3/activity.txt)" $'created\nassigned alice\ncommented bob' "writes at the limit"
expect_code 200 /p/BASE/2/status status=closed
expect_eq "$(get /p/BASE/limit.txt)" 'open 1 of 2' "after closing"
expect_code 200 /p/BASE/1/status status=open
expect_eq "$(get /p/BASE/1.txt)" 'BASE-1 open P1 alice [bug,ui] Login fails' "reopened under the limit"
expect_eq "$(get /p/BASE/1/activity.txt)" $'created\nstatus closed\ncommented bob\ncommented alice\nstatus open' "reopen activity"

# Lowering below the count; other projects unaffected; removing.
expect_code 200 /p/BASE/limit limit=1
expect_eq "$(get /p/BASE/limit.txt)" 'open 2 of 1' "limit below the count"
expect_has "$(get /p/BASE)" '<p class="limit">Open issues: 2 of 1</p>' "page below the count"
expect_err 422 "open issue limit reached: 1" /p/BASE/issues 'title=x'
expect_err 422 "open issue limit reached: 1" /p/BASE/2/status status=open
post /projects 'key=API&name=Api' >/dev/null
expect_has "$(post /p/API/issues 'title=Free')" '<h1>API-1: Free</h1>' "other project unaffected"
expect_eq "$(get /p/API/limit.txt)" 'open 1, no limit' "other project limit.txt"
expect_code 200 /p/API/limit limit=1
expect_err 422 "open issue limit reached: 1" /p/API/issues 'title=More'

# Persistence.
bb_restart
expect_eq "$(get /p/BASE/limit.txt)" 'open 2 of 1' "limit after restart"
expect_err 422 "open issue limit reached: 1" /p/BASE/issues 'title=x'
expect_code 200 /p/BASE/limit limit=0
expect_eq "$(get /p/BASE/limit.txt)" 'open 2, no limit' "removed"
expect_has "$(get /p/BASE)" '(3 issues)</p><form method="post" action="/p/BASE/limit"><input name="limit" value="0">' "page after removing"
expect_has "$(post /p/BASE/issues 'title=Fourth')" '<h1>BASE-4: Fourth</h1>' "create after removing"
expect_code 200 /p/BASE/2/status status=open

# Upgrade a database written by the current version (no open_limit column).
bb_stop
rm -f "$RUN/tracker.db"
python3 - "$RUN/tracker.db" <<'PY' || fail "could not create the old database"
import sqlite3, sys
c = sqlite3.connect(sys.argv[1])
c.executescript("""
create table users (id integer primary key autoincrement, name text not null unique);
create table projects (id integer primary key autoincrement, key text not null unique, name text not null);
create table issues (
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
create table labels (id integer primary key autoincrement, project_id integer not null references projects(id), name text not null, unique (project_id, name));
create table issue_labels (issue_id integer not null references issues(id), label_id integer not null references labels(id), primary key (issue_id, label_id));
create table comments (id integer primary key autoincrement, issue_id integer not null references issues(id), author_id integer not null references users(id), body text not null);
create table activity (id integer primary key autoincrement, issue_id integer not null references issues(id), kind text not null check (kind in ('created', 'status', 'assigned', 'commented')), detail text not null default '');
insert into users (name) values ('alice'), ('bob');
insert into projects (key, name) values ('OLD', 'Old project'), ('ACME', 'Acme');
insert into labels (project_id, name) values (1, 'bug');
insert into issues (project_id, number, title, body, status, priority, assignee_id) values
  (1, 1, 'Crash on save', 'body', 'open', 2, 1),
  (1, 2, 'Slow', '', 'open', 3, null),
  (1, 3, 'Done', '', 'closed', 4, 2);
insert into issue_labels (issue_id, label_id) values (1, 1);
insert into comments (issue_id, author_id, body) values (1, 2, 'seen it');
insert into activity (issue_id, kind, detail) values (1, 'created', ''), (2, 'created', ''), (3, 'created', ''), (3, 'status', 'closed'), (1, 'commented', 'bob');
""")
c.commit()
PY
bb_start
expect_eq "$(get /projects.txt)" $'ACME Acme\nOLD Old project' "old projects"
expect_eq "$(get /p/OLD/issues.txt)" $'OLD-1 open P2 alice [bug] Crash on save\nOLD-2 open P3 - [] Slow\nOLD-3 closed P4 bob [] Done' "old issues"
expect_eq "$(get /p/OLD/1/comments.txt)" 'bob: seen it' "old comments"
expect_eq "$(get /p/OLD/3/activity.txt)" $'created\nstatus closed' "old activity"
expect_eq "$(bb_sql "select key, open_limit from projects order by id")" $'OLD|0\nACME|0' "column added"
expect_eq "$(get /p/OLD/limit.txt)" 'open 2, no limit' "old project has no limit"
expect_has "$(get /p/OLD)" '<input name="limit" value="0">' "old project page"
expect_code 200 /p/OLD/limit limit=3
expect_has "$(post /p/OLD/issues 'title=New')" '<h1>OLD-4: New</h1>' "numbering continues"
expect_err 422 "open issue limit reached: 3" /p/OLD/3/status status=open
bb_restart
expect_eq "$(get /p/OLD/limit.txt)" 'open 3 of 3' "limit after a second start"
expect_err 422 "open issue limit reached: 3" /p/OLD/issues 'title=x'
expect_eq "$(bb_sql "select count(*) from pragma_table_info('projects') where name = 'open_limit'")" 1 "one column"
expect_eq "$(get /p/ACME/limit.txt)" 'open 0, no limit' "empty project"
bb_unit
echo PASS
