#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
bb_baseline
# BASE's events interleave across its two issues
expect_eq "$(get /p/BASE/activity.txt)" $'BASE-1 created\nBASE-2 created\nBASE-1 status closed\nBASE-2 assigned bob\nBASE-1 commented bob' "BASE feed"
expect_eq "$(ctype /p/BASE/activity.txt)" "text/plain; charset=utf-8" "content type"
post /projects 'key=WEB&name=Web' >/dev/null
expect_code 200 /p/WEB/activity.txt
expect_eq "$(get /p/WEB/activity.txt | wc -c | tr -d ' ')" 0 "empty feed is the empty body"
post /p/WEB/issues 'title=One' >/dev/null
post /p/WEB/issues 'title=Two' >/dev/null
post /p/WEB/1/status status=closed >/dev/null
post /p/WEB/2/assign assignee=alice >/dev/null
post /p/WEB/1/comments 'author=bob&body=hi' >/dev/null
post /p/WEB/2/status status=closed >/dev/null
post /p/WEB/2/assign assignee= >/dev/null
post /p/WEB/2/assign assignee= >/dev/null   # no change, no row
all='WEB-1 created\nWEB-2 created\nWEB-1 status closed\nWEB-2 assigned alice\nWEB-1 commented bob\nWEB-2 status closed\nWEB-2 assigned -\n'
getf /p/WEB/activity.txt "$RUN/all.txt"; expect_file "$RUN/all.txt" "$all"
expect_eq "$(get '/p/WEB/activity.txt?kind=status')" $'WEB-1 status closed\nWEB-2 status closed' "kind=status"
expect_eq "$(get '/p/WEB/activity.txt?kind=assigned&limit=1')" 'WEB-2 assigned -' "kind+limit"
expect_eq "$(get '/p/WEB/activity.txt?limit=2')" $'WEB-2 status closed\nWEB-2 assigned -' "limit=2 keeps the last two, oldest first"
expect_eq "$(get '/p/WEB/activity.txt?kind=created&limit=1000')" $'WEB-1 created\nWEB-2 created' "kind=created"
expect_eq "$(get '/p/WEB/activity.txt?kind=commented')" 'WEB-1 commented bob' "kind=commented"
expect_eq "$(get '/p/WEB/activity.txt?kind=&limit=')" "$(printf "$all")" "empty values mean defaults"
getf '/p/BASE/activity.txt?kind=commented&limit=5' "$RUN/b.txt"; expect_file "$RUN/b.txt" 'BASE-1 commented bob\n'
# errors
expect_err 400 "bad kind" '/p/WEB/activity.txt?kind=edited'
expect_err 400 "bad kind" '/p/WEB/activity.txt?kind=Status'
expect_err 400 "bad kind" '/p/WEB/activity.txt?kind=bogus&limit=0'
for bad in 0 1001 abc -1 %2B5 1234567890 2.5; do
  expect_err 400 "bad limit" "/p/WEB/activity.txt?limit=$bad"
done
expect_code 200 '/p/WEB/activity.txt?limit=1000'
expect_err 404 "no such project: NOPE" '/p/NOPE/activity.txt?kind=bogus'
expect_err 404 "not found" /p/WEB/activity.txt kind=status
expect_code 404 /p/WEB/activity.json
# per-issue routes unchanged
expect_eq "$(get /p/WEB/1/activity.txt)" $'created\nstatus closed\ncommented bob' "issue activity"
expect_eq "$(get /p/WEB/2.txt)" 'WEB-2 closed P3 - [] Two' "issue txt"
expect_code 200 /p/WEB/1
# default limit is 50: 7 + 45 = 52 rows
for i in $(seq 3 47); do post /p/WEB/issues "title=n+$i" >/dev/null; done
feed="$(get /p/WEB/activity.txt)"
expect_eq "$(wc -l <<<"$feed" | tr -d ' ')" 50 "default limit line count"
expect_eq "$(head -n 1 <<<"$feed")" 'WEB-1 status closed' "default limit keeps the last 50"
expect_eq "$(tail -n 1 <<<"$feed")" 'WEB-47 created' "last line"
expect_eq "$(get '/p/WEB/activity.txt?limit=1000' | wc -l | tr -d ' ')" 52 "limit=1000"
expect_eq "$(get '/p/BASE/activity.txt' | wc -l | tr -d ' ')" 5 "BASE feed untouched"
bb_restart
expect_eq "$(get '/p/WEB/activity.txt?limit=3')" $'WEB-45 created\nWEB-46 created\nWEB-47 created' "after restart"
expect_eq "$(get '/p/WEB/activity.txt?kind=status&limit=1')" 'WEB-2 status closed' "after restart, kind"
bb_unit
echo PASS
