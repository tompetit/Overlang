# Export a whole project as JSON

Teams moving a project elsewhere need all of it in one file. Add `GET /p/{KEY}/export.json`.

It answers `200` with content type `application/json` and one line of compact JSON followed by `\n`,
with these keys in this order:

```
{"key":"WEB","name":"Web site","labels":["bug","ui"],"users":["alice","bob"],"issues":[...]}
```

- `key` and `name`: the project's, as strings written like every string of the issue JSON (section
  2.3 `json_str`).
- `labels`: the names of all the project's labels, ascending, whether or not any issue uses them.
- `users`: the names of the users the project's issues refer to right now: every current assignee of
  one of its issues and every author of a comment on one of its issues. Each name once, in ascending
  code-point order. Users who only appear in activity details (for example someone who was assigned
  and later unassigned) or only in other projects are not listed.
- `issues`: every issue of the project, ordered by number, all of them (no pagination, no filters).
  Each element is exactly the object that `GET /p/{KEY}/{N}.json` answers for that issue, without
  that route's trailing newline; elements are separated by `,` with no spaces.

A project without labels, issues or users gives `"labels":[]`, `"users":[]`, `"issues":[]`. An
unknown project is `404` `no such project: {KEY}` like every route under `/p/{KEY}`. The query
string is ignored. Any other method, and paths such as `/p/{KEY}/export` or `/p/{KEY}/export.csv`,
are `404` `not found` as before. Nothing else changes.
