# Tracker

An HTTP issue tracker over SQLite: users, projects, issues with labels, assignees, comments,
an activity log, filters, search and pagination, served as HTML, text, JSON and CSV. The contract
is [SPEC.md](SPEC.md); the Python port in `../tracker_py/` gives byte-identical responses.

| File | Lines | Tests |
|---|---|---|
| `src/01_db.ov` | 139 | 4 |
| `src/02_models.ov` | 441 | 5 |
| `src/03_render.ov` | 380 | 5 |
| `src/04_routes.ov` | 504 | 5 |
| total | 1,464 | 19 |

The four files are one flat namespace (v0.1 has no modules) and depend on each other in file order.
The total is below the spec's 1,800 to 2,200 line budget; nothing in the spec was left out.

## Build

```
cargo build --release                  # once; the shims in runtime/ext/ are embedded in ov
examples/tracker/build.sh .ovbuild/tracker
```

`build.sh DIR` makes a fresh store in `DIR` (it deletes `DIR/.ovstore` and `DIR/.ovbuild`), runs
`ov add` on the four files in order, then `ov check`, `ov test` (19 test blocks, then fuzzing of
`page_count`, `page_offset`, `parse_nat`, `issue_ref`) and `ov build main`, and leaves the binary at
`DIR/tracker`. `.ovbuild/` in the repo root is gitignored, so it is a good `DIR`. Without a store,
`ov check|test|build src/01_db.ov src/02_models.ov src/03_render.ov src/04_routes.ov` also works.

## Run

```
.ovbuild/tracker/tracker [db_path] [port]   # defaults tracker.db and 8080; a bad port means 8080
```

It prints `listening on http://127.0.0.1:<port>` when ready, or `cannot open <db_path>` /
`cannot listen on port <port>` and exits 1. Routes are in SPEC.md section 2.2, for example:

```
curl -d name=alice localhost:8080/users
curl -d 'key=WEB&name=Web+site' localhost:8080/projects
curl -d 'title=Login+fails&priority=1&assignee=alice' localhost:8080/p/WEB/issues
curl 'localhost:8080/p/WEB/issues.txt?status=open'
curl localhost:8080/p/WEB/1.json
```

## Choices where the spec leaves room

Both ports do the same in each case (checked by running one request sequence against both and
diffing every status, content type and body).

- `{N}` with leading zeros is accepted: `/p/WEB/01` is issue 1 (spec: ASCII digits, value >= 1).
  `page=01` and `priority=02` likewise.
- `+` in a form or query value decodes to a space before trimming, so `priority=+3` is priority 3.
- A `page` past the last page (but <= 1,000,000) is a 200 with no issues, not an error.
- Line fields only clean `\r`, `\n`, `\x1e`, `\x1f`; a tab in a title is kept.
- Repeated names in `labels=` on issue creation (`bug,bug`) attach the label once.
- `ov` links SQLite 3.40, which lacks `group_concat(... order by ...)`, so `issue_select_sql`
  orders labels in a subquery; the columns and results match section 1.1 (see LIMITS.md).
- Long `++` chains break `ov check` from a store, so render builds pages with `join` (LIMITS.md).
