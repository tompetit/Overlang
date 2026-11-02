#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
bb_baseline
TEXT='text/plain; charset=utf-8'
# resp METHOD PATH: prints "status content-type"; the exact body is in $RUN/body.
resp() {
  local extra=(); [ "$1" = POST ] && extra=(--data-raw x=1)
  curl -s --max-time 10 -X "$1" "${extra[@]}" -o "$RUN/body" -w '%{http_code} %{content_type}' "$URL$2"
}
# expect_resp STATUS CTYPE BODY METHOD PATH: exact status, content type and body bytes.
expect_resp() {
  expect_eq "$(resp "$4" "$5")" "$1 $2" "$4 $5 status and content type"
  printf '%s' "$3" >"$RUN/want"
  cmp -s "$RUN/body" "$RUN/want" || fail "$4 $5 body: want [$3] got [$(cat "$RUN/body")]"
}
jerr() { expect_resp "$1" application/json "{\"error\":\"$2\"}"$'\n' "${3:-GET}" "$4"; }
terr() { expect_resp "$1" "$TEXT" "$2"$'\n' "${3:-GET}" "$4"; }

jerr 404 "no such issue: BASE-9" GET /p/BASE/9.json
jerr 404 "no such issue: BASE-9" GET /p/BASE/9.json/
jerr 404 "no such issue: BASE-9" GET '/p/BASE/9.json?x=1'
jerr 404 "no such project: NOPE" GET /p/NOPE/1.json
jerr 404 "no such project: A%22B" GET /p/A%22B/1.json
jerr 404 "no such project: BASE.json" GET /p/BASE.json
jerr 404 "not found" GET /nope.json
jerr 404 "not found" GET /.json
jerr 404 "not found" GET /p/BASE/0.json
jerr 404 "not found" GET /p/BASE/issues.json
jerr 404 "not found" POST /p/BASE/1.json
jerr 404 "not found" DELETE /x/y.json

terr 404 "not found" GET /p/BASE/9.JSON
terr 404 "not found" GET /p/BASE/1.json/comments.txt
terr 404 "not found" GET /a.json/b
terr 404 "no such issue: BASE-9" GET '/p/BASE/9?f=.json'
terr 404 "no such issue: BASE-9" GET /p/BASE/9.txt
terr 404 "no such issue: BASE-9" GET /p/BASE/9/comments.txt
terr 404 "no such project: NOPE" GET /p/NOPE
terr 400 "bad status" GET '/p/BASE/issues.txt?status=done'
terr 404 "not found" DELETE /
expect_eq "$(curl -s --max-time 10 -o /dev/null -w '%{content_type}' -d title= "$URL/p/BASE/issues")" "$TEXT" "422 content type"
expect_err 422 "title required" /p/BASE/issues title=

# successes unchanged
J1='{"key":"BASE-1","project":"BASE","number":1,"title":"Login fails","body":"Steps: **click**","status":"closed","priority":1,"assignee":"alice","labels":["bug","ui"],"comments":[{"author":"bob","body":"Fixed in <b>v2</b>"}],"activity":[{"kind":"created","detail":""},{"kind":"status","detail":"closed"},{"kind":"commented","detail":"bob"}]}'
expect_resp 200 application/json "$J1"$'\n' GET /p/BASE/1.json
expect_resp 200 application/json "$J1"$'\n' GET /p/BASE/1.json/
expect_resp 200 "$TEXT" $'ok\n' GET /health
expect_code 200 /p/BASE/1

bb_restart
jerr 404 "no such issue: BASE-3" GET /p/BASE/3.json
terr 404 "no such issue: BASE-3" GET /p/BASE/3
expect_has "$(get /p/BASE/2.json)" '"assignee":"bob"' "data after restart"
bb_unit
echo PASS
