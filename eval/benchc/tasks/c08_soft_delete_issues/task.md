# Delete and restore issues

Issues can now be deleted, and a deletion can be undone. Deleting keeps the row: add an integer column
`deleted` to `issues` (0 by default, 1 for a deleted issue); the issue's labels, comments and activity rows
stay where they are. A fresh database is fine, no migration needed.

New routes (unknown project and unknown issue are checked first as for every route under `/p/{KEY}/{N}`):

- `POST /p/{KEY}/{N}/delete` (no form fields): marks the issue deleted, logs an activity row of kind
  `deleted` with an empty detail, and answers the HTML project page with no filter.
- `POST /p/{KEY}/{N}/restore` (no form fields): if the issue is deleted, marks it not deleted and logs an
  activity row of kind `restored` with an empty detail; if it is not deleted, nothing changes and nothing is
  logged. Either way it answers the HTML issue page. A number that no issue of the project has (deleted or
  not) is `404` `no such issue: {KEY}-{N}`.
- `GET /p/{KEY}/trash.txt` (text): the `issue_line` of every deleted issue of the project, by number. No
  deleted issues is the empty body.

Apart from `restore`, a deleted issue no longer exists for any route: the project page (its rows and the
`Page x of y (n issues)` count), `issues.txt` with or without filters, and `issues.csv` leave it out, and every
other route under `/p/{KEY}/{N}` (the page, `.json`, `.txt`, `comments.txt`, `activity.txt`, `edit`, `status`,
`assign`, `comments`, `labels`, `labels/remove` and `delete` itself) answers `404` `no such issue: {KEY}-{N}`.

Numbers are never reused: a new issue gets the highest number of any issue of the project, deleted ones
included, plus one. A restored issue comes back exactly as it was (same number, title, status, labels,
comments), and its activity then ends with `deleted` and `restored` (`activity.txt` shows them as the lines
`deleted` and `restored`).
