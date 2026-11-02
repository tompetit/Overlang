# Rename the status "closed" to "done"

The team says "done", not "closed". Rename the status value everywhere, so an issue is `open` or `done`:

- the database: the `issues.status` column accepts exactly `open` and `done` (a row with `closed` is
  rejected by the table itself);
- `POST /p/{KEY}/{N}/status` takes `status=done`; `status=closed` is now `422` `bad status`;
- the filter takes `status=done`; `status=closed` is now `400` `bad status`;
- every output that shows a status: `issue_line` (`WEB-1 done P1 alice [bug] Login fails`), the project page
  rows, the issue page (`<span class="status">done</span>`), the JSON (`"status":"done"`), the CSV, and the
  activity detail (`status done`);
- the toggle form on the issue page of an open issue becomes
  `<form method="post" action="B/status"><input type="hidden" name="status" value="done"><button>done</button></form>`;
  for a done issue it stays `value="open"` with the button `reopen`.

**Existing databases.** Production databases were created by the previous version (the schema in section 1
of the spec, whose check allows `open` and `closed`). On startup the server must upgrade such a database in
place, before it serves anything: every issue whose status is `closed` becomes `done`, every activity row of
kind `status` whose detail is `closed` gets the detail `done`, and from then on the table accepts `done` and
rejects `closed`. Nothing else changes: issue ids, numbers, titles, bodies, priorities, assignees, labels,
comments and the other activity rows are kept as they were (an `assigned` row whose detail is a user named
`closed` stays as it is), and new issues continue the numbering. Starting on a new or already upgraded
database changes nothing.

Update the unit tests.
