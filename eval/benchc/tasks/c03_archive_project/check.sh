#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
bb_baseline   # project BASE: BASE-1 closed (bug,ui, comment by bob), BASE-2 assigned to bob

expect_code 200 /projects 'key=OLD&name=Old+site'
expect_code 200 /projects 'key=API&name=Api'
expect_code 200 /p/OLD/issues 'title=Legacy'
expect_eq "$(get /projects.txt)" $'API Api\nBASE Base line\nOLD Old site' "projects.txt"
home="$(get /)"
expect_has "$home" '<ul class="projects"><li><a href="/p/API">API</a> Api</li><li><a href="/p/BASE">BASE</a> Base line</li><li><a href="/p/OLD">OLD</a> Old site</li></ul><form method="post" action="/projects">' "home, none archived"
expect_lacks "$home" 'Archived' "no archived section"
page="$(get /p/BASE)"
expect_has "$page" '<form method="post" action="/p/BASE/archive"><button>archive</button></form></body></html>' "archive form"
expect_lacks "$page" 'class="archived"' "active project page"
expect_has "$(get /p/BASE/1.json)" '{"key":"BASE-1","project":"BASE","archived":false,"number":1,"title":"Login fails",' "json active"

# Archive OLD (twice: no-op).
page="$(post /p/OLD/archive '')"
expect_has "$page" '<h1>OLD: Old site</h1><p class="archived">This project is archived.</p>' "archive answers project page"
expect_has "$page" '<form method="post" action="/p/OLD/unarchive"><button>unarchive</button></form></body></html>' "unarchive form"
expect_code 200 /p/OLD/archive ''
expect_eq "$(get /projects.txt)" $'API Api\nBASE Base line\nOLD Old site (archived)' "projects.txt archived"
expect_has "$(get /)" '<ul class="projects"><li><a href="/p/API">API</a> Api</li><li><a href="/p/BASE">BASE</a> Base line</li></ul><h2>Archived</h2><ul class="archived"><li><a href="/p/OLD">OLD</a> Old site</li></ul><form method="post" action="/projects">' "home with archived"
expect_err 422 "project exists" /projects 'key=OLD&name=x'
expect_err 404 "no such project: NOPE" /p/NOPE/archive ''
expect_err 404 "no such project: NOPE" /p/NOPE/unarchive ''
expect_err 404 "not found" /p/OLD/archive
expect_err 404 "not found" /p/OLD/archive/x ''

