#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
bb_baseline   # project BASE: BASE-1 closed with comment 1 by bob, BASE-2 assigned to bob

expect_code 200 /p/BASE/2/comments 'author=alice&body=On+it'      # comment 2
expect_code 200 /p/BASE/1/comments 'author=alice&body=Thanks'     # comment 3
expect_eq "$(get /p/BASE/1/comments.txt)" $'bob: Fixed in <b>v2</b>\nalice: Thanks' "comments before"
expect_has "$(get /p/BASE/1)" '<ul class="comments"><li><b>bob</b> <p>Fixed in &lt;b&gt;v2&lt;/b&gt;</p></li><li><b>alice</b> <p>Thanks</p></li></ul>' "comments html before"
expect_has "$(get /p/BASE/1.json)" '"comments":[{"id":1,"author":"bob","body":"Fixed in <b>v2</b>","edits":0},{"id":3,"author":"alice","body":"Thanks","edits":0}],' "json before"
expect_err 422 "body required" /p/BASE/1/comments 'author=bob&body='
expect_err 422 "no such user: zed" /p/BASE/1/comments 'author=zed&body=x'

# Errors, in order; nothing changes.
act1="$(get /p/BASE/1/activity.txt)"
expect_err 404 "no such comment: 2" /p/BASE/1/comments/2/edit 'author=alice&body=x'
expect_err 404 "no such comment: 99" /p/BASE/1/comments/99/edit 'author=bob&body=x'
expect_err 404 "no such comment: 99" /p/BASE/1/comments/99/edit 'author=&body='
expect_err 404 "no such issue: BASE-9" /p/BASE/9/comments/1/edit 'author=bob&body=x'
expect_err 404 "no such project: NOPE" /p/NOPE/1/comments/1/edit 'author=bob&body=x'
expect_err 404 "not found" /p/BASE/1/comments/0/edit 'author=bob&body=x'
expect_err 404 "not found" /p/BASE/1/comments/x/edit 'author=bob&body=x'
expect_err 404 "not found" /p/BASE/1/comments/1 'author=bob&body=x'
expect_err 404 "not found" /p/BASE/1/comments/1/edit
expect_err 422 "author required" /p/BASE/1/comments/1/edit 'author=&body=x'
expect_err 422 "no such user: zed" /p/BASE/1/comments/1/edit 'author=zed&body='
expect_err 422 "not the author" /p/BASE/1/comments/1/edit 'author=alice&body=x'
expect_err 422 "not the author" /p/BASE/1/comments/1/edit 'author=alice&body='
expect_err 422 "body required" /p/BASE/1/comments/1/edit 'author=bob&body='
expect_err 422 "body required" /p/BASE/1/comments/1/edit 'author=bob&body=+%0D%0A+'
expect_eq "$(get /p/BASE/1/comments.txt)" $'bob: Fixed in <b>v2</b>\nalice: Thanks' "unchanged on errors"
expect_eq "$(get /p/BASE/1/activity.txt)" "$act1" "no activity on errors"

# Edit.
page="$(post /p/BASE/1/comments/1/edit 'author=bob&body=Fixed+in+**v3**')"
expect_has "$page" '<h1>BASE-1: Login fails</h1>' "answers issue page"
expect_has "$page" '<li><b>bob</b> <span class="edited">(edited)</span> <p>Fixed in <strong>v3</strong></p></li><li><b>alice</b> <p>Thanks</p></li>' "edited html"
expect_eq "$(get /p/BASE/1/comments.txt)" $'bob: Fixed in **v3** (edited)\nalice: Thanks' "edited text"
expect_eq "$(get /p/BASE/1/activity.txt)" $'created\nstatus closed\ncommented bob\ncommented alice\nedited bob' "edited activity"

# Same body after cleaning: no change, no activity.
expect_code 200 /p/BASE/1/comments/1/edit 'author=bob&body=+Fixed+in+**v3**%0D%0A'
expect_eq "$(get /p/BASE/1/activity.txt)" $'created\nstatus closed\ncommented bob\ncommented alice\nedited bob' "no-op edit"
expect_eq "$(get /p/BASE/1.json)" '{"key":"BASE-1","project":"BASE","number":1,"title":"Login fails","body":"Steps: **click**","status":"closed","priority":1,"assignee":"alice","labels":["bug","ui"],"comments":[{"id":1,"author":"bob","body":"Fixed in **v3**","edits":1},{"id":3,"author":"alice","body":"Thanks","edits":0}],"activity":[{"kind":"created","detail":""},{"kind":"status","detail":"closed"},{"kind":"commented","detail":"bob"},{"kind":"commented","detail":"alice"},{"kind":"edited","detail":"bob"}]}' "json after edit"

# Second edit, multi-line body (CRLF cleaned to \n).
expect_code 200 /p/BASE/1/comments/1/edit 'author=bob&body=line1%0D%0Aline2'
expect_eq "$(get /p/BASE/1/comments.txt)" $'bob: line1 line2 (edited)\nalice: Thanks' "second edit text"
expect_has "$(get /p/BASE/1.json)" '{"id":1,"author":"bob","body":"line1\nline2","edits":2}' "second edit json"

# Comment on another issue, edited through its own issue.
expect_code 200 /p/BASE/2/comments/2/edit 'author=alice&body=Done'
expect_eq "$(get /p/BASE/2/comments.txt)" 'alice: Done (edited)' "issue 2 comment"
expect_eq "$(get /p/BASE/2/activity.txt)" $'created\nassigned bob\ncommented alice\nedited alice' "issue 2 activity"
expect_eq "$(get /p/BASE/1/comments.txt)" $'bob: line1 line2 (edited)\nalice: Thanks' "issue 1 untouched"

# New comments still start unedited.
expect_code 200 /p/BASE/1/comments 'author=bob&body=More'
expect_has "$(get /p/BASE/1.json)" '{"id":4,"author":"bob","body":"More","edits":0}]' "new comment"
expect_has "$(get /p/BASE/1)" '<li><b>bob</b> <p>More</p></li></ul>' "new comment html"

# Persistence.
bb_restart
expect_eq "$(get /p/BASE/1/comments.txt)" $'bob: line1 line2 (edited)\nalice: Thanks\nbob: More' "comments after restart"
expect_has "$(get /p/BASE/1.json)" '"comments":[{"id":1,"author":"bob","body":"line1\nline2","edits":2},{"id":3,"author":"alice","body":"Thanks","edits":0},{"id":4,"author":"bob","body":"More","edits":0}],' "json after restart"
expect_eq "$(get /p/BASE/2/activity.txt)" $'created\nassigned bob\ncommented alice\nedited alice' "activity after restart"
expect_code 200 /p/BASE/1/comments/3/edit 'author=alice&body=Thanks!'
expect_eq "$(get /p/BASE/1/activity.txt)" $'created\nstatus closed\ncommented bob\ncommented alice\nedited bob\nedited bob\ncommented bob\nedited alice' "edit after restart"
bb_unit
echo PASS
