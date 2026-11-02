#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
bb_baseline
# del PATH: POST with an empty body; prints "<status>" then the body on the next lines.
del() { curl -s --max-time 10 -w '\n%{http_code}' -d '' "$URL$1"; }

# a second project with a comment of its own; more comments in BASE
post /projects 'key=OTHER&name=Other' >/dev/null
post /p/OTHER/issues 'title=Secret' >/dev/null
post /p/OTHER/1/comments 'author=alice&body=secret+note' >/dev/null
post /p/BASE/2/comments 'author=alice&body=line1%0D%0Aline2' >/dev/null
post /p/BASE/1/comments 'author=alice&body=second' >/dev/null
expect_eq "$(bb_sql 'select c.id, i.number, c.body from comments c join issues i on i.id = c.issue_id order by c.id' | tr '\n' ';')" \
  '1|1|Fixed in <b>v2</b>;2|1|secret note;3|2|line1;line2;4|1|second;' "comment ids"

# reading one comment
expect_eq "$(get /p/BASE/1/comments/1.txt)" 'bob: Fixed in <b>v2</b>' "comment 1"
expect_eq "$(ctype /p/BASE/1/comments/1.txt)" 'text/plain; charset=utf-8' "content type"
expect_eq "$(get /p/BASE/2/comments/3.txt)" 'alice: line1 line2' "newlines become spaces"
expect_eq "$(get /p/BASE/1/comments/004.txt)" 'alice: second' "leading zeros"
expect_eq "$(get /p/OTHER/1/comments/2.txt)" 'alice: secret note' "other project"
expect_err 404 "no such comment: 2" /p/BASE/1/comments/2.txt
expect_err 404 "no such comment: 3" /p/BASE/1/comments/3.txt
expect_err 404 "no such comment: 4" /p/OTHER/1/comments/004.txt
expect_err 404 "no such comment: 99" /p/BASE/1/comments/99.txt
expect_err 404 "no such issue: BASE-9" /p/BASE/9/comments/1.txt
expect_err 404 "no such issue: OTHER-2" /p/OTHER/2/comments/2.txt
expect_err 404 "no such project: NOPE" /p/NOPE/1/comments/1.txt
expect_err 404 "not found" /p/BASE/1/comments/0.txt
expect_err 404 "not found" /p/BASE/1/comments/x.txt
expect_err 404 "not found" /p/BASE/1/comments/1
expect_err 404 "not found" /p/BASE/1/comments/1.json
expect_err 404 "not found" /p/BASE/1/comments/1.txt x=1
expect_err 404 "not found" /p/BASE/1/comments/1/delete

# deleting through the wrong issue or project deletes nothing
expect_err 404 "no such comment: 2" /p/BASE/1/comments/2/delete ''
expect_err 404 "no such comment: 3" /p/BASE/1/comments/3/delete ''
expect_err 404 "no such comment: 1" /p/OTHER/1/comments/1/delete ''
expect_err 404 "no such comment: 1" /p/BASE/2/comments/1/delete ''
expect_err 404 "no such issue: OTHER-2" /p/OTHER/2/comments/2/delete ''
expect_err 404 "no such project: NOPE" /p/NOPE/1/comments/2/delete ''
expect_err 404 "not found" /p/BASE/1/comments/0/delete ''
expect_eq "$(get /p/BASE/1/comments.txt)" $'bob: Fixed in <b>v2</b>\nalice: second' "BASE-1 comments intact"
expect_eq "$(get /p/BASE/2/comments.txt)" 'alice: line1 line2' "BASE-2 comments intact"
expect_eq "$(get /p/OTHER/1/comments.txt)" 'alice: secret note' "OTHER-1 comments intact"

# deleting
out="$(del /p/BASE/1/comments/1/delete)"
expect_eq "${out##*$'\n'}" 200 "delete status"
expect_has "$out" '<h1>BASE-1: Login fails</h1>' "answers the issue page"
expect_lacks "$out" 'Fixed in' "deleted comment gone from the page"
expect_has "$out" '<li><b>alice</b> <p>second</p></li>' "other comment stays"
expect_eq "$(get /p/BASE/1/comments.txt)" 'alice: second' "comments.txt"
expect_has "$(get /p/BASE/1.json)" '"comments":[{"author":"alice","body":"second"}]' "json"
expect_eq "$(get /p/BASE/1/activity.txt)" $'created\nstatus closed\ncommented bob\ncommented alice' "activity unchanged"
expect_err 404 "no such comment: 1" /p/BASE/1/comments/1.txt
expect_err 404 "no such comment: 1" /p/BASE/1/comments/01/delete ''
out="$(del /p/OTHER/1/comments/002/delete)"
expect_eq "${out##*$'\n'}" 200 "delete with leading zeros"
expect_eq "$(get /p/OTHER/1/comments.txt)" '' "OTHER-1 has no comments"
expect_eq "$(get /p/BASE/2/comments.txt)" 'alice: line1 line2' "BASE-2 untouched"

# ids are not reused
post /p/BASE/1/comments 'author=bob&body=again' >/dev/null
expect_eq "$(get /p/BASE/1/comments/5.txt)" 'bob: again' "new comment id"
expect_err 404 "no such comment: 2" /p/BASE/1/comments/2.txt

bb_restart
expect_eq "$(get /p/BASE/1/comments.txt)" $'alice: second\nbob: again' "comments after restart"
expect_err 404 "no such comment: 1" /p/BASE/1/comments/1.txt
expect_err 404 "no such comment: 2" /p/OTHER/1/comments/2.txt
expect_eq "$(get /p/BASE/2/comments/3.txt)" 'alice: line1 line2' "comment 3 after restart"
expect_eq "$(bb_sql 'select count(*) from comments')" 3 "rows after restart"
bb_unit
echo PASS
