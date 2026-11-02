# Health check with counts

The monitoring system wants more from `GET /health` than `ok`, and a dashboard wants the same numbers as
JSON.

**`GET /health`** (text, content type `text/plain; charset=utf-8`) answers these lines, each ending `\n`:

```
ok
users 2
projects 2
issues 3
open 2
closed 1
labels 2
comments 1
project API 0 0
project WEB 2 1
```

The first eight lines are always there, in this order: the literal `ok`, then the number of users,
projects, issues (all projects), open issues, closed issues, labels (all projects) and comments. Then
one line `project {key} {open} {closed}` per project, ordered by key, including projects that have
no issues (`0 0`). With no projects there are no `project` lines.

**`GET /health.json`** (content type `application/json`) answers the same numbers as compact JSON on
one line followed by `\n`, keys in this order:

```
{"status":"ok","users":2,"projects":2,"issues":3,"open":2,"closed":1,"labels":2,"comments":1,"by_project":[{"key":"API","open":0,"closed":0},{"key":"WEB","open":2,"closed":1}]}
```

`by_project` is `[]` when there are no projects. Keys are written like the issue JSON writes strings.

**When the database fails.** If any of these numbers cannot be read because the database answers with
an error (for example a table is missing), both routes answer `503` with the text body `db error`
(content type `text/plain; charset=utf-8`, trailing newline like the other errors), never a partial
or zero count. Other routes are not affected by this change.

The query string is ignored on both routes, and any other method on them is `404` `not found` as
before. Update the unit tests that expect the old `/health` body.
