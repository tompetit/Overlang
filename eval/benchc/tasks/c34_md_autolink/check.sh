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
a() { printf '<a href="%s">%s</a>' "$1" "$1"; }
post /projects 'key=MD&name=Markdown' >/dev/null

# The old rules are untouched (spec 2.5 examples, as far as the form keeps the input).
expect_md $'a **b** `c<d>` [x](http://e.com)\n\nnext' $'<p>a <strong>b</strong> <code>c&lt;d&gt;</code> <a href="http://e.com">x</a></p>\n<p>next</p>'
expect_md '[x](javascript:alert(1)) **open' '<p>[x](javascript:alert(1)) **open</p>'
expect_md '<script>' '<p>&lt;script&gt;</p>'
expect_md $'line1\nline2' $'<p>line1\nline2</p>'
expect_md '' ''
expect_md '**a [l](/x) b**' '<p><strong>a <a href="/x">l</a> b</strong></p>'

# The statement's examples.
expect_p 'see https://e.com/a?x=1&y=2.' "see $(a 'https://e.com/a?x=1&amp;y=2')."
expect_p '(docs: http://e.com/x)!' "(docs: $(a http://e.com/x))!"
expect_p 'http://e.com/"onclick="x' "$(a http://e.com/)&quot;onclick=&quot;x"
expect_p 'xhttp://e.com HTTP://E.COM http://' 'xhttp://e.com HTTP://E.COM http://'

# Where a URL ends.
expect_md $'http://a.b/1\nhttps://a.b/2\thttp://a.b/3 end' "<p>$(a http://a.b/1)"$'\n'"$(a https://a.b/2)"$'\t'"$(a http://a.b/3) end</p>"
expect_p "http://a.b/x<y http://a.b/'q' http://a.b/>" "$(a http://a.b/x)&lt;y $(a http://a.b/)&#39;q&#39; $(a http://a.b/)&gt;"
expect_p 'http://a.b/?p=1&q=&lt;&amp; done' "$(a 'http://a.b/?p=1&amp;q=&amp;lt;&amp;amp;') done"
expect_p 'http://a.b/a;b,c;' "$(a 'http://a.b/a;b,c;')"
expect_p 'go to http://a.b/x.y.?!:,). now' "go to $(a http://a.b/x.y).?!:,). now"
expect_p 'https://.,) http://)' 'https://.,) http://)'
expect_p 'https://x' "$(a https://x)"
expect_p 'http://a.b/**x**' "$(a 'http://a.b/**x**')"
expect_p 'http://a.b/[x](y)' "$(a 'http://a.b/[x](y'))"
# What may come before it.
expect_p '1http://a.b Zhttps://a.b _http://a.b -http://a.b /https://a.b' "1http://a.b Zhttps://a.b _$(a http://a.b) -$(a http://a.b) /$(a https://a.b)"
expect_p $'\xc3\xa9http://a.b \xe2\x80\x9chttp://a.b\xe2\x80\x9d' $'\xc3\xa9'"$(a http://a.b)"$' \xe2\x80\x9c'"$(a $'http://a.b\xe2\x80\x9d')"
expect_p 'http://a.bhttp://c.d' "$(a http://a.bhttp://c.d)"
expect_p 'httpss://a.b ftp://a.b javascript:alert(1) Http://a.b' 'httpss://a.b ftp://a.b javascript:alert(1) Http://a.b'
# Inside code, links and bold.
expect_p '`http://a.b` [http://a.b](http://c.d) [x](http://e.f)' '<code>http://a.b</code> <a href="http://c.d">http://a.b</a> <a href="http://e.f">x</a>'
expect_p '**see http://a.b/x**, then http://c.d.' "<strong>see $(a http://a.b/x)</strong>, then $(a http://c.d)."
expect_p '**http://a.b/**x' "<strong>$(a http://a.b/)</strong>x"

# Hostile URLs cannot leave the attribute or open a tag.
expect_p 'http://a.b/"><script>alert(1)</script>' "$(a http://a.b/)&quot;&gt;&lt;script&gt;alert(1)&lt;/script&gt;"
expect_p "https://a.b/' onmouseover='alert(1)" "$(a https://a.b/)&#39; onmouseover=&#39;alert(1)"
expect_p 'http://a.b/<img/src=x/onerror=alert(1)>' "$(a http://a.b/)&lt;img/src=x/onerror=alert(1)&gt;"
expect_p 'javascript:http://a.b' "javascript:$(a http://a.b)"
page="$(get "/p/MD/$n")"
expect_lacks "$page" '<img' "raw img tag"
expect_lacks "$page" '<script' "raw script tag"
expect_lacks "$(get "/p/MD/$((n - 2))")" "onmouseover='" "raw quote"

# Raw formats keep the source.
expect_has "$(get /p/MD/7.json)" '"body":"see https://e.com/a?x=1&y=2."' "json body raw"
expect_has "$(get /p/MD/issues.csv)" 'MD-7,md,open,3,,,see https://e.com/a?x=1&y=2.' "csv body raw"
expect_eq "$(get '/p/MD/issues.txt?q=e.com%2Fx')" 'MD-8 open P3 - [] md' "search still sees the raw body"

# Comments autolink too.
expect_code 200 /p/MD/8/comments "author=alice&body=$(enc $'log: https://ci.example/run/42.\nok')"
expect_eq "$(comments_of /p/MD/8)" "<li><b>alice</b> <p>log: $(a https://ci.example/run/42)."$'\n'"ok</p></li>" "comment"
expect_eq "$(get /p/MD/8/comments.txt)" 'alice: log: https://ci.example/run/42. ok' "comments.txt raw"

# Editing the body re-renders.
expect_code 200 /p/MD/1/edit "body=$(enc 'now at https://e.com/new')"
expect_eq "$(body_of /p/MD/1)" "<p>now at $(a https://e.com/new)</p>" "edited body"

bb_restart
expect_eq "$(body_of /p/MD/1)" "<p>now at $(a https://e.com/new)</p>" "after restart"
expect_eq "$(body_of /p/MD/7)" "<p>see $(a 'https://e.com/a?x=1&amp;y=2').</p>" "after restart, example"
expect_eq "$(get /p/BASE/1.txt)" 'BASE-1 closed P1 alice [bug,ui] Login fails' "baseline after restart"
bb_unit
echo PASS
