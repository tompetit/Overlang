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
# The normal page with its POST forms and the New issue heading removed and the nav marker added.
ro_of() { python3 -c '
import re, sys
s = sys.stdin.read()
s = re.sub(r"<form method=\"post\"[^>]*>.*?</form>", "", s, flags=re.S).replace("<h2>New issue</h2>", "")
sys.stdout.write(s.replace("<a href=\"/users\">Users</a></nav>", "<a href=\"/users\">Users</a> <span class=\"mode\">read-only</span></nav>"))'; }
sum() { sha256sum "$RUN/tracker.db" | cut -d" " -f1; }

# Old behaviour, without the flag.
bb_baseline
post /p/BASE/2/comments 'author=alice&body=See+**this**' >/dev/null
post /p/BASE/2/labels label=ui >/dev/null
users_n="$(get /users)"; home_n="$(get /)"; proj_n="$(get /p/BASE)"; projf_n="$(get '/p/BASE?status=open&q=typo')"
iss1_n="$(get /p/BASE/1)"; iss2_n="$(get /p/BASE/2)"
txt_n="$(get /p/BASE/issues.txt)"; json_n="$(get /p/BASE/1.json)"; csv_n="$(get /p/BASE/issues.csv)"
expect_lacks "$users_n" 'read-only' "no marker without the flag"
expect_has "$users_n" '<form method="post" action="/users">' "users form without the flag"
before="$(sum)"
bb_stop

# Read-only, flag first.
start_with --read-only tracker.db @PORT@
expect_eq "$(grep -cx 'read-only mode' "$RUN/server.log")" 1 "startup line"
expect_eq "$(grep -A1 -x 'read-only mode' "$RUN/server.log" | tail -1)" "listening on http://127.0.0.1:$PORT" "startup line before listening"
expect_eq "$(get /health)" "ok read-only" "health"
expect_eq "$(ctype /health)" "text/plain; charset=utf-8" "health content type"
for p in "/users name=carol" "/users name=" "/users name=a+b" "/projects key=API&name=Api" "/projects key=&name=" \
         "/p/BASE/issues title=New" "/p/BASE/issues title=" "/p/NOPE/issues title=x" "/p/BASE/labels name=docs" \
         "/p/BASE/1/edit title=Changed" "/p/BASE/1/status status=open" "/p/BASE/1/status status=done" \
         "/p/BASE/2/assign assignee=" "/p/BASE/2/assign assignee=zed" "/p/BASE/1/comments author=bob&body=hi" \
         "/p/BASE/1/labels label=bug" "/p/BASE/1/labels/remove label=bug" "/p/BASE/99/status status=closed" \
         "/p/NOPE/1/comments author=x&body=y" "/p/BASE/2/labels/remove label=nope"; do
  expect_err 403 "read-only mode" "${p%% *}" "${p#* }"
done
expect_eq "$(curl -s -o /dev/null -w '%{content_type}' -d name=carol "$URL/users")" "text/plain; charset=utf-8" "403 content type"
expect_err 404 "not found" /users.txt name=x
expect_err 404 "not found" /p/BASE/abc/status status=closed
expect_err 404 "not found" /p/BASE/0/status status=closed
expect_err 404 "not found" /health x=1
expect_err 404 "not found" /nope x=1
expect_err 404 "not found" / x=1
expect_eq "$(curl -s -o /dev/null -w '%{http_code}' -X DELETE "$URL/")" 404 "DELETE /"
expect_err 404 "not found" /nope
expect_err 404 "no such project: NOPE" /p/NOPE
expect_err 404 "no such issue: BASE-99" /p/BASE/99
expect_err 400 "bad status" '/p/BASE?status=done'
# GET routes unchanged.
expect_eq "$(get /p/BASE/issues.txt)" "$txt_n" "issues.txt"
expect_eq "$(get /p/BASE/1.json)" "$json_n" "json"
expect_eq "$(get /p/BASE/issues.csv)" "$csv_n" "csv"
expect_eq "$(get /users.txt)" $'1 alice\n2 bob' "users.txt"
expect_eq "$(get /p/BASE/2/comments.txt)" 'alice: See **this**' "comments.txt"
# Pages without POST forms.
expect_eq "$(get /users)" '<!doctype html><html><head><meta charset="utf-8"><title>Users</title></head><body><nav><a href="/">Tracker</a> <a href="/users">Users</a> <span class="mode">read-only</span></nav><h1>Users</h1><ul class="users"><li>alice</li><li>bob</li></ul></body></html>' "users page"
expect_eq "$(get /)" "$(ro_of <<<"$home_n")" "home page"
expect_eq "$(get /p/BASE)" "$(ro_of <<<"$proj_n")" "project page"
expect_eq "$(get '/p/BASE?status=open&q=typo')" "$(ro_of <<<"$projf_n")" "filtered project page"
expect_eq "$(get /p/BASE/1)" "$(ro_of <<<"$iss1_n")" "issue page 1"
expect_eq "$(get /p/BASE/2)" "$(ro_of <<<"$iss2_n")" "issue page 2"
page="$(get /p/BASE)"
expect_has "$page" '<form method="get" action="/p/BASE">' "filter form kept"
expect_lacks "$page" 'method="post"' "project page forms"
expect_lacks "$page" 'New issue' "project page heading"
page="$(get /p/BASE/1)"
expect_has "$page" '<h2>Comments</h2><ul class="comments"><li><b>bob</b> ' "comments kept"
expect_has "$page" '<h2>Activity</h2><ul class="activity"><li>created</li>' "activity kept"
expect_lacks "$page" '<form' "issue page forms"
bb_stop
expect_eq "$(sum)" "$before" "database file unchanged by the read-only server"

# Read-only, flag last.
start_with tracker.db @PORT@ --read-only
expect_eq "$(get /health)" "ok read-only" "health, flag last"
expect_err 403 "read-only mode" /p/BASE/issues title=x
expect_eq "$(get /p/BASE/issues.txt)" "$txt_n" "issues.txt, flag last"
bb_stop
expect_eq "$(sum)" "$before" "database file unchanged, flag last"

# A missing database is not created.
rc=0; (cd "$RUN" && if [ "$LANG_" = python ]; then exec timeout 10 python3 -B "$APP/main.py" --read-only missing.db 1; else exec timeout 10 ./app --read-only missing.db 1; fi) >"$RUN/missing.log" 2>&1 || rc=$?
expect_eq "$rc" 1 "exit status for a missing database"
expect_has "$(cat "$RUN/missing.log")" "cannot open missing.db" "message for a missing database"
[ ! -e "$RUN/missing.db" ] || fail "read-only mode created missing.db"

# Back to normal: writes work, no marker, data as it was.
bb_start
expect_eq "$(get /health)" "ok" "health without the flag"
expect_eq "$(get /users)" "$users_n" "users page without the flag"
expect_eq "$(get /p/BASE/1)" "$iss1_n" "issue page without the flag"
expect_has "$(post /p/BASE/issues 'title=After+restart')" '<h1>BASE-3: After restart</h1>' "write after read-only"
expect_code 200 /p/BASE/3/status status=closed
expect_eq "$(get /p/BASE/3.txt)" 'BASE-3 closed P3 - [] After restart' "issue 3"
bb_unit
echo PASS
