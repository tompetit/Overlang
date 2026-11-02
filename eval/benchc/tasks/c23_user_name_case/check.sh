#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
bb_baseline

# creating users
expect_err 422 "user exists" /users name=Alice
expect_err 422 "user exists" /users 'name=+%40ALICE+'
expect_err 422 "name required" /users 'name=%40'
expect_err 422 "name required" /users 'name=+%40+'
expect_err 422 "bad name" /users 'name=%40%40bob'
expect_err 422 "bad name" /users 'name=%40a+b'
expect_code 200 /users 'name=%40Carol.D'
expect_has "$(post /users 'name=DAVE_1')" '<li>dave_1</li>' "users page"
expect_eq "$(get /users.txt)" $'1 alice\n2 bob\n3 carol.d\n4 dave_1' "users.txt"
expect_err 422 "user exists" /users 'name=Dave_1'

# assignee on create
expect_has "$(post /p/BASE/issues 'title=Mixed+case&assignee=%40Carol.D')" 'Assignee: carol.d' "create page"
expect_eq "$(get /p/BASE/3.txt)" 'BASE-3 open P3 carol.d [] Mixed case' "assignee normalised"
expect_err 422 "no such user: zed" /p/BASE/issues 'title=x&assignee=%40ZED'
expect_err 422 "no such user: zed" /p/BASE/issues 'title=x&assignee=Zed&labels=nope'
expect_err 422 "no such user: @zed" /p/BASE/issues 'title=x&assignee=%40%40Zed'
expect_code 200 /p/BASE/issues 'title=Nobody&assignee=+%40+'
expect_eq "$(get /p/BASE/4.txt)" 'BASE-4 open P3 - [] Nobody' "@ alone is no assignee"
expect_err 404 "no such issue: BASE-5" /p/BASE/5

# assign
post /p/BASE/4/assign assignee=BOB >/dev/null
expect_eq "$(get /p/BASE/4.txt)" 'BASE-4 open P3 bob [] Nobody' "assign upper case"
expect_err 422 "no such user: zed" /p/BASE/4/assign 'assignee=%40Zed'
post /p/BASE/4/assign 'assignee=+%40Bob+' >/dev/null
expect_eq "$(get /p/BASE/4/activity.txt)" $'created\nassigned bob' "same user is no change"
post /p/BASE/4/assign 'assignee=%40' >/dev/null
expect_eq "$(get /p/BASE/4.txt)" 'BASE-4 open P3 - [] Nobody' "@ unassigns"
post /p/BASE/4/assign 'assignee=%40CAROL.D' >/dev/null
expect_eq "$(get /p/BASE/4/activity.txt)" $'created\nassigned bob\nassigned -\nassigned carol.d' "assign activity"

# comment author
expect_err 422 "author required" /p/BASE/4/comments 'author=%40&body=hi'
expect_err 422 "no such user: zed" /p/BASE/4/comments 'author=ZED&body=hi'
expect_err 422 "no such user: zed" /p/BASE/4/comments 'author=%40zed&body='
expect_code 200 /p/BASE/4/comments 'author=%40Dave_1&body=hi'
expect_eq "$(get /p/BASE/4/comments.txt)" 'dave_1: hi' "author normalised"
expect_eq "$(get /p/BASE/4/activity.txt | tail -1)" 'commented dave_1' "comment activity"

# assignee filter
expect_eq "$(get '/p/BASE/issues.txt?assignee=%40BOB')" 'BASE-2 open P3 bob [] Typo on home' "filter @BOB"
expect_eq "$(get '/p/BASE/issues.txt?assignee=Carol.D')" $'BASE-3 open P3 carol.d [] Mixed case\nBASE-4 open P3 carol.d [] Nobody' "filter Carol.D"
expect_eq "$(get '/p/BASE/issues.txt?assignee=%40&status=closed')" 'BASE-1 closed P1 alice [bug,ui] Login fails' "@ alone is no filter"
expect_eq "$(get '/p/BASE/issues.txt?assignee=ZED')" '' "unknown assignee matches nothing"
page="$(get '/p/BASE?assignee=+%40Alice+')"
expect_has "$page" '<input name="assignee" value="alice" placeholder="assignee">' "filter form value"
expect_has "$page" 'Page 1 of 1 (1 issues)' "filtered count"
expect_err 400 "bad status" '/p/BASE/issues.txt?assignee=%40Bob&status=x'

bb_restart
expect_eq "$(get /users.txt)" $'1 alice\n2 bob\n3 carol.d\n4 dave_1' "users after restart"
expect_err 422 "user exists" /users 'name=%40CAROL.d'
expect_eq "$(get /p/BASE/issues.txt)" "BASE-1 closed P1 alice [bug,ui] Login fails
BASE-2 open P3 bob [] Typo on home
BASE-3 open P3 carol.d [] Mixed case
BASE-4 open P3 carol.d [] Nobody" "issues after restart"
expect_eq "$(get '/p/BASE/issues.txt?assignee=%40Dave_1')" '' "filter after restart"
bb_unit
echo PASS
