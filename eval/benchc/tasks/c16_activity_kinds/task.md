# More activity kinds

The activity log records only creation, status, assignee and comment events, so edits and label
changes are invisible in an issue's history. Add three activity kinds:

| Event | kind | detail |
|---|---|---|
| `POST /p/{KEY}/{N}/edit` that actually changes the issue | `edited` | the names of the changed fields among `title`, `body`, `priority`, in that order, joined by `,` (for example `title,priority`) |
| `POST /p/{KEY}/{N}/labels` that actually attaches the label | `labeled` | the label name |
| `POST /p/{KEY}/{N}/labels/remove` that actually detaches the label | `unlabeled` | the label name |

"Actually changes" compares the value that would be stored (after the usual cleaning of form fields,
and with an empty field keeping its current value) with the current one: an edit that changes nothing
logs nothing, attaching a label the issue already has logs nothing, and removing a label the issue
does not have logs nothing. One edit request logs at most one `edited` row. Labels given when an issue
is created log nothing beyond the usual `created`. A request answered with an error (4xx) logs nothing.

The new rows appear wherever activity appears, in the existing formats: `GET /p/{KEY}/{N}/activity.txt`
(for example `edited body,priority`, `labeled ui`), the activity list of the issue page, and the
`activity` array of `GET /p/{KEY}/{N}.json` (for example `{"kind":"unlabeled","detail":"ui"}`).
Store them in the database like the other kinds. A fresh database is fine: existing database files
need not be migrated. Everything else is unchanged.
