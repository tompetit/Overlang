#!/usr/bin/env bash
. "$(dirname "$0")/../../lib.sh"; bb_init "$@"
bb_baseline
add '<script>alert(1)</script>'
add '"q" & '"'"'z'"'"' <i>'
page="$(get /)"
expect_lacks "$page" "<script>" "raw script tag in page"
expect_lacks "$page" "<i>" "raw tag in page"
expect_has "$page" "&lt;script&gt;alert(1)&lt;/script&gt;" "escaped script"
expect_has "$page" "&quot;q&quot; &amp; &#39;z&#39; &lt;i&gt;" "escaped quotes and ampersand"
expect_lacks "$page" "&amp;amp;" "double escaping"
expect_lacks "$page" "&amp;lt;" "double escaping"
expect_has "$(list)" "3 [ ] <script>alert(1)</script>" "plain text keeps raw title"
bb_unit
echo PASS
