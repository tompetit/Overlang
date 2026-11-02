#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
# Old behaviour (issue_line changes, so not bb_baseline): users, projects, labels, comments, activity, errors.
expect_eq "$(get /health)" "ok" "health"
post /users name=alice >/dev/null; post /users name=bob >/dev/null
expect_eq "$(get /users.txt)" $'1 alice\n2 bob' "users.txt"
post /projects 'key=WEB&name=Web+site' >/dev/null
expect_eq "$(get /projects.txt)" "WEB Web site" "projects.txt"
post /p/WEB/labels name=bug >/dev/null; post /p/WEB/labels name=ui >/dev/null
expect_eq "$(get /p/WEB/labels.txt)" $'bug\nui' "labels.txt"
post /p/WEB/issues 'title=Login+fails&body=Steps%3A+**click**+%60login%60&priority=1&assignee=alice&labels=bug,ui' >/dev/null
post /p/WEB/issues 'title=Typo+on+home&body=50%25+of+the+text' >/dev/null
post /p/WEB/1/status status=closed >/dev/null
post /p/WEB/2/assign assignee=bob >/dev/null
post /p/WEB/1/comments 'author=bob&body=Fixed+in+%3Cb%3Ev2%3C%2Fb%3E' >/dev/null
expect_eq "$(get /p/WEB/1/activity.txt)" $'created\nstatus closed\ncommented bob' "activity.txt"
expect_eq "$(get /p/WEB/1/comments.txt)" 'bob: Fixed in <b>v2</b>' "comments.txt"
expect_has "$(get /p/WEB/1)" '<strong>click</strong>' "markdown"
expect_err 404 "no such project: NOPE" /p/NOPE
expect_err 404 "no such issue: WEB-99" /p/WEB/99
expect_err 400 "bad status" '/p/WEB?status=done'
expect_err 422 "title required" /p/WEB/issues title=
expect_err 422 "no such label: nope" /p/WEB/1/labels label=nope
expect_err 404 "not found" /nope

# No due date shows as "-".
expect_eq "$(get /p/WEB/issues.txt)" $'WEB-1 closed P1 alice [bug,ui] - Login fails\nWEB-2 open P3 bob [] - Typo on home' "issues.txt"

# Create with due dates (form values are trimmed; "-" and "" mean none).
expect_code 200 /p/WEB/issues 'title=Ship+it&priority=2&due=2025-03-01'
expect_code 200 /p/WEB/issues 'title=Leap&due=+2000-02-29+'
expect_code 200 /p/WEB/issues 'title=Empty+due&due='
expect_code 200 /p/WEB/issues 'title=Dash+due&due=-'
want=$'WEB-1 closed P1 alice [bug,ui] - Login fails\nWEB-2 open P3 bob [] - Typo on home\nWEB-3 open P2 - [] 2025-03-01 Ship it\nWEB-4 open P3 - [] 2000-02-29 Leap\nWEB-5 open P3 - [] - Empty due\nWEB-6 open P3 - [] - Dash due'
expect_eq "$(get /p/WEB/issues.txt)" "$want" "issues.txt with due"

# Errors on create: shape 400, calendar 422, existing checks first.
for d in 2025-3-01 25-03-01 2025/03/01 2025-03-1 abcd-ef-gh 2025-03-01x 20250301 today; do
  expect_err 400 "bad due date" /p/WEB/issues "title=x&due=$d"
done
for d in 2025-02-29 1900-02-29 2023-02-29 2025-13-01 2025-00-10 2025-04-31 2025-01-00 2025-01-32 0000-01-01; do
  expect_err 422 "bad due date" /p/WEB/issues "title=x&due=$d"
done
expect_err 422 "title required" /p/WEB/issues 'title=&due=xx'
expect_err 422 "bad priority" /p/WEB/issues 'title=x&priority=9&due=xx'
expect_err 400 "bad priority" /p/WEB/issues 'title=x&priority=abc&due=2025-02-30'
expect_err 422 "no such user: zed" /p/WEB/issues 'title=x&assignee=zed&due=xx'
expect_err 422 "no such label: nope" /p/WEB/issues 'title=x&labels=nope&due=2025-02-30'
expect_eq "$(get /p/WEB/issues.txt)" "$want" "nothing stored on errors"

# Edit: set, keep on empty, remove with "-"; no activity.
expect_code 200 /p/WEB/1/edit due=2025-01-15
expect_code 200 /p/WEB/1/edit title=Login+broken
expect_code 200 /p/WEB/3/edit due=-
expect_code 200 /p/WEB/4/edit priority=1
expect_code 200 /p/WEB/6/edit 'due=2024-02-29'
expect_err 422 "bad due date" /p/WEB/2/edit due=2025-02-29
expect_err 400 "bad due date" /p/WEB/2/edit due=x
expect_err 422 "bad priority" /p/WEB/2/edit 'priority=0&due=x'
expect_err 400 "bad priority" /p/WEB/2/edit 'priority=x&due=x'
expect_err 404 "no such issue: WEB-9" /p/WEB/9/edit due=x
want=$'WEB-1 closed P1 alice [bug,ui] 2025-01-15 Login broken\nWEB-2 open P3 bob [] - Typo on home\nWEB-3 open P2 - [] - Ship it\nWEB-4 open P1 - [] 2000-02-29 Leap\nWEB-5 open P3 - [] - Empty due\nWEB-6 open P3 - [] 2024-02-29 Dash due'
expect_eq "$(get /p/WEB/issues.txt)" "$want" "issues.txt after edits"
expect_eq "$(get /p/WEB/4.txt)" 'WEB-4 open P1 - [] 2000-02-29 Leap' "issue txt"
expect_eq "$(get /p/WEB/1/activity.txt)" $'created\nstatus closed\ncommented bob' "edit logs no activity"

