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
post /projects 'key=MD&name=Markdown' >/dev/null

# The old rules are untouched (spec 2.5 examples, as far as the form keeps the input).
expect_md $'a **b** `c<d>` [x](http://e.com)\n\nnext' $'<p>a <strong>b</strong> <code>c&lt;d&gt;</code> <a href="http://e.com">x</a></p>\n<p>next</p>'
expect_md '[x](javascript:alert(1)) **open' '<p>[x](javascript:alert(1)) **open</p>'
expect_md '<script>' '<p>&lt;script&gt;</p>'
expect_md $'line1\nline2\n   indented' $'<p>line1\nline2\n   indented</p>'
expect_md '' ''
expect_md '**a [l](/x) b**' '<p><strong>a <a href="/x">l</a> b</strong></p>'

# The statement's example.
expect_md $'Steps:\n1. open the page\n2. click **Login**\n   (twice)\n- it fails\n* with <b>500</b>\n\ndone' \
  $'<p>Steps:</p>\n<ol><li>open the page</li><li>click <strong>Login</strong>\n(twice)</li></ol>\n<ul><li>it fails</li><li>with &lt;b&gt;500&lt;/b&gt;</li></ul>\n<p>done</p>'
# Two lists split by a blank line; start numbers; later numbers ignored.
expect_md $'- a\n- b\n\n- c' $'<ul><li>a</li><li>b</li></ul>\n<ul><li>c</li></ul>'
expect_md $'3. three\n1. one\n\n007. seven\n\n0. zero\n10. ten' $'<ol start="3"><li>three</li><li>one</li></ol>\n<ol start="7"><li>seven</li></ol>\n<ol start="0"><li>zero</li><li>ten</li></ol>'
expect_md $'1. x\n- y\n2. z' $'<ol><li>x</li></ol>\n<ul><li>y</li></ul>\n<ol start="2"><li>z</li></ol>'
expect_md '123456789. max' '<ol start="123456789"><li>max</li></ol>'
# Not items.
expect_md $'-x\n**bold** text\n1.x\n1) x\n1234567890. ten digits\n -  indented\n+ plus' $'<p>-x\n<strong>bold</strong> text\n1.x\n1) x\n1234567890. ten digits\n -  indented\n+ plus</p>'
expect_md $'para\n-    \nmore\n1.   \n*' $'<p>para\n-    \nmore\n1.   \n*</p>'
# Paragraph then list then paragraph, no blank lines; continuation vs. indented paragraph line.
expect_md $'intro\n- a\n  - not nested\n  **x\n  y**\nafter\n  still after' $'<p>intro</p>\n<ul><li>a\n- not nested\n<strong>x\ny</strong></li></ul>\n<p>after\n  still after</p>'
expect_md $'- a\n\n  b' $'<ul><li>a</li></ul>\n<p>  b</p>'
expect_md $'-   spaced   \n *  not an item\n* `- code`' $'<ul><li>spaced</li></ul>\n<p> *  not an item</p>\n<ul><li><code>- code</code></li></ul>'
expect_md $'- [ok](http://a.com) and [x](javascript:alert(1))\n1. **open' $'<ul><li><a href="http://a.com">ok</a> and [x](javascript:alert(1))</li></ul>\n<ol><li>**open</li></ol>'

# Hostile items are escaped.
expect_md $'- <img src=x onerror=alert(1)>\n- </li></ul><script>alert(1)</script>\n1. "\' & <b>' \
  $'<ul><li>&lt;img src=x onerror=alert(1)&gt;</li><li>&lt;/li&gt;&lt;/ul&gt;&lt;script&gt;alert(1)&lt;/script&gt;</li></ul>\n<ol><li>&quot;&#39; &amp; &lt;b&gt;</li></ol>'
page="$(get "/p/MD/$n")"
expect_lacks "$page" '<img' "raw img tag"
expect_lacks "$page" '<script' "raw script tag"

# Raw formats keep the source.
expect_has "$(get /p/MD/8.json)" '"body":"- a\n- b\n\n- c"' "json body raw"
expect_eq "$(get /p/MD/8.txt)" 'MD-8 open P3 - [] md' "issue txt"

# Comments render lists too.
expect_code 200 /p/MD/8/comments "author=alice&body=$(enc $'Repro:\n1. log in\n2. log out')"
expect_code 200 /p/MD/8/comments "author=bob&body=$(enc $'* yes')"
expect_eq "$(comments_of /p/MD/8)" '<li><b>alice</b> <p>Repro:</p>
<ol><li>log in</li><li>log out</li></ol></li><li><b>bob</b> <ul><li>yes</li></ul></li>' "comments"
expect_eq "$(get /p/MD/8/comments.txt)" $'alice: Repro: 1. log in 2. log out\nbob: * yes' "comments.txt raw"
expect_err 422 "no such user: zed" /p/MD/8/comments 'author=zed&body=-+x'

# Editing the body re-renders.
expect_code 200 /p/MD/1/edit "body=$(enc $'- x\n- y')"
expect_eq "$(body_of /p/MD/1)" '<ul><li>x</li><li>y</li></ul>' "edited body"

bb_restart
expect_eq "$(body_of /p/MD/1)" '<ul><li>x</li><li>y</li></ul>' "after restart"
expect_eq "$(body_of /p/MD/9)" $'<ol start="3"><li>three</li><li>one</li></ol>\n<ol start="7"><li>seven</li></ol>\n<ol start="0"><li>zero</li><li>ten</li></ol>' "after restart, numbered"
expect_eq "$(get /p/BASE/1.txt)" 'BASE-1 closed P1 alice [bug,ui] Login fails' "baseline after restart"
bb_unit
echo PASS
