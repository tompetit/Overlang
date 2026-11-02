# Close with a resolution

"closed" lumps together fixed bugs, rejected requests and duplicates. Split it: `status` stays `open` or
`closed`, and a new text column `resolution` on `issues` says why a closed issue is closed. It is one of
`fixed`, `wontfix` or `duplicate` when the issue is closed, and the empty string exactly when it is open.
A fresh database is fine, no migration needed.

**Status token.** Wherever the status is shown as text, show the token `open` for an open issue and
`closed:{resolution}` for a closed one (`closed:wontfix`): in `issue_line` (so `issues.txt` and `{N}.txt`
read `WEB-1 closed:fixed P2 alice [bug] Fix it`), in the status cell of the project page rows
(`<td>closed:fixed</td>`), on the issue page (`<span class="status">closed:fixed</span>`), and in the
detail of `status` activity rows (`status closed:duplicate`; reopening logs `status open` as before). The
close/reopen form on the issue page is unchanged (it still depends only on open/closed).

**Setting it.** `POST /p/{KEY}/{N}/status` takes `status` and an optional `resolution` (a line field). After
the project and issue lookups, checked in this order:

1. status not `open` or `closed`: `422` `bad status`;
2. status `open` and resolution non-empty: `422` `bad resolution`;
3. status `closed` and resolution neither empty nor one of the three values: `422` `bad resolution`.

Closing with an empty resolution means `fixed`. Opening clears the resolution. Closing a closed issue with a
different resolution is a change. Every change of status or resolution logs one `status` activity row
whose detail is the new token; a request that changes nothing logs nothing.

**JSON and CSV.** The JSON keeps `"status":"open"` / `"status":"closed"` and gets a new key `"resolution"`
right after `"status"`: `null` when open, else the resolution as a JSON string. The CSV gets a column
`resolution` right after `status`: header `key,title,status,resolution,priority,assignee,labels,body`, the
field empty when open.

**Filter.** The project page and `issues.txt` accept a new query key `resolution`: empty (no filter) or one
of the three values, else `400` `bad resolution`. It is read right after `status` and before `priority`, so
`?status=x&resolution=y` is `bad status` and `?resolution=y&priority=9` is `bad resolution`. A resolution
filter matches only issues with that resolution (so `status=open&resolution=fixed` matches nothing);
`status=closed` alone still matches every closed issue. In the pager links the `resolution` pair comes right
after `status`: `/p/WEB?status=closed&resolution=wontfix&page=2`. The filter form on the project page is
unchanged.

Update the unit tests.
