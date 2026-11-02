#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
bb_baseline
post /projects 'key=WEB&name=Web' >/dev/null
post /projects 'key=API&name=Api' >/dev/null
post /p/WEB/labels name=bug >/dev/null
post /p/WEB/issues 'title=One&labels=bug' >/dev/null
post /p/WEB/issues 'title=Two' >/dev/null
post /p/WEB/issues 'title=Three' >/dev/null
post /p/API/issues 'title=Api+one' >/dev/null

# Creating milestones.
expect_eq "$(get /p/WEB/milestones.txt)" "" "no milestones yet"
expect_eq "$(ctype /p/WEB/milestones.txt)" 'text/plain; charset=utf-8' "milestones.txt content type"
expect_has "$(post /p/WEB/milestones name=v1.0)" '<h1>WEB: Web</h1>' "create answers the project page"
post /p/WEB/milestones name=alpha >/dev/null
post /p/WEB/milestones 'name=+Beta+' >/dev/null
post /p/WEB/milestones 'name=release_2026-11.rc_1' >/dev/null
expect_err 422 "name required" /p/WEB/milestones name=
expect_err 422 "name required" /p/WEB/milestones 'name=+++'
expect_err 422 "bad milestone" /p/WEB/milestones 'name=v+1'
expect_err 422 "bad milestone" /p/WEB/milestones 'name=v1%2F2'
expect_err 422 "bad milestone" /p/WEB/milestones 'name=%C3%A9t%C3%A9'
expect_err 422 "bad milestone" /p/WEB/milestones "name=release_2026-11.rc_12"
expect_err 422 "milestone exists" /p/WEB/milestones name=v1.0
expect_err 404 "no such project: NOPE" /p/NOPE/milestones name=x
expect_err 404 "no such project: NOPE" /p/NOPE/milestones.txt
expect_err 404 "not found" /p/WEB/milestones
expect_has "$(post /p/API/milestones name=v1.0)" '<h1>API: Api</h1>' "same name in another project"
expect_eq "$(get /p/WEB/milestones.txt)" $'Beta 0/0\nalpha 0/0\nrelease_2026-11.rc_1 0/0\nv1.0 0/0' "byte order"
expect_eq "$(get /p/API/milestones.txt)" 'v1.0 0/0' "API milestones"
expect_eq "$(get /p/BASE/milestones.txt)" "" "BASE milestones"

# Putting issues in milestones.
r="$(post /p/WEB/1/milestone milestone=v1.0)"
expect_has "$r" '<p class="meta">Status: <span class="status">open</span> | Priority: P3 | Assignee: - | Labels: bug | Milestone: v1.0</p>' "issue meta"
expect_has "$r" '<button>remove label</button></form><form method="post" action="/p/WEB/1/milestone"><input name="milestone" value="v1.0"> <button>set milestone</button></form>' "milestone form"
post /p/WEB/2/milestone milestone=v1.0 >/dev/null
post /p/WEB/3/milestone milestone=alpha >/dev/null
post /p/WEB/2/status status=closed >/dev/null
post /p/API/1/milestone milestone=v1.0 >/dev/null
expect_eq "$(get /p/WEB/milestones.txt)" $'Beta 0/0\nalpha 1/1\nrelease_2026-11.rc_1 0/0\nv1.0 1/2' "WEB counts"
expect_eq "$(get /p/API/milestones.txt)" 'v1.0 1/1' "API counts are per project"
expect_err 422 "no such milestone: nope" /p/WEB/1/milestone milestone=nope
expect_err 422 "no such milestone: Alpha" /p/WEB/1/milestone milestone=Alpha
expect_err 422 "no such milestone: alpha" /p/API/1/milestone milestone=alpha
expect_err 404 "no such issue: WEB-9" /p/WEB/9/milestone milestone=v1.0
expect_err 404 "no such project: NOPE" /p/NOPE/1/milestone milestone=v1.0
expect_err 404 "not found" /p/WEB/1/milestone
expect_eq "$(get /p/WEB/1.json)" '{"key":"WEB-1","project":"WEB","number":1,"title":"One","body":"","status":"open","priority":3,"assignee":null,"labels":["bug"],"milestone":"v1.0","comments":[],"activity":[{"kind":"created","detail":""},{"kind":"milestone","detail":"v1.0"}]}' "json with milestone"
expect_has "$(get /p/BASE/1)" 'Labels: bug, ui | Milestone: -</p>' "meta without milestone"
expect_has "$(get /p/BASE/2.json)" '"labels":[],"milestone":null,"comments":[]' "json without milestone"

# Activity only on change.
post /p/WEB/1/milestone milestone=v1.0 >/dev/null
expect_eq "$(get /p/WEB/1/activity.txt)" $'created\nmilestone v1.0' "no row for the same milestone"
post /p/WEB/1/milestone milestone=alpha >/dev/null
post /p/WEB/1/milestone milestone= >/dev/null
post /p/WEB/1/milestone milestone= >/dev/null
expect_eq "$(get /p/WEB/1/activity.txt)" $'created\nmilestone v1.0\nmilestone alpha\nmilestone -' "milestone activity"
r="$(get /p/WEB/1)"
expect_has "$r" '| Labels: bug | Milestone: -</p>' "cleared meta"
expect_has "$r" '<input name="milestone" value=""> <button>set milestone</button>' "cleared form"
expect_has "$(get /p/WEB/1.json)" '"labels":["bug"],"milestone":null,' "cleared json"

# Unchanged formats.
expect_eq "$(get /p/WEB/issues.txt)" $'WEB-1 open P3 - [bug] One\nWEB-2 closed P3 - [] Two\nWEB-3 open P3 - [] Three' "issues.txt unchanged"
getf /p/WEB/issues.csv "$RUN/web.csv"
expect_file "$RUN/web.csv" 'key,title,status,priority,assignee,labels,body\nWEB-1,One,open,3,,bug,\nWEB-2,Two,closed,3,,,\nWEB-3,Three,open,3,,,\n'

bb_restart
expect_eq "$(get /p/WEB/milestones.txt)" $'Beta 0/0\nalpha 1/1\nrelease_2026-11.rc_1 0/0\nv1.0 0/1' "milestones persist"
expect_has "$(get /p/WEB/3.json)" '"milestone":"alpha",' "issue milestone persists"
expect_eq "$(get /p/WEB/1/activity.txt)" $'created\nmilestone v1.0\nmilestone alpha\nmilestone -' "activity persists"
expect_err 422 "milestone exists" /p/WEB/milestones name=alpha
post /p/WEB/2/status status=open >/dev/null
expect_eq "$(get /p/WEB/milestones.txt | tail -1)" 'v1.0 1/1' "counts follow status"
bb_unit
echo PASS
