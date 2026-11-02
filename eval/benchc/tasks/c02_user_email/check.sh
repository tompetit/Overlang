#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
# Old behaviour (users.txt changes, so not bb_baseline).
expect_eq "$(get /health)" "ok" "health"
expect_code 200 /users name=alice
expect_code 200 /users name=bob
expect_err 422 "user exists" /users name=alice
expect_err 422 "name required" /users name=
expect_err 422 "bad name" /users "name=a+b"
expect_code 200 /projects 'key=WEB&name=Web+site'
expect_eq "$(get /projects.txt)" "WEB Web site" "projects.txt"
post /p/WEB/labels name=bug >/dev/null; post /p/WEB/labels name=ui >/dev/null
post /p/WEB/issues 'title=Login+fails&body=Steps%3A+**click**+%60login%60&priority=1&assignee=alice&labels=bug,ui' >/dev/null
post /p/WEB/issues 'title=Typo+on+home&body=50%25+of+the+text' >/dev/null
post /p/WEB/issues 'title=Third' >/dev/null
post /p/WEB/1/status status=closed >/dev/null
post /p/WEB/1/comments 'author=bob&body=Fixed+in+%3Cb%3Ev2%3C%2Fb%3E' >/dev/null
expect_eq "$(get /p/WEB/issues.txt)" $'WEB-1 closed P1 alice [bug,ui] Login fails\nWEB-2 open P3 - [] Typo on home\nWEB-3 open P3 - [] Third' "issues.txt"
expect_eq "$(get /p/WEB/1/activity.txt)" $'created\nstatus closed\ncommented bob' "activity"
expect_eq "$(get '/p/WEB/issues.txt?assignee=alice')" 'WEB-1 closed P1 alice [bug,ui] Login fails' "assignee filter"
expect_err 422 "no such user: zed" /p/WEB/2/assign assignee=zed
expect_err 404 "no such project: NOPE" /p/NOPE
expect_err 404 "not found" /nope

# Users without email: "-", and several of them may exist.
expect_eq "$(get /users.txt)" $'1 alice -\n2 bob -' "users.txt no email"

# Create with email: lowercased, trimmed; empty means none.
expect_code 200 /users 'name=carol&email=+Carol.King%40Example.ORG+'
expect_code 200 /users 'name=dave&email='
expect_code 200 /users name=erin
long="$(printf 'a%.0s' $(seq 242))@example.org"   # 254 code points
expect_code 200 /users "name=long&email=$long"
expect_code 200 /users 'name=frank&email=f%2Btag_1%25x-y.z%40mail-1.example.org'
want=$'1 alice -\n2 bob -\n3 carol carol.king@example.org\n4 dave -\n5 erin -\n6 long '"$long"$'\n7 frank f+tag_1%x-y.z@mail-1.example.org'
expect_eq "$(get /users.txt)" "$want" "users.txt with emails"

# Errors on create, in order; nothing stored.
expect_err 422 "user exists" /users 'name=alice&email=bad'
expect_err 422 "bad name" /users 'name=a+b&email=bad'
expect_err 422 "name required" /users 'name=&email=x%40y.org'
expect_err 422 "email exists" /users 'name=gina&email=CAROL.KING%40example.org'
for e in gina gina%40 %40example.org g%40example g%40%40example.org 'g+h%40example.org' g%40example..org \
         g%40.example.org g%40example.org. 'g!%40example.org' g%40exa_mple.org 'g%40example.org%40x.org' \
         "a$long"; do
  expect_err 422 "bad email" /users "name=gina&email=$e"
done
expect_eq "$(get /users.txt)" "$want" "nothing stored on errors"

