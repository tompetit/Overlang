# Autolink bare URLs

People paste links into issues and comments without the `[text](url)` syntax, and they render as dead
text. Make the markdown-lite renderer (spec section 2.5) turn bare `http://` and `https://` URLs into
links, everywhere markdown is rendered (issue bodies on the issue page and comments). Everything else it
does stays as it is, and the text, JSON and CSV formats keep showing the raw body.

**The rule.** `md_inline` gets a new rule, tried after rules 1 to 3 (code span, bold, link) and before
"append one code point", which becomes rule 5. Like the others it works on the already-escaped text `t`,
at position `k`:

4. `t` continues at `k` with `http://` or `https://` (lowercase, exactly), and `k` is 0 or the code point
   before it, `t[k-1]`, is not an ASCII letter or ASCII digit. The *candidate* runs from `k` up to, not
   including, the first space, tab or `\n`, or the first `&` that starts one of `&lt;` `&gt;` `&quot;`
   `&#39;` (so a raw `<` `>` `"` `'` ends a URL, while a raw `&` is kept as `&amp;`); or to the end of
   `t`. Then, as long as the candidate ends with one of `.` `,` `:` `!` `?` `)`, drop that last code point.
   If at least one code point is left after the `://`, the rest is the URL `u`: append
   `<a href="` + u + `">` + u + `</a>`, and `k` moves just past `u` (dropped punctuation is then
   processed normally). Otherwise rule 4 does not apply.

Since it is a rule of the same scan, a URL inside a code span or inside a `[text](url)` link (its text or
its URL) is not autolinked, while one inside `**bold**` is, and a URL never extends past the end of the
bold text it is in.

Examples (`md_inline` of the raw input, then output):

    see https://e.com/a?x=1&y=2.        see <a href="https://e.com/a?x=1&amp;y=2">https://e.com/a?x=1&amp;y=2</a>.
    (docs: http://e.com/x)!             (docs: <a href="http://e.com/x">http://e.com/x</a>)!
    http://e.com/"onclick="x            <a href="http://e.com/">http://e.com/</a>&quot;onclick=&quot;x
    xhttp://e.com HTTP://E.COM http://  xhttp://e.com HTTP://E.COM http://
