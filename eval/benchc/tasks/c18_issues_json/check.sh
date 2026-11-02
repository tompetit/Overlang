#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
bb_baseline
# json PATH WANT: exact body (minus the final newline), and the body ends in exactly one newline.
json() {
  expect_eq "$(ctype "$1")" "application/json" "$1 content type"
  expect_eq "$(get "$1")" "$2" "$1"
  expect_eq "$(get "$1" | tail -c 1 | od -An -c | tr -d ' ')" '\n' "$1 ends with a newline"
}
o() { echo "{\"key\":\"WEB-$1\",\"project\":\"WEB\",\"number\":$1,\"title\":\"bulk item $2\",\"body\":\"\",\"status\":\"open\",\"priority\":3,\"assignee\":null,\"labels\":[]}"; }
post /projects 'key=EMPTY&name=Nothing' >/dev/null
json /p/EMPTY/issues.json '{"project":"EMPTY","total":0,"page":1,"pages":1,"per_page":20,"prev":null,"next":null,"issues":[]}'
post /projects 'key=WEB&name=Web' >/dev/null
post /p/WEB/labels name=bug >/dev/null; post /p/WEB/labels name=ui >/dev/null
post /p/WEB/issues 'title=Say+%22hi%22&body=line1%0D%0Aline2%09tab&priority=2&assignee=alice&labels=ui,bug' >/dev/null
post /p/WEB/issues 'title=caf%C3%A9+%2F+na%C3%AFve&body=x%01y' >/dev/null
for i in $(seq 1 21); do post /p/WEB/issues "title=bulk+item+$i" >/dev/null; done
o1='{"key":"WEB-1","project":"WEB","number":1,"title":"Say \"hi\"","body":"line1\nline2\ttab","status":"open","priority":2,"assignee":"alice","labels":["bug","ui"]}'
o2='{"key":"WEB-2","project":"WEB","number":2,"title":"café / naïve","body":"x\u0001y","status":"open","priority":3,"assignee":null,"labels":[]}'
# existing routes unchanged
expect_eq "$(get /p/WEB/1.json)" "${o1%\}},\"comments\":[],\"activity\":[{\"kind\":\"created\",\"detail\":\"\"}]}" "issue json"
expect_eq "$(get /p/WEB/issues.txt | wc -l | tr -d ' ')" 20 "issues.txt page"
expect_eq "$(get '/p/WEB/issues.txt?page=2')" $'WEB-21 open P3 - [] bulk item 19\nWEB-22 open P3 - [] bulk item 20\nWEB-23 open P3 - [] bulk item 21' "issues.txt page 2"
# page 1
p1="$(get /p/WEB/issues.json)"
head1="{\"project\":\"WEB\",\"total\":23,\"page\":1,\"pages\":2,\"per_page\":20,\"prev\":null,\"next\":\"/p/WEB/issues.json?page=2\",\"issues\":[$o1,$o2,$(o 3 1),"
[[ "$p1" == "$head1"* ]] || fail "page 1: want prefix [$head1] got [${p1:0:600}]"
[[ "$p1" == *",$(o 20 18)]}" ]] || fail "page 1: want suffix [$(o 20 18)]}] got [${p1: -300}]"
expect_eq "$(grep -o '"key":' <<<"$p1" | wc -l | tr -d ' ')" 20 "page 1 issue count"
# page 2
want2="{\"project\":\"WEB\",\"total\":23,\"page\":2,\"pages\":2,\"per_page\":20,\"prev\":\"/p/WEB/issues.json?page=1\",\"next\":null,\"issues\":[$(o 21 19),$(o 22 20),$(o 23 21)]}"
json '/p/WEB/issues.json?page=2' "$want2"
# filters: links follow the canonical parameter order and encoding
json '/p/WEB/issues.json?q=bulk+item&priority=3&status=open&page=2' "{\"project\":\"WEB\",\"total\":21,\"page\":2,\"pages\":2,\"per_page\":20,\"prev\":\"/p/WEB/issues.json?status=open&priority=3&q=bulk%20item&page=1\",\"next\":null,\"issues\":[$(o 23 21)]}"
f1="$(get '/p/WEB/issues.json?q=bulk+item&priority=3&status=open')"
[[ "$f1" == '{"project":"WEB","total":21,"page":1,"pages":2,"per_page":20,"prev":null,"next":"/p/WEB/issues.json?status=open&priority=3&q=bulk%20item&page=2","issues":[{"key":"WEB-3",'* ]] || fail "filtered page 1: got [${f1:0:400}]"
json '/p/WEB/issues.json?assignee=alice' "{\"project\":\"WEB\",\"total\":1,\"page\":1,\"pages\":1,\"per_page\":20,\"prev\":null,\"next\":null,\"issues\":[$o1]}"
json '/p/WEB/issues.json?label=bug&page=3' '{"project":"WEB","total":1,"page":3,"pages":1,"per_page":20,"prev":"/p/WEB/issues.json?label=bug&page=2","next":null,"issues":[]}'
json '/p/WEB/issues.json?q=%C3%A9' "{\"project\":\"WEB\",\"total\":1,\"page\":1,\"pages\":1,\"per_page\":20,\"prev\":null,\"next\":null,\"issues\":[$o2]}"
# errors
expect_err 400 "bad page" '/p/WEB/issues.json?page=0'
expect_err 400 "bad status" '/p/WEB/issues.json?status=done'
expect_err 400 "bad priority" '/p/WEB/issues.json?priority=9'
expect_err 404 "no such project: NOPE" '/p/NOPE/issues.json'
expect_err 404 "not found" /p/WEB/issues.json 'title=x'
expect_eq "$(get /p/WEB/issues.txt | wc -l | tr -d ' ')" 20 "POST did not create an issue"
bb_restart
json '/p/WEB/issues.json?page=2' "$want2"
json /p/BASE/issues.json '{"project":"BASE","total":2,"page":1,"pages":1,"per_page":20,"prev":null,"next":null,"issues":[{"key":"BASE-1","project":"BASE","number":1,"title":"Login fails","body":"Steps: **click**","status":"closed","priority":1,"assignee":"alice","labels":["bug","ui"]},{"key":"BASE-2","project":"BASE","number":2,"title":"Typo on home","body":"50% of the text","status":"open","priority":3,"assignee":"bob","labels":[]}]}'
bb_unit
echo PASS
