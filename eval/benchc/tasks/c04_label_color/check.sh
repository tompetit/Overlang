#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
# Old behaviour (labels.txt changes, so not bb_baseline).
expect_eq "$(get /health)" "ok" "health"
post /users name=alice >/dev/null; post /users name=bob >/dev/null
expect_eq "$(get /users.txt)" $'1 alice\n2 bob' "users.txt"
expect_code 200 /projects 'key=WEB&name=Web+site'
expect_code 200 /projects 'key=API&name=Api'
expect_code 200 /p/WEB/labels name=bug
expect_code 200 /p/WEB/labels name=ui
expect_err 422 "label exists" /p/WEB/labels name=bug
expect_err 422 "bad label" /p/WEB/labels name=Bug
expect_err 422 "name required" /p/WEB/labels name=
post /p/WEB/issues 'title=Login+fails&body=Steps%3A+**click**+%60login%60&priority=1&assignee=alice&labels=bug,ui' >/dev/null
post /p/WEB/issues 'title=Typo+on+home&body=50%25+of+the+text' >/dev/null
post /p/WEB/1/status status=closed >/dev/null
post /p/WEB/1/comments 'author=bob&body=Fixed+in+%3Cb%3Ev2%3C%2Fb%3E' >/dev/null
expect_eq "$(get /p/WEB/issues.txt)" $'WEB-1 closed P1 alice [bug,ui] Login fails\nWEB-2 open P3 - [] Typo on home' "issues.txt"
expect_eq "$(get /p/WEB/1/activity.txt)" $'created\nstatus closed\ncommented bob' "activity"
expect_err 422 "no such label: nope" /p/WEB/1/labels label=nope
expect_err 404 "no such project: NOPE" /p/NOPE
expect_err 404 "not found" /nope

# Default colour.
expect_eq "$(get /p/WEB/labels.txt)" $'bug #888888\nui #888888' "labels.txt default"

# Create with colours: lowercased, trimmed, empty = default.
expect_code 200 /p/WEB/labels 'name=crash&color=%23FF0000'
expect_code 200 /p/WEB/labels 'name=docs&color=+%2300aA11+'
expect_code 200 /p/WEB/labels 'name=empty&color='
want=$'bug #888888\ncrash #ff0000\ndocs #00aa11\nempty #888888\nui #888888'
expect_eq "$(get /p/WEB/labels.txt)" "$want" "labels.txt colours"
for c in red %23abc %23abcdeg %23abcdef0 00ff00 %23%2300ff00 '%23+00ff00' %2300ff0 '%2300FF00x'; do
  expect_err 422 "bad color" /p/WEB/labels "name=x1&color=$c"
done
expect_err 422 "name required" /p/WEB/labels 'name=&color=nope'
expect_err 422 "bad label" /p/WEB/labels 'name=Bad&color=nope'
expect_err 422 "label exists" /p/WEB/labels 'name=bug&color=nope'
expect_eq "$(get /p/WEB/labels.txt)" "$want" "nothing stored on errors"

# Same name in another project; change colours per project.
expect_code 200 /p/API/labels name=bug
expect_code 200 /p/WEB/labels/bug/color 'color=%23AbCdEf'
expect_eq "$(get /p/API/labels.txt)" 'bug #888888' "other project untouched"
page="$(post /p/API/labels/bug/color 'color=%23111111')"
expect_has "$page" '<h1>API: Api</h1>' "answers project page"
expect_eq "$(get /p/WEB/labels.txt)" $'bug #abcdef\ncrash #ff0000\ndocs #00aa11\nempty #888888\nui #888888' "WEB after colour changes"
expect_eq "$(get /p/API/labels.txt)" 'bug #111111' "API after colour change"
expect_err 404 "no such label: nope" /p/WEB/labels/nope/color 'color=%23000000'
expect_err 404 "no such label: nope" /p/WEB/labels/nope/color 'color='
expect_err 404 "no such label: crash" /p/API/labels/crash/color 'color=%23000000'
expect_err 404 "no such project: NOPE" /p/NOPE/labels/bug/color 'color='
expect_err 422 "color required" /p/WEB/labels/bug/color 'color='
expect_err 422 "color required" /p/WEB/labels/bug/color 'x=1'
expect_err 422 "bad color" /p/WEB/labels/bug/color 'color=blue'
expect_err 404 "not found" /p/WEB/labels/bug/color
expect_eq "$(get /p/WEB/labels.txt)" $'bug #abcdef\ncrash #ff0000\ndocs #00aa11\nempty #888888\nui #888888' "unchanged on errors"

