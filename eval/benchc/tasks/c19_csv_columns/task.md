# Richer, spreadsheet-safe CSV export

`GET /p/{KEY}/issues.csv` is opened in spreadsheets, which has two problems: it lacks the issue's comment
count and latest activity, and a title such as `=HYPERLINK(...)` is run as a formula. Change the export:

- Two new columns at the end. The header line becomes
  `key,title,status,priority,assignee,labels,body,comments,last_activity`. `comments` is the number of
  comments on the issue (`0` when none). `last_activity` is the issue's most recent activity row (the
  one with the highest id), written as one line of the issue's `activity.txt` (for example `created`,
  `status closed`, `assigned -`, `commented bob`).
- Formula guard: a field whose first character is `=`, `+`, `-` or `@` gets a `'` put in front of it.
  This applies to every column; the existing quoting rule is then applied to the guarded value (a field
  containing `,` `"` a newline or a carriage return is wrapped in `"` with each `"` doubled). For example
  the title `=SUM(A1,B1)` is written `"'=SUM(A1,B1)"` and the labels `-wip` and `bug` are written
  `'-wip;bug`.
- Every line, the header included, ends with `\r\n` instead of `\n`. A newline inside a quoted field is
  written as it is stored (a lone `\n`), not converted.

Everything else in the export stays as it is (one row per issue of the project by number, all issues,
no query parameters, content type `text/csv; charset=utf-8`; a project without issues gives the header
line alone). Only the CSV changes: the JSON, text and HTML outputs show stored values without the guard.
