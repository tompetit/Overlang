# A limit on open issues per project

Some teams want to cap their backlog. Each project gets an optional limit on how many of its issues
may be open at once.

**Storage.** Store the limit in a new column `open_limit` of the `projects` table: an integer, not
null, default `0`, where `0` means no limit. Production databases were created by the current version
(section 1 of the spec) and have no such column: on startup, before serving, add the column when it is
missing (every existing project then has no limit) and keep all existing data as it is. A database
that already has the column is left alone, so restarting is harmless.

**Setting it.** `POST /p/{KEY}/limit` with the form field `limit` (a line field):

- unknown project: `404` `no such project: {KEY}` as for every route under `/p/{KEY}`;
- `limit` not `parse_nat`-able (this includes empty): `400` `bad limit`;
- a value above `1000`: `422` `bad limit`;
- `0` removes the limit; `1` to `1000` sets it. A limit below the current number of open issues is
  allowed: nothing is closed, it only blocks new open issues.

On success it answers the HTML project page (`no_filter()`), like creating a label does. Any other
method on that path is `404` `not found`.

**Reading it.** `GET /p/{KEY}/limit.txt` (text) answers one line: `open {n} of {limit}` when the
project has a limit, `open {n}, no limit` when it has none, where `{n}` is the project's number of open
issues. Unknown project: `404` as above.

**Enforcing it.** While a project has a limit and its open issues number at least the limit:

- `POST /p/{KEY}/issues` answers `422` `open issue limit reached: {limit}` and creates nothing. This
  check comes after all of the route's existing checks, so a blank title, a bad priority, an unknown
  user or label are still reported as before.
- `POST /p/{KEY}/{N}/status` with `status=open` on a closed issue answers the same `422` and changes
  nothing (no activity). This check comes after the existing `bad status` check. Setting `open` on an
  issue that is already open is still a successful no-op, and closing is always allowed.

Nothing else is limited (edits, assignment, comments, labels), and other projects are unaffected.

**Showing it.** On the project page, right after the pager paragraph and before `<h2>Labels</h2>`,
add, when the project has a limit, `<p class="limit">Open issues: {n} of {limit}</p>`, then, always,
`<form method="post" action="/p/{key}/limit"><input name="limit" value="{limit}"> <button>set limit</button></form>`
where `{limit}` is `0` when there is none.