# Chips follow the label order (by name), not the colour order.
expect_code 200 /p/WEB/1/labels label=crash
expect_code 200 /p/API/issues 'title=Api+bug&labels=bug'
B='<span class="label" style="background:#abcdef">bug</span>'
C='<span class="label" style="background:#ff0000">crash</span>'
D='<span class="label" style="background:#00aa11">docs</span>'
E='<span class="label" style="background:#888888">empty</span>'
U='<span class="label" style="background:#888888">ui</span>'
page="$(get /p/WEB)"
expect_has "$page" "<td>P1</td><td>alice</td><td>$B, $C, $U</td></tr>" "row chips"
expect_has "$page" '<td>P3</td><td>-</td><td>-</td></tr>' "row without labels"
expect_has "$page" "<h2>Labels</h2><p class=\"labels\">$B, $C, $D, $E, $U</p>" "labels paragraph"
expect_has "$page" '<input name="name" placeholder="label"> <input name="color" placeholder="#rrggbb"> <button>add label</button>' "label form"
expect_has "$(get /p/WEB/1)" "| Labels: $B, $C, $U</p>" "issue page chips"
expect_has "$(get /p/WEB/2)" '| Labels: -</p>' "issue page no labels"
expect_has "$(get /p/API/1)" '| Labels: <span class="label" style="background:#111111">bug</span></p>' "own project's colour"
expect_eq "$(get /p/WEB/1.json)" '{"key":"WEB-1","project":"WEB","number":1,"title":"Login fails","body":"Steps: **click** `login`","status":"closed","priority":1,"assignee":"alice","labels":["bug","crash","ui"],"label_colors":["#abcdef","#ff0000","#888888"],"comments":[{"author":"bob","body":"Fixed in <b>v2</b>"}],"activity":[{"kind":"created","detail":""},{"kind":"status","detail":"closed"},{"kind":"commented","detail":"bob"}]}' "json"
expect_has "$(get /p/WEB/2.json)" '"labels":[],"label_colors":[],"comments"' "json no labels"
expect_has "$(get /p/API/1.json)" '"labels":["bug"],"label_colors":["#111111"],' "json own project"

# Names-only formats and the filter are unchanged.
expect_eq "$(get /p/WEB/1.txt)" 'WEB-1 closed P1 alice [bug,crash,ui] Login fails' "issue_line"
expect_eq "$(get '/p/WEB/issues.txt?label=crash')" 'WEB-1 closed P1 alice [bug,crash,ui] Login fails' "label filter"
expect_has "$(get /p/WEB/issues.csv)" 'WEB-1,Login fails,closed,1,alice,bug;crash;ui,Steps: **click** `login`' "csv"
expect_code 200 /p/WEB/1/labels/remove label=bug
expect_has "$(get /p/WEB/1.json)" '"labels":["crash","ui"],"label_colors":["#ff0000","#888888"],' "after remove"

# Persistence.
bb_restart
expect_eq "$(get /p/WEB/labels.txt)" $'bug #abcdef\ncrash #ff0000\ndocs #00aa11\nempty #888888\nui #888888' "labels.txt after restart"
expect_eq "$(get /p/API/labels.txt)" 'bug #111111' "API after restart"
expect_has "$(get /p/WEB/1)" "| Labels: $C, $U</p>" "chips after restart"
bb_unit
echo PASS
