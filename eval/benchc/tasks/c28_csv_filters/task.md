# One issue query, and a filterable CSV export

The models spell the issue query out several times: the page of issues, the count, the CSV export and
the single-issue lookup each glue the issue select, a `where` and their own tail together, and the
count repeats the joins and the filter. Factor it: one function builds the `from`/`where` part of the
query (with its parameters) for a project and a filter, and the page, the count and the export are
all made from it, so each filter condition is written in exactly one place. Every value stays a bound
`?` parameter.

Then use it to make the export filterable:

- `GET /p/{KEY}/issues.csv` now reads the same query values as `GET /p/{KEY}/issues.txt`
  (`status`, `assignee`, `label`, `priority`, `q`, `page`) with the same meanings, the same validation
  and the same errors in the same order (`400` with `bad status`, `bad priority` or `bad page`; an
  unknown project is still `404` `no such project: {KEY}` before any of them).
- The export is not paginated: it lists every matching issue, by number, however many there are.
  `page` is validated like on the other routes but otherwise ignored.
- The format is unchanged (content type `text/csv; charset=utf-8`, the header line first, always
  present, also when nothing matches), and with no query the export is exactly what it was.

`/p/{KEY}` and `/p/{KEY}/issues.txt` behave as before, 20 issues per page.