# Archive BASE: every write is refused, in the stated check order; reads still work.
before_txt="$(get /p/BASE/issues.txt)"; before_act="$(get /p/BASE/1/activity.txt)"
before_com="$(get /p/BASE/1/comments.txt)"; before_lab="$(get /p/BASE/labels.txt)"
before_csv="$(get /p/BASE/issues.csv)"
expect_code 200 /p/BASE/archive ''
expect_err 422 "project archived" /p/BASE/issues 'title=New'
expect_err 422 "project archived" /p/BASE/issues 'title='
expect_err 422 "project archived" /p/BASE/issues 'title=x&priority=9'
expect_err 422 "project archived" /p/BASE/labels 'name=new'
expect_err 422 "project archived" /p/BASE/labels 'name='
expect_err 422 "project archived" /p/BASE/1/status 'status=open'
expect_err 422 "project archived" /p/BASE/1/status 'status=bogus'
expect_err 422 "project archived" /p/BASE/1/edit 'title=Renamed'
expect_err 422 "project archived" /p/BASE/1/edit 'priority=x'
expect_err 422 "project archived" /p/BASE/2/assign 'assignee=alice'
expect_err 422 "project archived" /p/BASE/2/assign 'assignee=zed'
expect_err 422 "project archived" /p/BASE/1/comments 'author=alice&body=hi'
expect_err 422 "project archived" /p/BASE/1/comments 'author=&body='
expect_err 422 "project archived" /p/BASE/2/labels 'label=bug'
expect_err 422 "project archived" /p/BASE/1/labels/remove 'label=bug'
expect_err 422 "project archived" /p/BASE/1/labels/remove 'label=nope'
expect_err 404 "no such issue: BASE-9" /p/BASE/9/status 'status=closed'
expect_err 404 "no such issue: BASE-9" /p/BASE/9/comments 'author=bob&body=x'
expect_err 404 "no such project: NOPE" /p/NOPE/issues 'title=x'
expect_eq "$(get /p/BASE/issues.txt)" "$before_txt" "issues unchanged"
expect_eq "$(get /p/BASE/1/activity.txt)" "$before_act" "activity unchanged"
expect_eq "$(get /p/BASE/1/comments.txt)" "$before_com" "comments unchanged"
expect_eq "$(get /p/BASE/labels.txt)" "$before_lab" "labels unchanged"
expect_eq "$(get /p/BASE/issues.csv)" "$before_csv" "csv unchanged"
expect_eq "$(get /p/BASE/1.txt)" 'BASE-1 closed P1 alice [bug,ui] Login fails' "issue txt"
expect_eq "$(get '/p/BASE/issues.txt?status=open')" 'BASE-2 open P3 bob [] Typo on home' "filter on archived"
expect_has "$(get /p/BASE/1.json)" '{"key":"BASE-1","project":"BASE","archived":true,"number":1,"title":"Login fails",' "json archived"
expect_has "$(get /p/BASE/1)" '<h1>BASE-1: Login fails</h1><p class="archived">This project is archived.</p><p class="meta">' "issue page archived"
expect_has "$(get '/p/BASE?status=closed')" '<h1>BASE: Base line</h1><p class="archived">This project is archived.</p><form method="get"' "project page archived"
expect_eq "$(get /projects.txt)" $'API Api\nBASE Base line (archived)\nOLD Old site (archived)' "projects.txt two archived"
expect_has "$(get /)" '<ul class="projects"><li><a href="/p/API">API</a> Api</li></ul><h2>Archived</h2><ul class="archived"><li><a href="/p/BASE">BASE</a> Base line</li><li><a href="/p/OLD">OLD</a> Old site</li></ul>' "home two archived"

# Other projects stay writable.
expect_code 200 /p/API/labels 'name=bug'
expect_code 200 /p/API/issues 'title=Api+one&labels=bug&assignee=alice'
expect_eq "$(get /p/API/issues.txt)" 'API-1 open P3 alice [bug] Api one' "active project writable"
expect_lacks "$(get /p/API/1)" 'class="archived"' "active issue page"

# Unarchive BASE (twice: no-op): writes work again.
page="$(post /p/BASE/unarchive '')"
expect_lacks "$page" 'class="archived"' "unarchived page"
expect_has "$page" 'action="/p/BASE/archive"><button>archive</button></form></body>' "archive form back"
expect_code 200 /p/BASE/unarchive ''
expect_code 200 /p/BASE/2/status 'status=closed'
expect_code 200 /p/BASE/issues 'title=After'
expect_eq "$(get /p/BASE/issues.txt)" $'BASE-1 closed P1 alice [bug,ui] Login fails\nBASE-2 closed P3 bob [] Typo on home\nBASE-3 open P3 - [] After' "writes after unarchive"
expect_eq "$(get /p/BASE/2/activity.txt)" $'created\nassigned bob\nstatus closed' "activity after unarchive"

# Persistence.
bb_restart
expect_eq "$(get /projects.txt)" $'API Api\nBASE Base line\nOLD Old site (archived)' "projects.txt after restart"
expect_err 422 "project archived" /p/OLD/1/status 'status=closed'
expect_err 422 "project archived" /p/OLD/labels 'name=x'
expect_eq "$(get /p/OLD/issues.txt)" 'OLD-1 open P3 - [] Legacy' "archived issues readable"
expect_code 200 /p/BASE/labels 'name=new'
bb_unit
echo PASS
