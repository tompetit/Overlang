# Project activity feed

Each issue has its own `activity.txt`, but there is no way to see what happened in a whole project.
Add `GET /p/{KEY}/activity.txt`: the activity rows of every issue of the project, one line per row,
`{issue key} {activity line}` followed by `\n`, where the activity line is exactly what the issue's own
`activity.txt` shows (for example `WEB-3 status closed`, `WEB-1 created`, `WEB-2 assigned -`). Lines are
in the order the events happened, that is by activity id ascending, across all issues of the project;
activity of other projects never appears. No matching rows is the empty body (still 200). The response
is text (`text/plain; charset=utf-8`).

Query parameters (read like the other query values):

- `kind`: empty or absent means all kinds; otherwise it must be one of `created`, `status`, `assigned`,
  `commented` and only rows of that kind are listed; anything else is 400 `bad kind`.
- `limit`: empty or absent means 50; otherwise 1 to 9 ASCII digits with a value from 1 to 1000, else
  400 `bad limit`. Only the last `limit` matching rows (the most recent ones) are listed, still oldest
  first.

An unknown project is 404 `no such project: {KEY}` before the query is looked at; then `kind` is
checked, then `limit`. Any other method on this path is 404 `not found`. The per-issue routes, including
`GET /p/{KEY}/{N}/activity.txt`, are unchanged.
