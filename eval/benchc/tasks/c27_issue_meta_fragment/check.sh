#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
bb_baseline
meta() { printf '<p class="meta">Status: <span class="status">%s</span> | Priority: P%s | Assignee: %s | Labels: %s</p>' "$@"; }
# expect_meta N STATUS PRIORITY ASSIGNEE LABELS: the fragment is exact and the page embeds it in place.
expect_meta() {
  local n="$1"; shift; local want; want="$(meta "$@")"
  getf "/p/BASE/$n/meta" "$RUN/frag"; printf '%s' "$want" >"$RUN/want"
  cmp -s "$RUN/frag" "$RUN/want" || fail "/p/BASE/$n/meta: want [$want] got [$(cat "$RUN/frag")]"
  expect_has "$(get "/p/BASE/$n")" "</h1>${want}<div class=\"body\">" "issue page $n embeds the fragment"
}
expect_meta 1 closed 1 alice "bug, ui"
expect_meta 2 open 3 bob -
expect_code 200 /p/BASE/1/meta
expect_eq "$(ctype /p/BASE/1/meta)" "text/html; charset=utf-8" "content type"

post /p/BASE/issues 'title=Third' >/dev/null
expect_meta 3 open 3 - -
expect_has "$(get /p/BASE/3)" '<h1>BASE-3: Third</h1><p class="meta">' "page start"
expect_has "$(get /p/BASE/3)" '</p><div class="body"></div><form method="post" action="/p/BASE/3/status">' "page body"
post /p/BASE/3/assign assignee=alice >/dev/null
post /p/BASE/3/labels label=ui >/dev/null
expect_meta 3 open 3 alice ui
post /p/BASE/3/labels label=bug >/dev/null
post /p/BASE/3/edit priority=4 >/dev/null
post /p/BASE/3/status status=closed >/dev/null
expect_meta 3 closed 4 alice "bug, ui"
post /p/BASE/3/labels/remove label=bug >/dev/null
post /p/BASE/3/assign assignee= >/dev/null
post /p/BASE/3/status status=open >/dev/null
expect_meta 3 open 4 - ui

# same fragment with a trailing slash or a query
getf /p/BASE/3/meta "$RUN/a"; getf /p/BASE/3/meta/ "$RUN/b"; getf '/p/BASE/3/meta?x=1&status=closed' "$RUN/c"
cmp -s "$RUN/a" "$RUN/b" || fail "trailing slash changes the fragment"
cmp -s "$RUN/a" "$RUN/c" || fail "query changes the fragment"

# errors
expect_err 404 "no such project: NOPE" /p/NOPE/1/meta
expect_err 404 "no such issue: BASE-9" /p/BASE/9/meta
expect_eq "$(ctype /p/BASE/9/meta)" "text/plain; charset=utf-8" "error content type"
expect_err 404 "not found" /p/BASE/0/meta
expect_err 404 "not found" /p/BASE/x/meta
expect_err 404 "not found" /p/BASE/1.json/meta
expect_err 404 "not found" /p/BASE/1/meta/x
expect_err 404 "not found" /p/BASE/1/metas
expect_err 404 "not found" /p/BASE/1/meta x=1
expect_code 404 /p/BASE/meta
expect_eq "$(get /p/BASE/3.txt)" "BASE-3 open P4 - [ui] Third" "issue txt"

bb_restart
expect_meta 3 open 4 - ui
expect_meta 1 closed 1 alice "bug, ui"
bb_unit
echo PASS
