#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
bb_baseline
post /projects 'key=WEB&name=Web' >/dev/null
for l in bug ui wontfix; do post /p/WEB/labels "name=$l" >/dev/null; done
post /p/WEB/issues 'title=Crash&body=one&priority=2&labels=bug' >/dev/null
expect_eq "$(get /p/WEB/1/activity.txt)" 'created' "labels at creation log nothing extra"
# edits
post /p/WEB/1/edit 'title=Crash+on+save' >/dev/null
post /p/WEB/1/edit 'title=Crash+on+save&priority=2' >/dev/null          # same values: nothing
post /p/WEB/1/edit 'priority=4&body=two' >/dev/null
post /p/WEB/1/edit 'title=++Crash+on+save++&body=&priority=' >/dev/null  # trims to the same: nothing
post /p/WEB/1/edit 'body=a%0D%0Ab' >/dev/null                          # stored as a\nb
post /p/WEB/1/edit 'body=a%0Ab' >/dev/null                             # same after cleaning: nothing
expect_err 422 "bad priority" /p/WEB/1/edit 'title=Other&priority=9'
expect_err 400 "bad priority" /p/WEB/1/edit 'title=Other&priority=x'
expect_eq "$(get /p/WEB/1/activity.txt)" $'created\nedited title\nedited body,priority\nedited body' "edits"
expect_eq "$(get /p/WEB/1.txt)" 'WEB-1 open P4 - [bug] Crash on save' "issue after edits"
# labels
post /p/WEB/1/labels label=bug >/dev/null        # already attached at creation
post /p/WEB/1/labels label=ui >/dev/null
post /p/WEB/1/labels label=ui >/dev/null         # already attached
post /p/WEB/1/labels/remove label=ui >/dev/null
post /p/WEB/1/labels/remove label=ui >/dev/null  # not attached
post /p/WEB/1/labels/remove label=wontfix >/dev/null
expect_err 422 "no such label: nope" /p/WEB/1/labels label=nope
expect_err 422 "label required" /p/WEB/1/labels/remove label=
expect_err 422 "bad status" /p/WEB/1/status status=done
post /p/WEB/1/status status=closed >/dev/null
post /p/WEB/1/status status=closed >/dev/null
want=$'created\nedited title\nedited body,priority\nedited body\nlabeled ui\nunlabeled ui\nstatus closed'
expect_eq "$(get /p/WEB/1/activity.txt)" "$want" "activity.txt"
expect_eq "$(get /p/WEB/1.txt)" 'WEB-1 closed P4 - [bug] Crash on save' "issue after labels"
json="$(get /p/WEB/1.json)"
expect_has "$json" '"activity":[{"kind":"created","detail":""},{"kind":"edited","detail":"title"},{"kind":"edited","detail":"body,priority"},{"kind":"edited","detail":"body"},{"kind":"labeled","detail":"ui"},{"kind":"unlabeled","detail":"ui"},{"kind":"status","detail":"closed"}]}' "json activity"
expect_has "$json" '"body":"a\nb","status":"closed","priority":4' "json fields"
page="$(get /p/WEB/1)"
expect_has "$page" '<li>edited body,priority</li>' "issue page"
expect_has "$page" '<li>labeled ui</li><li>unlabeled ui</li><li>status closed</li></ul>' "issue page"
# a second issue is independent; untouched issues are unchanged
post /p/WEB/issues 'title=Second' >/dev/null
post /p/WEB/2/labels label=wontfix >/dev/null
post /p/WEB/2/edit 'title=Second+one' >/dev/null
expect_eq "$(get /p/WEB/2/activity.txt)" $'created\nlabeled wontfix\nedited title' "second issue"
expect_eq "$(get /p/BASE/1/activity.txt)" $'created\nstatus closed\ncommented bob' "BASE-1 unchanged"
bb_restart
expect_eq "$(get /p/WEB/1/activity.txt)" "$want" "activity.txt after restart"
post /p/WEB/1/labels label=wontfix >/dev/null
expect_eq "$(get /p/WEB/1/activity.txt)" "$want"$'\nlabeled wontfix' "logging after restart"
bb_unit
echo PASS