# Change email.
expect_code 200 /users/alice/email 'email=ALICE%40example.org'
expect_err 422 "email exists" /users/bob/email 'email=alice%40EXAMPLE.org'
expect_code 200 /users/alice/email 'email=Alice%40Example.org'
expect_code 200 /users/carol/email 'email='
expect_code 200 /users/bob/email 'email=carol.king%40example.org'
expect_err 422 "bad email" /users/dave/email 'email=bad'
expect_err 404 "no such user: nobody" /users/nobody/email 'email=x%40y.org'
expect_err 404 "no such user: nobody" /users/nobody/email 'email=bad'
expect_err 404 "no such user: no%20body" /users/no%20body/email 'email=x%40y.org'
expect_err 404 "not found" /users/alice/email
expect_err 404 "not found" /users/alice 'email=x%40y.org'
want=$'1 alice alice@example.org\n2 bob carol.king@example.org\n3 carol -\n4 dave -\n5 erin -\n6 long '"$long"$'\n7 frank f+tag_1%x-y.z@mail-1.example.org'
expect_eq "$(get /users.txt)" "$want" "users.txt after changes"
expect_err 422 "email exists" /users 'name=gina&email=Carol.King%40example.org'
expect_code 200 /users 'name=gina&email=GINA%40example.org'

# Users page.
page="$(get /users)"
expect_has "$page" '<li>alice <a href="mailto:alice@example.org">alice@example.org</a></li><li>bob <a href="mailto:carol.king@example.org">carol.king@example.org</a></li><li>carol</li><li>dave</li>' "users page"
expect_has "$page" '<input name="name" placeholder="name"> <input name="email" placeholder="email"> <button>create user</button>' "users form"

# Issues show the assignee's current email.
post /p/WEB/2/assign assignee=dave >/dev/null
expect_eq "$(get /p/WEB/1.json)" '{"key":"WEB-1","project":"WEB","number":1,"title":"Login fails","body":"Steps: **click** `login`","status":"closed","priority":1,"assignee":"alice","assignee_email":"alice@example.org","labels":["bug","ui"],"comments":[{"author":"bob","body":"Fixed in <b>v2</b>"}],"activity":[{"kind":"created","detail":""},{"kind":"status","detail":"closed"},{"kind":"commented","detail":"bob"}]}' "json with email"
expect_has "$(get /p/WEB/2.json)" '"assignee":"dave","assignee_email":null,"labels":[]' "json assignee without email"
expect_has "$(get /p/WEB/3.json)" '"assignee":null,"assignee_email":null,"labels":[]' "json unassigned"
expect_has "$(get /p/WEB/1)" 'Priority: P1 | Assignee: <a href="mailto:alice@example.org">alice</a> | Labels: bug, ui</p>' "issue page email"
expect_has "$(get /p/WEB/2)" 'Priority: P3 | Assignee: dave | Labels: -</p>' "issue page no email"
expect_has "$(get /p/WEB/3)" 'Priority: P3 | Assignee: - | Labels: -</p>' "issue page unassigned"
expect_code 200 /users/dave/email 'email=Dave%40x.io'
expect_has "$(get /p/WEB/2.json)" '"assignee":"dave","assignee_email":"dave@x.io",' "json follows email change"
expect_has "$(get /p/WEB/2)" 'Assignee: <a href="mailto:dave@x.io">dave</a> |' "page follows email change"
expect_eq "$(get /p/WEB/issues.txt)" $'WEB-1 closed P1 alice [bug,ui] Login fails\nWEB-2 open P3 dave [] Typo on home\nWEB-3 open P3 - [] Third' "issues.txt unchanged"
expect_has "$(get /p/WEB/issues.csv)" 'WEB-2,Typo on home,open,3,dave,,50% of the text' "csv unchanged"

# Persistence.
bb_restart
want=$'1 alice alice@example.org\n2 bob carol.king@example.org\n3 carol -\n4 dave dave@x.io\n5 erin -\n6 long '"$long"$'\n7 frank f+tag_1%x-y.z@mail-1.example.org\n8 gina gina@example.org'
expect_eq "$(get /users.txt)" "$want" "users.txt after restart"
expect_err 422 "email exists" /users/erin/email 'email=DAVE%40X.IO'
expect_has "$(get /p/WEB/1.json)" '"assignee":"alice","assignee_email":"alice@example.org",' "json after restart"
bb_unit
echo PASS
