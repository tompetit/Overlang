#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
bb_baseline
# ver ISSUE_PATH: the version in the issue's JSON
ver() { get "$1.json" | sed -n 's/.*"number":[0-9]*,"version":\([0-9]*\),"title".*/\1/p'; }
expect_has "$(get /p/BASE/1.json)" '{"key":"BASE-1","project":"BASE","number":1,"version":3,"title":"Login fails",' "BASE-1 (status + comment)"
expect_eq "$(ver /p/BASE/2)" 2 "BASE-2 (assign)"
post /projects 'key=WEB&name=Web' >/dev/null
post /p/WEB/labels name=bug >/dev/null
post /p/WEB/issues 'title=Crash&priority=2' >/dev/null
expect_eq "$(get /p/WEB/1.json)" '{"key":"WEB-1","project":"WEB","number":1,"version":1,"title":"Crash","body":"","status":"open","priority":2,"assignee":null,"labels":[],"comments":[],"activity":[{"kind":"created","detail":""}]}' "new issue json"
# status
expect_code 200 /p/WEB/1/status 'status=closed&version=1'
expect_eq "$(ver /p/WEB/1)" 2 "after status change"
expect_code 200 /p/WEB/1/status 'status=closed&version=2'
expect_eq "$(ver /p/WEB/1)" 2 "same status: no bump"
expect_err 409 "version conflict: current is 2" /p/WEB/1/status 'status=open&version=1'
expect_eq "$(get /p/WEB/1.txt)" 'WEB-1 closed P2 - [] Crash' "409 changed nothing"
expect_eq "$(curl -s --max-time 10 -o /dev/null -w '%{content_type}' -d 'status=open&version=1' "$URL/p/WEB/1/status")" "text/plain; charset=utf-8" "409 content type"
expect_err 409 "version conflict: current is 2" /p/WEB/1/status 'status=done&version=1'
expect_err 422 "bad status" /p/WEB/1/status 'status=done&version=2'
expect_err 400 "bad version" /p/WEB/1/status 'status=open&version=abc'
expect_err 400 "bad version" /p/WEB/1/status 'status=open&version=-1'
expect_err 400 "bad version" /p/WEB/1/status 'status=open&version=1234567890'
expect_err 409 "version conflict: current is 2" /p/WEB/1/status 'status=open&version=0'
expect_err 400 "bad version" /p/WEB/1/status 'status=done&version=x'
expect_err 404 "no such issue: WEB-9" /p/WEB/9/status 'status=open&version=x'
expect_err 404 "no such project: NOPE" /p/NOPE/1/edit 'title=x&version=1'
# edit
post /p/WEB/1/edit 'title=Crash' >/dev/null
expect_eq "$(ver /p/WEB/1)" 2 "edit to same values: no bump"
expect_code 200 /p/WEB/1/edit 'title=Crash+hard&version=2'
expect_eq "$(ver /p/WEB/1)" 3 "edit"
expect_err 422 "bad priority" /p/WEB/1/edit 'priority=9&version=3'
expect_err 409 "version conflict: current is 3" /p/WEB/1/edit 'priority=x&version=2'
expect_eq "$(ver /p/WEB/1)" 3 "failed edits: no bump"
# assign
expect_code 200 /p/WEB/1/assign 'assignee=alice&version=3'
post /p/WEB/1/assign 'assignee=alice' >/dev/null
expect_eq "$(ver /p/WEB/1)" 4 "assign once"
expect_err 422 "no such user: nobody" /p/WEB/1/assign 'assignee=nobody&version=4'
expect_err 409 "version conflict: current is 4" /p/WEB/1/assign 'assignee=&version=3'
expect_eq "$(get /p/WEB/1.txt)" 'WEB-1 closed P2 alice [] Crash hard' "assignee kept after 409"
# labels
expect_code 200 /p/WEB/1/labels 'label=bug&version=4'
post /p/WEB/1/labels 'label=bug' >/dev/null
expect_eq "$(ver /p/WEB/1)" 5 "label attached once"
expect_err 409 "version conflict: current is 5" /p/WEB/1/labels/remove 'label=bug&version=4'
expect_code 200 /p/WEB/1/labels/remove 'label=bug&version=5'
post /p/WEB/1/labels/remove 'label=bug' >/dev/null
expect_eq "$(ver /p/WEB/1)" 6 "label detached once"
expect_err 422 "no such label: nope" /p/WEB/1/labels 'label=nope&version=6'
# comments
expect_code 200 /p/WEB/1/comments 'author=bob&body=hi&version=6'
expect_err 409 "version conflict: current is 7" /p/WEB/1/comments 'author=bob&body=again&version=6'
expect_err 422 "author required" /p/WEB/1/comments 'author=&body=x&version=7'
expect_eq "$(get /p/WEB/1/comments.txt)" 'bob: hi' "comment not added on 409"
expect_eq "$(ver /p/WEB/1)" 7 "comment"
# html forms carry the version
page="$(get /p/WEB/1)"
for a in status assign labels labels/remove edit comments; do
  expect_has "$page" "<form method=\"post\" action=\"/p/WEB/1/$a\"><input type=\"hidden\" name=\"version\" value=\"7\">" "form $a"
done
expect_eq "$(grep -o 'name="version" value="7"' <<<"$page" | wc -l | tr -d ' ')" 6 "six version inputs"
# unchanged outputs
expect_eq "$(get /p/WEB/1/activity.txt)" $'created\nstatus closed\nassigned alice\ncommented bob' "activity unchanged"
expect_eq "$(get /p/WEB/issues.txt)" 'WEB-1 closed P2 alice [] Crash hard' "issues.txt unchanged"
expect_eq "$(head -n 1 <(get /p/WEB/issues.csv))" 'key,title,status,priority,assignee,labels,body' "csv header unchanged"
# a second issue has its own version
post /p/WEB/issues 'title=Other' >/dev/null
expect_err 409 "version conflict: current is 1" /p/WEB/2/status 'status=closed&version=7'
bb_restart
expect_eq "$(ver /p/WEB/1)" 7 "version persists"
expect_err 409 "version conflict: current is 7" /p/WEB/1/status 'status=open&version=6'
expect_code 200 /p/WEB/1/status 'status=open&version=7'
expect_has "$(get /p/WEB/1.json)" '"number":1,"version":8,"title":"Crash hard","body":"","status":"open"' "bump after restart"
expect_has "$(get /p/WEB/1)" '<input type="hidden" name="version" value="8">' "page after restart"
bb_unit
echo PASS
