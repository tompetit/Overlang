# Link issue references

People write "duplicate of WEB-12" or "blocked by API-3" in issue descriptions and comments. Make the
markdown-lite renderer (spec section 2.5) turn such references into links to the issue, everywhere
markdown is rendered (issue bodies on the issue page and comments). Only markdown changes: issue titles
(in headings, tables, page titles), the text, JSON and CSV formats and search keep the raw text.

**The rule.** `md_inline`'s scan gets a new rule, tried after rules 1 to 3 (code span, bold, link) and
before "append one code point", which becomes rule 5. Like the others it works on the already-escaped
text `t`, at position `k`. A reference starts at `k` when all of these hold:

- `t[k]` is an ASCII uppercase letter, and `k` is 0 or `t[k-1]` is not an ASCII letter, ASCII digit,
  `_`, `-` or `/`;
- the maximal run of ASCII uppercase letters and ASCII digits starting at `k` (the key) is 2 to 10 code
  points long, and the code point right after it is `-`;
- after the `-` comes a maximal run of 1 to 9 ASCII digits (the number) whose first digit is not `0`, and
  the code point after that run, if there is one, is not an ASCII letter, ASCII digit or `_`;
- an issue with that number exists in the project with that key, right now.

Then append

    <a href="/p/{key}/{number}" class="{class}" title="{title}">{key}-{number}</a>

where `{class}` is `issue-ref` when that issue is open and `issue-ref closed` when it is closed, and
`{title}` is that issue's current title, HTML-escaped; `k` moves past the number. When any condition
fails, rule 4 does not apply (and the scan goes on at the next code point as usual, so a non-reference
is left exactly as it was).

So a reference always shows the referenced issue as it is when the page is rendered: closing, reopening
or renaming it, or creating the issue a reference names, changes how pages that mention it render,
without editing those pages. Since it is a rule of the same scan, a reference inside a code span or a
`[text](url)` link is not linked, while one inside `**bold**` is.

Example: with WEB-2 open and titled `Tom's "cat"`, and API-1 closed and titled `Rate limit`, the body
`dup of WEB-2 (see API-1). WEB-99 web-2` renders as

    <p>dup of <a href="/p/WEB/2" class="issue-ref" title="Tom&#39;s &quot;cat&quot;">WEB-2</a> (see <a href="/p/API/1" class="issue-ref closed" title="Rate limit">API-1</a>). WEB-99 web-2</p>
