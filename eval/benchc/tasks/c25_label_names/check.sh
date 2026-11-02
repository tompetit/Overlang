#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
bb_baseline
L1='BASE-1 closed P1 alice [bug,ui] Login fails'

# creating labels
expect_err 422 "label exists" /p/BASE/labels name=BUG
expect_err 422 "label exists" /p/BASE/labels 'name=+Ui+'
expect_err 422 "name required" /p/BASE/labels 'name=+%0D%0A+'
expect_err 422 "bad label" /p/BASE/labels 'name=a_b'
expect_err 422 "bad label" /p/BASE/labels 'name=Abcdefghij+Klmnopqrst'
expect_code 200 /p/BASE/labels 'name=Needs++Review'
expect_code 200 /p/BASE/labels 'name=WONT+FIX'
expect_code 200 /p/BASE/labels 'name=Abcdefghij+Klmnopqrs'
expect_code 200 /p/BASE/labels 'name=Line%0ABreak'
expect_code 200 /p/BASE/labels 'name=a+-+b'
expect_err 422 "label exists" /p/BASE/labels 'name=needs+review'
expect_eq "$(get /p/BASE/labels.txt)" $'a---b\nabcdefghij-klmnopqrs\nbug\nline-break\nneeds-review\nui\nwont-fix' "labels.txt"
expect_has "$(get /p/BASE)" '<p class="labels">a---b, abcdefghij-klmnopqrs, bug, line-break, needs-review, ui, wont-fix</p>' "project page"
# labels are per project
post /projects 'key=OPS&name=Ops' >/dev/null
expect_code 200 /p/OPS/labels name=Bug
expect_eq "$(get /p/OPS/labels.txt)" bug "OPS labels"
expect_err 422 "no such label: needs-review" /p/OPS/issues 'title=x&labels=Needs+Review'

# labels on create
expect_has "$(post /p/BASE/issues 'title=Review+me&labels=Needs+Review,+BUG+,,UI')" 'Labels: bug, needs-review, ui' "create page"
expect_eq "$(get /p/BASE/3.txt)" 'BASE-3 open P3 - [bug,needs-review,ui] Review me' "normalised labels"
expect_err 422 "no such label: wont-fixed" /p/BASE/issues 'title=x&labels=Wont+Fixed'
expect_err 422 "duplicate label: bug" /p/BASE/issues 'title=x&labels=bug,BUG,nope'
expect_err 422 "no such label: nope" /p/BASE/issues 'title=x&labels=bug,nope,BUG'
expect_err 422 "duplicate label: needs-review" /p/BASE/issues 'title=x&labels=needs-review,ui,+Needs++Review'
expect_err 422 "duplicate label: ui" /p/BASE/issues 'title=x&labels=ui,,ui'
expect_err 422 "no such user: zed" /p/BASE/issues 'title=x&assignee=zed&labels=bug,bug'
expect_err 422 "title required" /p/BASE/issues 'title=&labels=bug,bug'
expect_err 404 "no such issue: BASE-4" /p/BASE/4.txt

# add and remove on an issue
expect_code 200 /p/BASE/2/labels 'label=Wont+Fix'
expect_code 200 /p/BASE/2/labels 'label=+line%0Abreak+'
expect_code 200 /p/BASE/2/labels 'label=WONT++FIX'
expect_eq "$(get /p/BASE/2.txt)" 'BASE-2 open P3 bob [line-break,wont-fix] Typo on home' "added"
expect_err 422 "no such label: nope-nope" /p/BASE/2/labels 'label=Nope+Nope'
expect_err 422 "label required" /p/BASE/2/labels 'label=+'
expect_code 200 /p/BASE/2/labels/remove 'label=WONT+FIX'
expect_err 422 "no such label: nope" /p/BASE/2/labels/remove 'label=NOPE'
expect_err 422 "label required" /p/BASE/2/labels/remove 'label=%0A'
expect_eq "$(get /p/BASE/2.txt)" 'BASE-2 open P3 bob [line-break] Typo on home' "removed"
expect_code 200 /p/BASE/3/labels/remove 'label=Needs+Review'
expect_eq "$(get /p/BASE/3.txt)" 'BASE-3 open P3 - [bug,ui] Review me' "removed by normalised name"
expect_code 200 /p/BASE/3/labels 'label=A+-+B'
expect_eq "$(get /p/BASE/3.txt)" 'BASE-3 open P3 - [a---b,bug,ui] Review me' "a - b"

# label filter
expect_eq "$(get '/p/BASE/issues.txt?label=BUG')" "$L1"$'\nBASE-3 open P3 - [a---b,bug,ui] Review me' "filter BUG"
expect_eq "$(get '/p/BASE/issues.txt?label=+Line++Break+')" 'BASE-2 open P3 bob [line-break] Typo on home' "filter Line Break"
expect_eq "$(get '/p/BASE/issues.txt?label=Wont+Fix')" '' "filter with no match"
page="$(get '/p/BASE?label=Line+Break')"
expect_has "$page" '<input name="label" value="line-break" placeholder="label">' "filter form value"
expect_has "$page" 'Page 1 of 1 (1 issues)' "filtered count"

bb_restart
expect_eq "$(get /p/BASE/labels.txt)" $'a---b\nabcdefghij-klmnopqrs\nbug\nline-break\nneeds-review\nui\nwont-fix' "labels after restart"
expect_eq "$(get /p/BASE/issues.txt)" "$L1
BASE-2 open P3 bob [line-break] Typo on home
BASE-3 open P3 - [a---b,bug,ui] Review me" "issues after restart"
expect_err 422 "label exists" /p/BASE/labels 'name=Wont+Fix'
expect_err 422 "duplicate label: line-break" /p/BASE/issues 'title=x&labels=line-break,Line+Break'
bb_unit
echo PASS
