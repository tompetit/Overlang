#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
bb_baseline
# penc EXPR: a Python string expression, form-encoded; ptxt EXPR: the same string as UTF-8 text.
penc() { python3 -c 'import sys,urllib.parse as u;sys.stdout.write(u.quote_plus(eval(sys.argv[1])))' "$1"; }
ptxt() { python3 -c 'import sys;sys.stdout.buffer.write(eval(sys.argv[1]).encode())' "$1"; }
bodylen() { get "$1" | python3 -c 'import json,sys;print(len(json.load(sys.stdin)["body"]))'; }

# titles: inner runs of spaces/tabs collapse; bodies keep their spacing
expect_code 200 /p/BASE/issues "title=$(penc '"  Spaced    out\t\ttitle  "')&body=$(penc '"keep   these\n  spaces"')"
expect_eq "$(get /p/BASE/3.txt)" 'BASE-3 open P3 - [] Spaced out title' "normalised title"
expect_has "$(get /p/BASE/3.json)" '"title":"Spaced out title","body":"keep   these\n  spaces"' "json"
expect_has "$(get /p/BASE/3)" '<h1>BASE-3: Spaced out title</h1>' "issue page"
expect_has "$(get /p/BASE/issues.csv)" 'BASE-3,Spaced out title,open,3,,,' "csv"

# 120 code points (after trimming) is fine, 121 is not; code points, not bytes
expect_code 200 /p/BASE/issues "title=$(penc '"   " + "é"*120 + "   "')"
expect_eq "$(get /p/BASE/4.txt)" "BASE-4 open P3 - [] $(ptxt '"é"*120')" "120 code points"
expect_err 422 "title too long" /p/BASE/issues "title=$(penc '"é"*121')"
expect_err 422 "title too long" /p/BASE/issues "title=$(penc '"a"*121')"
# the length is measured after collapsing: 24 words with 3 spaces between are 165 raw, 119 stored
expect_code 200 /p/BASE/issues "title=$(penc '"   ".join(["word"]*24)')"
expect_eq "$(get /p/BASE/5.txt)" "BASE-5 open P3 - [] $(ptxt '" ".join(["word"]*24)')" "collapsed then measured"
expect_err 422 "title too long" /p/BASE/issues "title=$(penc '" ".join(["word"]*25)')"

# order of checks on create
expect_err 422 "title required" /p/BASE/issues "title=+++&body=$(penc '"b"*2001')"
expect_err 422 "title too long" /p/BASE/issues "title=$(penc '"t"*121')&body=$(penc '"b"*2001')&priority=9"
expect_err 422 "body too long" /p/BASE/issues "title=ok&body=$(penc '"b"*2001')&priority=abc"
expect_err 422 "body too long" /p/BASE/issues "title=ok&body=$(penc '"b"*2001')&assignee=zed&labels=nope"
expect_err 422 "bad priority" /p/BASE/issues "title=$(penc '"t"*120')&body=$(penc '"b"*2000')&priority=9"
expect_err 404 "no such issue: BASE-6" /p/BASE/6.txt

# body length is measured after cleaning: \r\n counts once, surrounding whitespace is trimmed
expect_code 200 /p/BASE/issues "title=Long+body&body=$(penc '"  " + "abc\r\n"*499 + "abcd" + "\r\n  "')"
expect_eq "$(bodylen /p/BASE/6.json)" 2000 "stored body length"
expect_err 422 "body too long" /p/BASE/issues "title=Longer&body=$(penc '"abc\r\n"*499 + "abcde"')"
expect_err 404 "no such issue: BASE-7" /p/BASE/7.txt

# edit
expect_err 422 "title too long" /p/BASE/2/edit "title=$(penc '"x"*121')&priority=1"
expect_err 422 "title too long" /p/BASE/2/edit "title=$(penc '"x"*121')&body=$(penc '"y"*2001')&priority=abc"
expect_err 422 "body too long" /p/BASE/2/edit "body=$(penc '"x"*2001')&priority=abc"
expect_err 422 "body too long" /p/BASE/2/edit "title=New&body=$(penc '"x"*2001')&priority=1"
expect_err 422 "bad priority" /p/BASE/2/edit "title=$(penc '"x"*120')&priority=7"
expect_eq "$(get /p/BASE/2.txt)" 'BASE-2 open P3 bob [] Typo on home' "rejected edits change nothing"
expect_has "$(get /p/BASE/2.json)" '"body":"50% of the text"' "rejected edits keep the body"
expect_code 200 /p/BASE/2/edit "title=$(penc '"  Typo \t on   the  home "')&priority=2"
expect_eq "$(get /p/BASE/2.txt)" 'BASE-2 open P2 bob [] Typo on the home' "edit normalises the title"
expect_code 200 /p/BASE/2/edit "title=+++&body=&priority="
expect_eq "$(get /p/BASE/2.txt)" 'BASE-2 open P2 bob [] Typo on the home' "blank title keeps the current one"
expect_code 200 /p/BASE/5/edit "title=$(penc '"é"*120')&body=$(penc '"é"*2000')"
expect_eq "$(bodylen /p/BASE/5.json)" 2000 "edited body length"

# comments
expect_err 422 "body too long" /p/BASE/1/comments "author=bob&body=$(penc '"c"*2001')"
expect_err 422 "no such user: zed" /p/BASE/1/comments "author=zed&body=$(penc '"c"*2001')"
expect_err 422 "author required" /p/BASE/1/comments "author=&body=$(penc '"c"*2001')"
expect_err 422 "body required" /p/BASE/1/comments "author=bob&body=+%0D%0A+"
expect_eq "$(get /p/BASE/1/comments.txt)" 'bob: Fixed in <b>v2</b>' "rejected comments add nothing"
expect_code 200 /p/BASE/1/comments "author=alice&body=$(penc '"é"*2000')"
expect_eq "$(get /p/BASE/1/comments.txt)" "bob: Fixed in <b>v2</b>"$'\n'"alice: $(ptxt '"é"*2000')" "comments"
expect_eq "$(get /p/BASE/1/activity.txt)" $'created\nstatus closed\ncommented bob\ncommented alice' "activity"

bb_restart
want="BASE-1 closed P1 alice [bug,ui] Login fails
BASE-2 open P2 bob [] Typo on the home
BASE-3 open P3 - [] Spaced out title
BASE-4 open P3 - [] $(ptxt '"é"*120')
BASE-5 open P3 - [] $(ptxt '"é"*120')
BASE-6 open P3 - [] Long body"
expect_eq "$(get /p/BASE/issues.txt)" "$want" "issues after restart"
expect_eq "$(bodylen /p/BASE/6.json)" 2000 "body after restart"
expect_eq "$(get /p/BASE/1/comments.txt | wc -l)" 2 "comments after restart"
bb_unit
echo PASS
