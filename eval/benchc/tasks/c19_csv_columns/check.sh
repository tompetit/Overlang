#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
bb_baseline
post /projects 'key=EMPTY&name=Nothing' >/dev/null
expect_eq "$(ctype /p/EMPTY/issues.csv)" "text/csv; charset=utf-8" "content type"
getf /p/EMPTY/issues.csv "$RUN/empty.csv"
expect_file "$RUN/empty.csv" 'key,title,status,priority,assignee,labels,body,comments,last_activity\r\n'
post /projects 'key=WEB&name=Web' >/dev/null
post /p/WEB/labels name=-wip >/dev/null; post /p/WEB/labels name=bug >/dev/null
post /p/WEB/issues 'title=%3DSUM(A1%2CB1)&body=-+item+one%0D%0A-+item+two&priority=2&assignee=alice&labels=bug,-wip' >/dev/null
post /p/WEB/issues 'title=%40home+%2B1&body=say+%22hi%22' >/dev/null
post /p/WEB/issues 'title=Plain' >/dev/null
post /p/WEB/1/comments 'author=alice&body=first' >/dev/null
post /p/WEB/1/status status=closed >/dev/null
post /p/WEB/1/assign assignee=bob >/dev/null
post /p/WEB/1/assign assignee= >/dev/null
post /p/WEB/3/comments 'author=bob&body=x' >/dev/null
post /p/WEB/3/comments 'author=alice&body=y%2C+z' >/dev/null
post /p/WEB/3/status status=open >/dev/null   # no change, no activity
want=$'key,title,status,priority,assignee,labels,body,comments,last_activity\r\n'
want+=$'WEB-1,"\'=SUM(A1,B1)",closed,2,,\'-wip;bug,"\'- item one\n- item two",1,assigned -\r\n'
want+=$'WEB-2,\'@home +1,open,3,,,"say ""hi""",0,created\r\n'
want+=$'WEB-3,Plain,open,3,,,,2,commented alice\r\n'
getf /p/WEB/issues.csv "$RUN/web.csv"; expect_file "$RUN/web.csv" "$want"
getf /p/BASE/issues.csv "$RUN/base.csv"
expect_file "$RUN/base.csv" $'key,title,status,priority,assignee,labels,body,comments,last_activity\r\nBASE-1,Login fails,closed,1,alice,bug;ui,Steps: **click**,1,commented bob\r\nBASE-2,Typo on home,open,3,bob,,50%% of the text,0,assigned bob\r\n'
# other outputs keep the stored values
expect_eq "$(get /p/WEB/issues.txt)" $'WEB-1 closed P2 - [-wip,bug] =SUM(A1,B1)\nWEB-2 open P3 - [] @home +1\nWEB-3 open P3 - [] Plain' "issues.txt"
expect_has "$(get /p/WEB/2.json)" '"title":"@home +1","body":"say \"hi\""' "json unguarded"
expect_has "$(get /p/WEB/1)" '<h1>WEB-1: =SUM(A1,B1)</h1>' "html unguarded"
expect_eq "$(get /p/WEB/3/comments.txt)" $'bob: x\nalice: y, z' "comments.txt"
expect_err 404 "no such project: NOPE" /p/NOPE/issues.csv
expect_err 404 "not found" /p/WEB/issues.csv x=1
bb_restart
getf /p/WEB/issues.csv "$RUN/web2.csv"; expect_file "$RUN/web2.csv" "$want"
post /p/WEB/2/comments 'author=bob&body=%3Dcmd' >/dev/null
getf /p/WEB/issues.csv "$RUN/web3.csv"
expect_file "$RUN/web3.csv" "${want/$'0,created\r\n'/$'1,commented bob\r\n'}"
bb_unit
echo PASS
