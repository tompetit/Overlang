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
expect_md $'line1\nline2' $'<p>line1\nline2</p>'
expect_md '' ''
expect_md '**a [l](/x) b**' '<p><strong>a <a href="/x">l</a> b</strong></p>'

# Headings.
expect_md '# Fix **login** & logout!' '<h3 id="h-fix-login-logout">Fix <strong>login</strong> &amp; logout!</h3>'
expect_md $'Intro\n## Steps\n1. open\n#tag\n###   Steps' $'<p>Intro</p>\n<h4 id="h-steps">Steps</h4>\n<p>1. open\n#tag</p>\n<h5 id="h-steps-2">Steps</h5>'
expect_md $'# One\n## Two\n### Three\n#### Four' $'<h3 id="h-one">One</h3>\n<h4 id="h-two">Two</h4>\n<h5 id="h-three">Three</h5>\n<p>#### Four</p>'
expect_md $'# A\n\n# a\nx\n\n## A!' $'<h3 id="h-a">A</h3>\n<h3 id="h-a-2">a</h3>\n<p>x</p>\n<h4 id="h-a-3">A!</h4>'
expect_md $'# ***\ntext\n# ?!\n### \xc3\xa9\xc3\xa9' $'<h3 id="h">***</h3>\n<p>text</p>\n<h3 id="h-2">?!</h3>\n<h5 id="h-3">\xc3\xa9\xc3\xa9</h5>'
expect_md $'# Caf\xc3\xa9 cr\xc3\xa8me 2024' $'<h3 id="h-caf-cr-me-2024">Caf\xc3\xa9 cr\xc3\xa8me 2024</h3>'
expect_md $'# x\n#\tnot\n #indented\n # indented too\n#12 issue\n#    \nend' $'<h3 id="h-x">x</h3>\n<p>#\tnot\n #indented\n # indented too\n#12 issue\n#    \nend</p>'
expect_md $'##no space\n#  Two  spaces  \n\n\n  \nafter' $'<p>##no space</p>\n<h3 id="h-two-spaces">Two  spaces</h3>\n<p>after</p>'
expect_md $'# `<code>` and [l](/p/MD/1)\n## [x](javascript:alert(1))' $'<h3 id="h-code-and-l-p-md-1"><code>&lt;code&gt;</code> and <a href="/p/MD/1">l</a></h3>\n<h4 id="h-x-javascript-alert-1">[x](javascript:alert(1))</h4>'

# Hostile headings: the text is escaped and the id is only [a-z0-9-].
expect_md '# <script>alert(1)</script>' '<h3 id="h-script-alert-1-script">&lt;script&gt;alert(1)&lt;/script&gt;</h3>'
expect_md $'## "><img src=x onerror=alert(1)>\n### \' onmouseover=\'alert(1)' $'<h4 id="h-img-src-x-onerror-alert-1">&quot;&gt;&lt;img src=x onerror=alert(1)&gt;</h4>\n<h5 id="h-onmouseover-alert-1">&#39; onmouseover=&#39;alert(1)</h5>'
page="$(get "/p/MD/$n")"
expect_lacks "$page" '<img' "raw img tag"
expect_lacks "$page" "id=\"h-'" "quote in id"
expect_has "$page" '<textarea name="body">## &quot;&gt;&lt;img src=x onerror=alert(1)&gt;' "edit form shows the raw body escaped"

# Raw formats keep the source.
expect_has "$(get /p/MD/7.json)" '"body":"# Fix **login** & logout!"' "json body raw"
expect_has "$(get /p/MD/issues.csv)" 'MD-7,md,open,3,,,# Fix **login** & logout!' "csv body raw"

# Comments render headings too, with ids counted per comment.
expect_code 200 /p/MD/7/comments "author=alice&body=$(enc $'## Repro\nclick **here**\n# Fix **login** & logout!')"
expect_code 200 /p/MD/7/comments "author=bob&body=$(enc $'## Repro\n## Repro')"
expect_eq "$(comments_of /p/MD/7)" $'<li><b>alice</b> <h4 id="h-repro">Repro</h4>\n<p>click <strong>here</strong></p>\n<h3 id="h-fix-login-logout">Fix <strong>login</strong> &amp; logout!</h3></li><li><b>bob</b> <h4 id="h-repro">Repro</h4>\n<h4 id="h-repro-2">Repro</h4></li>' "comments"
expect_eq "$(get /p/MD/7/comments.txt)" $'alice: ## Repro click **here** # Fix **login** & logout!\nbob: ## Repro ## Repro' "comments.txt raw"
expect_err 422 "body required" /p/MD/7/comments 'author=bob&body=+'

# Editing the body re-renders.
expect_code 200 /p/MD/1/edit "body=$(enc $'para\n# New')"
expect_eq "$(body_of /p/MD/1)" $'<p>para</p>\n<h3 id="h-new">New</h3>' "edited body"

bb_restart
expect_eq "$(body_of /p/MD/1)" $'<p>para</p>\n<h3 id="h-new">New</h3>' "after restart"
expect_eq "$(body_of /p/MD/10)" $'<h3 id="h-a">A</h3>\n<h3 id="h-a-2">a</h3>\n<p>x</p>\n<h4 id="h-a-3">A!</h4>' "after restart, ids"
expect_eq "$(get /p/BASE/1.txt)" 'BASE-1 closed P1 alice [bug,ui] Login fails' "baseline after restart"
bb_unit
echo PASS
