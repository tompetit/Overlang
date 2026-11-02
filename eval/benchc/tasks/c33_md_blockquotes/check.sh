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
B='<blockquote>'; E='</blockquote>'

# The old rules are untouched (spec 2.5 examples, as far as the form keeps the input).
expect_md $'a **b** `c<d>` [x](http://e.com)\n\nnext' $'<p>a <strong>b</strong> <code>c&lt;d&gt;</code> <a href="http://e.com">x</a></p>\n<p>next</p>'
expect_md '[x](javascript:alert(1)) **open' '<p>[x](javascript:alert(1)) **open</p>'
expect_md '<script>' '<p>&lt;script&gt;</p>'
expect_md $'line1\nline2\na > b' $'<p>line1\nline2\na &gt; b</p>'
expect_md '' ''
expect_md '**a [l](/x) b**' '<p><strong>a <a href="/x">l</a> b</strong></p>'

# The statement's example.
expect_md $'> alice wrote:\n>\n>> it is **broken**\nFixed in v2, a > b.' \
  "$B<p>alice wrote:</p>"$'\n'"$B<p>it is <strong>broken</strong></p>$E$E"$'\n<p>Fixed in v2, a &gt; b.</p>'
# Paragraph, quote, paragraph without blank lines; spaces after ">".
expect_md $'before\n>one\n> two\n>  three\nafter' "<p>before</p>"$'\n'"$B<p>one"$'\n'"two"$'\n'" three</p>$E"$'\n<p>after</p>'
# Two quotes split by a blank line; an empty quote; paragraphs inside a quote.
expect_md $'> a\n\n> b\n>\n>\n> c\n\n>\n> ' "$B<p>a</p>$E"$'\n'"$B<p>b</p>"$'\n'"<p>c</p>$E"$'\n'"$B$E"
expect_md $'x\n>\ny' $'<p>x</p>\n'"$B$E"$'\n<p>y</p>'
# Nesting with "> >", and a quote inside a quote between paragraphs.
expect_md $'> > deep\n> back\n>\n> > again' "$B$B<p>deep</p>$E"$'\n'"<p>back</p>"$'\n'"$B<p>again</p>$E$E"
# Not quote lines.
expect_md $'&gt; not a quote\n > indented\n a >> b' $'<p>&amp;gt; not a quote\n &gt; indented\n a &gt;&gt; b</p>'
# Inline markup inside quotes; it does not span the quote boundary.
expect_md $'> `<b>` [l](/p/MD/1) **x\n**y**' "$B<p><code>&lt;b&gt;</code> <a href=\"/p/MD/1\">l</a> **x</p>$E"$'\n<p><strong>y</strong></p>'
# Depth limit: at most 4 levels.
expect_md '>>>> four' "$B$B$B$B<p>four</p>$E$E$E$E"
expect_md '>>>>>> six' "$B$B$B$B<p>&gt;&gt; six</p>$E$E$E$E"
deep="$(printf '>%.0s' $(seq 300)) deep"
expect_md "$deep" "$B$B$B$B<p>$(printf '&gt;%.0s' $(seq 296)) deep</p>$E$E$E$E"
expect_md $'> > > > x\n> > > > > y' "$B$B$B$B<p>x"$'\n'"&gt; y</p>$E$E$E$E"

# Hostile quotes are escaped.
expect_md $'> <script>alert(1)</script>\n>> </blockquote><img src=x onerror=alert(1)>' \
  "$B<p>&lt;script&gt;alert(1)&lt;/script&gt;</p>"$'\n'"$B<p>&lt;/blockquote&gt;&lt;img src=x onerror=alert(1)&gt;</p>$E$E"
page="$(get "/p/MD/$n")"
expect_lacks "$page" '<img' "raw img tag"
expect_lacks "$page" '<script' "raw script tag"
expect_has "$page" '<textarea name="body">&gt; &lt;script&gt;alert(1)&lt;/script&gt;' "edit form shows the raw body escaped"

# Raw formats keep the source.
expect_has "$(get /p/MD/7.json)" '"body":"> alice wrote:\n>\n>> it is **broken**\nFixed in v2, a > b."' "json body raw"
expect_has "$(get /p/MD/issues.csv)" 'MD-7,md,open,3,,,"> alice wrote:' "csv body raw"

# Comments: quoting a reply.
expect_code 200 /p/MD/7/comments "author=alice&body=$(enc $'> Fixed in v2\n\nNo, still broken')"
expect_code 200 /p/MD/7/comments "author=bob&body=$(enc $'>> Fixed in v2\n> No, still broken\nnow?')"
expect_eq "$(comments_of /p/MD/7)" "<li><b>alice</b> $B<p>Fixed in v2</p>$E"$'\n'"<p>No, still broken</p></li><li><b>bob</b> $B$B<p>Fixed in v2</p>$E"$'\n'"<p>No, still broken</p>$E"$'\n'"<p>now?</p></li>" "comments"
expect_eq "$(get /p/MD/7/comments.txt)" $'alice: > Fixed in v2  No, still broken\nbob: >> Fixed in v2 > No, still broken now?' "comments.txt raw"
expect_err 422 "author required" /p/MD/7/comments 'author=&body=%3E+x'

# Editing the body re-renders.
expect_code 200 /p/MD/1/edit "body=$(enc $'> q\nr')"
expect_eq "$(body_of /p/MD/1)" "$B<p>q</p>$E"$'\n<p>r</p>' "edited body"

bb_restart
expect_eq "$(body_of /p/MD/1)" "$B<p>q</p>$E"$'\n<p>r</p>' "after restart"
expect_eq "$(body_of /p/MD/11)" "$B$B<p>deep</p>$E"$'\n'"<p>back</p>"$'\n'"$B<p>again</p>$E$E" "after restart, nested"
expect_eq "$(get /p/BASE/1.txt)" 'BASE-1 closed P1 alice [bug,ui] Login fails' "baseline after restart"
bb_unit
echo PASS
