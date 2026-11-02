#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
bb_baseline

# body_of PATH: the rendered issue body (inside <div class="body">) of the page at PATH.
body_of() { get "$1" | python3 -c 'import sys
t = sys.stdin.buffer.read().decode(); a = t.find("<div class=\"body\">")
if a >= 0: a += 18; sys.stdout.buffer.write(t[a:t.find("</div>", a)].encode())'; }
# comments_of PATH: everything inside <ul class="comments"> of the page at PATH.
comments_of() { get "$1" | python3 -c 'import sys
t = sys.stdin.buffer.read().decode(); m = "<ul class=\"comments\">"; a = t.find(m)
if a >= 0: a += len(m); sys.stdout.buffer.write(t[a:t.find("</ul><form method=\"post\" action=\"/p/", a)].encode())'; }
# expect_md SRC WANT: a new MD issue with body SRC renders exactly WANT.
n=0
expect_md() {
  expect_code 200 /p/MD/issues "title=md&body=$(enc "$1")"; n=$((n + 1))
  expect_eq "$(body_of "/p/MD/$n")" "$2" "md #$n [$1]"
}
# expect_p SRC WANT: as expect_md, for a one-paragraph SRC whose md_inline is WANT.
expect_p() { expect_md "$1" "<p>$2</p>"; }
# r KEY N CLASS ESCAPED_TITLE: the expected link.
r() { printf '<a href="/p/%s/%s" class="%s" title="%s">%s-%s</a>' "$1" "$2" "$3" "$4" "$1" "$2"; }

post /projects 'key=MD&name=Markdown' >/dev/null
for k in WEB API ABCDEFGHIJ W1; do post /projects "key=$k&name=$k" >/dev/null; done
expect_code 200 /p/WEB/issues "title=$(enc '"><script>alert(1)</script>')"
expect_code 200 /p/WEB/issues "title=$(enc $'Tom\'s & "Jerry"')"
expect_code 200 /p/API/issues 'title=Rate+limit'
post /p/API/1/status status=closed >/dev/null
expect_code 200 /p/ABCDEFGHIJ/issues 'title=Ten'
expect_code 200 /p/W1/issues 'title=Short'
X='&quot;&gt;&lt;script&gt;alert(1)&lt;/script&gt;'   # WEB-1's title, escaped
T='Tom&#39;s &amp; &quot;Jerry&quot;'                 # WEB-2's title, escaped
W2="$(r WEB 2 issue-ref "$T")"

# The old rules are untouched (spec 2.5 examples, as far as the form keeps the input).
expect_md $'a **b** `c<d>` [x](http://e.com)\n\nnext' $'<p>a <strong>b</strong> <code>c&lt;d&gt;</code> <a href="http://e.com">x</a></p>\n<p>next</p>'
expect_md '[x](javascript:alert(1)) **open' '<p>[x](javascript:alert(1)) **open</p>'
expect_md '<script>' '<p>&lt;script&gt;</p>'
expect_md $'line1\nline2' $'<p>line1\nline2</p>'
expect_md '' ''
expect_md '**a [l](/x) b**' '<p><strong>a <a href="/x">l</a> b</strong></p>'

# References.
expect_p 'See WEB-2 and BASE-1.' "See $W2 and $(r BASE 1 'issue-ref closed' 'Login fails')."
expect_p 'WEB-1' "$(r WEB 1 issue-ref "$X")"
expect_p $'API-1\nBASE-2' "$(r API 1 'issue-ref closed' 'Rate limit')"$'\n'"$(r BASE 2 issue-ref 'Typo on home')"
expect_p '(WEB-2), [BASE-2]: WEB-2-3 WEB-2. "WEB-2"' "($W2), [$(r BASE 2 issue-ref 'Typo on home')]: $W2-3 $W2. &quot;$W2&quot;"
expect_p 'ABCDEFGHIJ-1 XABCDEFGHIJ-1 W1-1' "$(r ABCDEFGHIJ 1 issue-ref Ten) XABCDEFGHIJ-1 $(r W1 1 issue-ref Short)"
expect_p $'\xc3\xa9WEB-2 WEB-2\xc3\xa9' $'\xc3\xa9'"$W2 $W2"$'\xc3\xa9'
# Not references.
expect_p 'WEB-99 NOPE-1 web-1 Web-1 WEB-0 WEB-01 WEB- WEB-1x WEB-1_ WEB-2WEB-2' 'WEB-99 NOPE-1 web-1 Web-1 WEB-0 WEB-01 WEB- WEB-1x WEB-1_ WEB-2WEB-2'
expect_p 'xWEB-1 1WEB-1 _WEB-1 -WEB-1 /WEB-1 /p/WEB/1 W-1 WEB--1 WEB-1234567890' 'xWEB-1 1WEB-1 _WEB-1 -WEB-1 /WEB-1 /p/WEB/1 W-1 WEB--1 WEB-1234567890'
# Inside code, links and bold.
expect_p '`WEB-2` [WEB-2](/p/WEB/2) [x](http://e.com/WEB-2) **WEB-2**' '<code>WEB-2</code> <a href="/p/WEB/2">WEB-2</a> <a href="http://e.com/WEB-2">x</a> <strong>'"$W2"'</strong>'
# A self-reference, and a reference to an issue that does not exist yet.
expect_p 'me: MD-16, next: MD-17' "me: $(r MD 16 issue-ref md), next: MD-17"
expect_code 200 /p/MD/issues 'title=Seventeen'; n=$((n + 1))
expect_eq "$(body_of /p/MD/16)" "<p>me: $(r MD 16 issue-ref md), next: $(r MD 17 issue-ref Seventeen)</p>" "issue created later"

# Hostile text around references stays escaped.
expect_p '<b>WEB-2</b> "onmouseover=WEB-2' "&lt;b&gt;$W2&lt;/b&gt; &quot;onmouseover=$W2"
page="$(get /p/MD/7)"
expect_lacks "$(get /p/MD/8)" '<script' "raw script tag from a title"
expect_has "$page" "<h1>MD-7: md</h1>" "page heading"

# Titles are not markdown: no links in headings, tables or text formats.
expect_code 200 /p/MD/issues "title=$(enc 'Dup of WEB-2')&body=$(enc 'Dup of WEB-2')"; n=$((n + 1))
page="$(get "/p/MD/$n")"
expect_has "$page" "<title>MD-$n: Dup of WEB-2</title>" "page title raw"
expect_has "$page" "<h1>MD-$n: Dup of WEB-2</h1>" "heading raw"
expect_eq "$(body_of "/p/MD/$n")" "<p>Dup of $W2</p>" "body linked"
expect_has "$(get /p/MD)" "<td>Dup of WEB-2</td>" "table title raw"
expect_eq "$(get "/p/MD/$n.txt")" "MD-$n open P3 - [] Dup of WEB-2" "issue txt"
expect_has "$(get "/p/MD/$n.json")" '"body":"Dup of WEB-2"' "json raw"
expect_eq "$(get '/p/MD/issues.txt?q=WEB-2%22')" 'MD-10 open P3 - [] md' "search sees raw body"

# Comments link references too.
expect_code 200 /p/MD/7/comments "author=bob&body=$(enc $'blocked by API-1\n`API-1`')"
expect_eq "$(comments_of /p/MD/7)" "<li><b>bob</b> <p>blocked by $(r API 1 'issue-ref closed' 'Rate limit')"$'\n'"<code>API-1</code></p></li>" "comment"
expect_eq "$(get /p/MD/7/comments.txt)" 'bob: blocked by API-1 `API-1`' "comments.txt raw"

# The link follows the referenced issue's current state.
post /p/WEB/2/status status=closed >/dev/null
post /p/WEB/2/edit "title=$(enc 'Renamed <again>')" >/dev/null
post /p/API/1/status status=open >/dev/null
W2C="$(r WEB 2 'issue-ref closed' 'Renamed &lt;again&gt;')"
expect_eq "$(body_of /p/MD/7)" "<p>See $W2C and $(r BASE 1 'issue-ref closed' 'Login fails').</p>" "after close and rename"
expect_eq "$(comments_of /p/MD/7)" "<li><b>bob</b> <p>blocked by $(r API 1 issue-ref 'Rate limit')"$'\n'"<code>API-1</code></p></li>" "comment after reopen"
expect_eq "$(get /p/WEB/2.txt)" 'WEB-2 closed P3 - [] Renamed <again>' "WEB-2 txt"

bb_restart
expect_eq "$(body_of /p/MD/7)" "<p>See $W2C and $(r BASE 1 'issue-ref closed' 'Login fails').</p>" "after restart"
expect_eq "$(body_of /p/MD/16)" "<p>me: $(r MD 16 issue-ref md), next: $(r MD 17 issue-ref Seventeen)</p>" "after restart, created later"
expect_eq "$(get /p/BASE/1.txt)" 'BASE-1 closed P1 alice [bug,ui] Login fails' "baseline after restart"
bb_unit
echo PASS