# JSON and CSV.
expect_eq "$(get /p/WEB/1.json)" '{"key":"WEB-1","project":"WEB","number":1,"title":"Login broken","body":"Steps: **click** `login`","status":"closed","priority":1,"assignee":"alice","labels":["bug","ui"],"due":"2025-01-15","comments":[{"author":"bob","body":"Fixed in <b>v2</b>"}],"activity":[{"kind":"created","detail":""},{"kind":"status","detail":"closed"},{"kind":"commented","detail":"bob"}]}' "json"
expect_has "$(get /p/WEB/3.json)" '"labels":[],"due":null,"comments":[]' "json null due"
csv="$(get /p/WEB/issues.csv)"
expect_eq "$(head -1 <<<"$csv")" 'key,title,status,priority,assignee,labels,due,body' "csv header"
expect_has "$csv" 'WEB-1,Login broken,closed,1,alice,bug;ui,2025-01-15,Steps: **click** `login`' "csv due"
expect_has "$csv" 'WEB-2,Typo on home,open,3,bob,,,50% of the text' "csv no due"

# HTML.
page="$(get /p/WEB)"
expect_has "$page" '<th>Labels</th><th>Due</th></tr>' "table header"
expect_has "$page" '<td>bug, ui</td><td>2025-01-15</td></tr>' "row due"
expect_has "$page" '<tr><td><a href="/p/WEB/2">WEB-2</a></td><td>Typo on home</td><td>open</td><td>P3</td><td>bob</td><td>-</td><td>-</td></tr>' "row no due"
expect_has "$page" '<input name="labels" placeholder="labels"> <input name="due" placeholder="YYYY-MM-DD"> <textarea name="body"></textarea>' "new issue form"
expect_has "$page" '<input name="q" value="" placeholder="search"> <input name="due_before" value="" placeholder="due before"> <button>filter</button>' "filter form"
page="$(get /p/WEB/1)"
expect_has "$page" '| Labels: bug, ui | Due: 2025-01-15</p>' "issue meta"
expect_has "$page" '<input name="priority" value="1"> <input name="due" value="2025-01-15"> <textarea name="body">' "edit form"
page="$(get /p/WEB/2)"
expect_has "$page" '| Labels: - | Due: -</p>' "issue meta no due"
expect_has "$page" '<input name="due" value=""> <textarea name="body">' "edit form no due"

# Filter.
expect_eq "$(get '/p/WEB/issues.txt?due_before=2025-02-01')" $'WEB-1 closed P1 alice [bug,ui] 2025-01-15 Login broken\nWEB-4 open P1 - [] 2000-02-29 Leap\nWEB-6 open P3 - [] 2024-02-29 Dash due' "due_before"
expect_eq "$(get '/p/WEB/issues.txt?due_before=2025-01-15')" $'WEB-4 open P1 - [] 2000-02-29 Leap\nWEB-6 open P3 - [] 2024-02-29 Dash due' "strictly before"
expect_eq "$(get '/p/WEB/issues.txt?status=closed&due_before=2025-02-01')" 'WEB-1 closed P1 alice [bug,ui] 2025-01-15 Login broken' "combined"
expect_eq "$(get '/p/WEB/issues.txt?due_before=0001-01-01')" '' "nothing before year 1"
expect_eq "$(get '/p/WEB/issues.txt?due_before=')" "$want" "empty due_before"
expect_err 400 "bad due date" '/p/WEB/issues.txt?due_before=2025-02-30'
expect_err 400 "bad due date" '/p/WEB?due_before=x'
expect_err 400 "bad status" '/p/WEB/issues.txt?status=done&due_before=x'
expect_err 400 "bad priority" '/p/WEB/issues.txt?priority=7&due_before=x'
expect_err 400 "bad page" '/p/WEB/issues.txt?page=0&due_before=x'
page="$(get '/p/WEB?due_before=2025-02-01')"
expect_has "$page" '<input name="due_before" value="2025-02-01" placeholder="due before">' "filter form value"
expect_has "$page" 'Page 1 of 1 (3 issues)' "filtered count"

# Pagination with the filter: 20 more issues due 2020-01-01 (WEB-7 .. WEB-26).
for i in $(seq 20); do post /p/WEB/issues "title=n+$i&due=2020-01-01" >/dev/null; done
page="$(get '/p/WEB?due_before=2021-01-01')"
expect_has "$page" 'Page 1 of 2 (21 issues)' "pager count"
expect_has "$page" 'href="/p/WEB?due_before=2021-01-01&amp;page=2">next</a>' "pager link"
expect_eq "$(get '/p/WEB/issues.txt?status=open&due_before=2021-01-01&page=2')" 'WEB-26 open P3 - [] 2020-01-01 n 20' "page 2"
expect_has "$(get '/p/WEB?status=open&due_before=2021-01-01&page=2')" 'href="/p/WEB?status=open&amp;due_before=2021-01-01&amp;page=1">prev</a>' "prev link"

# Persistence.
bb_restart
expect_eq "$(get /p/WEB/1.txt)" 'WEB-1 closed P1 alice [bug,ui] 2025-01-15 Login broken' "after restart"
expect_eq "$(get /p/WEB/3.txt)" 'WEB-3 open P2 - [] - Ship it' "after restart, removed"
expect_eq "$(get /p/WEB/6.txt)" 'WEB-6 open P3 - [] 2024-02-29 Dash due' "after restart, edited"
expect_has "$(get /p/WEB/1.json)" '"labels":["bug","ui"],"due":"2025-01-15","comments"' "json after restart"
bb_unit
echo PASS
