#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
penc() { python3 -c 'import sys,urllib.parse as u;sys.stdout.write(u.quote_plus(eval(sys.argv[1])))' "$1"; }
ptxt() { python3 -c 'import sys;sys.stdout.buffer.write(eval(sys.argv[1]).encode())' "$1"; }

# regression (lib's bb_baseline expects `key=base` to be `bad key`, which this task changes)
expect_eq "$(get /health)" ok "health"
post /users name=alice >/dev/null; post /users name=bob >/dev/null
expect_eq "$(get /users.txt)" $'1 alice\n2 bob' "users.txt"
expect_err 422 "user exists" /users name=alice
post /projects 'key=WEB&name=Web+site' >/dev/null
expect_eq "$(get /projects.txt)" "WEB Web site" "projects.txt"
post /p/WEB/labels name=bug >/dev/null; post /p/WEB/labels name=ui >/dev/null
post /p/WEB/issues 'title=Login+fails&body=Steps%3A+**click**&priority=1&assignee=alice&labels=bug,ui' >/dev/null
post /p/WEB/issues 'title=Typo+on+home&body=50%25+of+the+text' >/dev/null
post /p/WEB/1/status status=closed >/dev/null
post /p/WEB/2/assign assignee=bob >/dev/null
post /p/WEB/1/comments 'author=bob&body=Fixed+in+%3Cb%3Ev2%3C%2Fb%3E' >/dev/null
expect_eq "$(get /p/WEB/issues.txt)" $'WEB-1 closed P1 alice [bug,ui] Login fails\nWEB-2 open P3 bob [] Typo on home' "issues.txt"
expect_eq "$(get /p/WEB/1/activity.txt)" $'created\nstatus closed\ncommented bob' "activity.txt"
expect_eq "$(get /p/WEB/1/comments.txt)" 'bob: Fixed in <b>v2</b>' "comments.txt"
expect_has "$(get /p/WEB/1)" '<strong>click</strong>' "markdown"
expect_eq "$(get '/p/WEB/issues.txt?status=open')" 'WEB-2 open P3 bob [] Typo on home' "status filter"
expect_err 404 "no such project: NOPE" /p/NOPE
expect_err 404 "no such issue: WEB-99" /p/WEB/99
expect_err 400 "bad page" '/p/WEB?page=abc'
expect_err 422 "title required" /p/WEB/issues title=
expect_err 422 "no such label: nope" /p/WEB/issues 'title=x&labels=nope'
expect_err 404 "not found" /nope
expect_err 404 "not found" /p/web/0
expect_code 404 /p/WEB/abc
expect_err 422 "key required" /projects 'key=+&name=x'
expect_err 422 "bad key" /projects 'key=W&name=x'
expect_err 422 "name required" /projects 'key=API&name=+'
expect_err 422 "project exists" /projects 'key=WEB&name=x'

# creating with any case
call_page="$(post /projects 'key=+api+&name=Public+API')"
expect_has "$call_page" '<h1>API: Public API</h1>' "created from lower case"
expect_eq "$(get /projects.txt)" $'API Public API\nWEB Web site' "projects.txt"
expect_err 422 "project exists" /projects 'key=web&name=Again'
expect_err 422 "project exists" /projects 'key=aPi&name=Again'
expect_err 422 "bad key" /projects 'key=w&name=x'
expect_err 422 "bad key" /projects 'key=3web&name=x'
expect_err 422 "bad key" /projects 'key=w-b&name=x'
expect_err 422 "bad key" /projects "key=$(penc '"éb"')&name=x"
expect_err 422 "bad key" /projects 'key=abcdefghijk&name=x'
expect_code 200 /projects 'key=abcdefghij&name=Ten'

# name length, and the order of checks
expect_err 422 "name too long" /projects "key=OPS&name=$(penc '"é"*61')"
expect_err 422 "name too long" /projects "key=web&name=$(penc '"n"*61')"
expect_err 422 "bad key" /projects "key=w&name=$(penc '"n"*61')"
expect_err 422 "name required" /projects 'key=web&name=+'
expect_code 200 /projects "key=ops&name=$(penc '"  " + "é"*60 + "  "')"
expect_eq "$(get /projects.txt)" "ABCDEFGHIJ Ten
API Public API
OPS $(ptxt '"é"*60')
WEB Web site" "projects.txt"
expect_has "$(get /)" '<li><a href="/p/OPS">OPS</a> ' "home page uses the stored key"

# reading through any case
expect_eq "$(get /p/web/1.txt)" 'WEB-1 closed P1 alice [bug,ui] Login fails' "lower-case issue path"
expect_eq "$(get '/p/wEb/issues.txt?status=open')" 'WEB-2 open P3 bob [] Typo on home' "mixed-case filter path"
page="$(get /p/web)"
expect_has "$page" '<title>WEB: Web site</title>' "project page title"
expect_has "$page" '<a href="/p/WEB/1">WEB-1</a>' "project page links"
expect_has "$page" 'action="/p/WEB/issues"' "project page form"
expect_has "$(get /p/Web/2)" '<h1>WEB-2: Typo on home</h1>' "issue page"
expect_has "$(get /p/web/1.json)" '{"key":"WEB-1","project":"WEB","number":1,' "json"
expect_eq "$(get /p/web/1/comments.txt)" 'bob: Fixed in <b>v2</b>' "comments path"
expect_eq "$(get /p/web/labels.txt)" $'bug\nui' "labels path"
expect_eq "$(get /p/web/issues.csv | head -2 | tail -1)" 'WEB-1,Login fails,closed,1,alice,bug;ui,Steps: **click**' "csv path"
expect_err 404 "no such project: NOPE" /p/nope
expect_err 404 "no such project: NOPE" /p/nope/1.txt
expect_err 404 "no such project: NOPE" /p/Nope/issues.txt
expect_err 404 "no such issue: WEB-9" /p/web/9
expect_err 404 "no such issue: WEB-9" /p/Web/9/comments.txt
expect_err 404 "no such issue: WEB-9" /p/web/9/status status=closed
expect_err 404 "no such project: NOPE" /p/nope/labels name=x
expect_err 400 "bad status" '/p/web?status=done'

# writing through any case
post /p/api/labels name=docs >/dev/null
expect_eq "$(get /p/API/labels.txt)" docs "label via lower-case path"
expect_has "$(post /p/api/issues 'title=Rate+limits&labels=docs&assignee=bob')" '<h1>API-1: Rate limits</h1>' "create via lower-case path"
post /p/Api/1/comments 'author=alice&body=ok' >/dev/null
post /p/api/1/status status=closed >/dev/null
post /p/aPI/1/edit 'priority=2' >/dev/null
expect_eq "$(get /p/API/1.txt)" 'API-1 closed P2 bob [docs] Rate limits' "writes via lower-case paths"
expect_eq "$(get /p/API/1/activity.txt)" $'created\ncommented alice\nstatus closed' "activity"
expect_err 422 "no such label: bug" /p/api/1/labels label=bug
expect_err 422 "label exists" /p/api/labels name=docs

bb_restart
expect_eq "$(get /projects.txt)" "ABCDEFGHIJ Ten
API Public API
OPS $(ptxt '"é"*60')
WEB Web site" "projects after restart"
expect_eq "$(get /p/api/1.txt)" 'API-1 closed P2 bob [docs] Rate limits' "issue after restart"
expect_err 422 "project exists" /projects 'key=ops&name=x'
expect_eq "$(get /p/web/issues.txt)" $'WEB-1 closed P1 alice [bug,ui] Login fails\nWEB-2 open P3 bob [] Typo on home' "web after restart"
bb_unit
echo PASS
