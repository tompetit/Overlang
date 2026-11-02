#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
# expect_body PATH S: the exact body of GET PATH is S followed by one newline.
expect_body() { getf "$1" "$RUN/got"; printf '%s\n' "$2" >"$RUN/want"; cmp -s "$RUN/got" "$RUN/want" || fail "$1: want [$(cat "$RUN/want")] got [$(od -c "$RUN/got" | head -c 1500)]"; }
bb_baseline
expect_has "$(get /p/BASE/issues.csv)" 'BASE-2,Typo on home,open,3,bob,,50% of the text' "csv"

# Export of the baseline project, built from the per-issue JSON.
j1="$(get /p/BASE/1.json)"; j2="$(get /p/BASE/2.json)"
expect_body /p/BASE/export.json "{\"key\":\"BASE\",\"name\":\"Base line\",\"labels\":[\"bug\",\"ui\"],\"users\":[\"alice\",\"bob\"],\"issues\":[$j1,$j2]}"
expect_eq "$(ctype /p/BASE/export.json)" "application/json" "content type"
expect_eq "$(get /p/BASE/export.json)" '{"key":"BASE","name":"Base line","labels":["bug","ui"],"users":["alice","bob"],"issues":[{"key":"BASE-1","project":"BASE","number":1,"title":"Login fails","body":"Steps: **click**","status":"closed","priority":1,"assignee":"alice","labels":["bug","ui"],"comments":[{"author":"bob","body":"Fixed in <b>v2</b>"}],"activity":[{"kind":"created","detail":""},{"kind":"status","detail":"closed"},{"kind":"commented","detail":"bob"}]},{"key":"BASE-2","project":"BASE","number":2,"title":"Typo on home","body":"50% of the text","status":"open","priority":3,"assignee":"bob","labels":[],"comments":[],"activity":[{"kind":"created","detail":""},{"kind":"assigned","detail":"bob"}]}]}' "literal export"

# An empty project with a name that needs escaping.
post /projects "key=API&name=$(enc 'Say "hi" \ é/x')" >/dev/null
expect_body /p/API/export.json '{"key":"API","name":"Say \"hi\" \\ é/x","labels":[],"users":[],"issues":[]}'

# Users: current assignees and comment authors only, each once, code-point order.
post /users name=carol >/dev/null; post /users name=dave >/dev/null; post /users name=Zed >/dev/null; post /users name=erin >/dev/null
post /p/API/labels name=docs >/dev/null; post /p/API/labels name=a-1 >/dev/null
post /p/API/issues "title=One&body=$(enc $'x\ty\x01z\r\nw')&assignee=dave&labels=docs" >/dev/null
post /p/API/issues 'title=Two&assignee=Zed' >/dev/null
post /p/API/issues 'title=Three' >/dev/null
post /p/API/1/comments 'author=carol&body=one' >/dev/null
post /p/API/3/comments 'author=carol&body=two' >/dev/null
post /p/API/3/comments 'author=alice&body=three' >/dev/null
post /p/API/1/assign assignee= >/dev/null
post /p/API/2/assign assignee=erin >/dev/null
post /p/API/2/assign assignee=Zed >/dev/null
post /p/API/3/status status=closed >/dev/null
post /p/BASE/2/assign assignee=dave >/dev/null
post /p/BASE/2/assign assignee=bob >/dev/null
a1="$(get /p/API/1.json)"; a2="$(get /p/API/2.json)"; a3="$(get /p/API/3.json)"
expect_has "$a1" '"body":"x\ty\u0001z\nw"' "issue json escaping"
want_api="{\"key\":\"API\",\"name\":\"Say \\\"hi\\\" \\\\ é/x\",\"labels\":[\"a-1\",\"docs\"],\"users\":[\"Zed\",\"alice\",\"carol\"],\"issues\":[$a1,$a2,$a3]}"
expect_body /p/API/export.json "$want_api"
j2="$(get /p/BASE/2.json)"
want_base="{\"key\":\"BASE\",\"name\":\"Base line\",\"labels\":[\"bug\",\"ui\"],\"users\":[\"alice\",\"bob\"],\"issues\":[$j1,$j2]}"
expect_eq "$(get /p/BASE/export.json)" "$want_base" "BASE export after reassigning"
expect_eq "$(get '/p/API/export.json?status=open&page=2')" "$want_api" "query ignored"
expect_eq "$(get /p/API/export.json/)" "$want_api" "trailing slash"

# No pagination: 25 issues.
post /projects 'key=BIG&name=Big' >/dev/null
for i in $(seq 25); do post /p/BIG/issues "title=n+$i&priority=$((1 + i % 4))" >/dev/null; done
post /p/BIG/7/status status=closed >/dev/null
getf /p/BIG/export.json "$RUN/big"
python3 - "$RUN/big" "$(get /p/BIG/7.json)" "$(get /p/BIG/25.json)" <<'PY' || fail "BIG export: $(head -c 400 "$RUN/big")"
import json, sys
raw = open(sys.argv[1], encoding="utf-8").read()
assert raw.endswith("}\n") and raw.count("\n") == 1, "one line"
d = json.loads(raw)
assert list(d) == ["key", "name", "labels", "users", "issues"], list(d)
assert [i["number"] for i in d["issues"]] == list(range(1, 26)), [i["number"] for i in d["issues"]]
assert d["users"] == [] and d["labels"] == []
assert "," + sys.argv[2] + "," in raw, "issue 7 as its own JSON"
assert raw.rstrip("\n").endswith(sys.argv[3] + "]}"), "last issue"
PY

# Errors and other routes.
expect_err 404 "no such project: NOPE" /p/NOPE/export.json
expect_err 404 "no such project: base" /p/base/export.json
expect_err 404 "not found" /p/BASE/export.json x=1
expect_err 404 "not found" /p/BASE/export
expect_err 404 "not found" /p/BASE/export.csv
expect_err 404 "not found" /p/BASE/export.json/x
expect_err 404 "not found" /export.json
expect_eq "$(get /p/BASE/2.txt)" 'BASE-2 open P3 bob [] Typo on home' "issue txt"
expect_eq "$(get /p/API/issues.txt)" $'API-1 open P3 - [docs] One\nAPI-2 open P3 Zed [] Two\nAPI-3 closed P3 - [] Three' "API issues.txt"

# Persistence.
bb_restart
expect_eq "$(get /p/API/export.json)" "$want_api" "API export after restart"
expect_eq "$(get /p/BASE/export.json)" "$want_base" "BASE export after restart"
bb_unit
echo PASS
