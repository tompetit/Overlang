# Due dates on issues

Issues get an optional due date. Store it in a new `due` column of the `issues` table (a fresh
database is fine; no migration of existing files is needed). A due date is written `YYYY-MM-DD`.
Nothing else about issues changes, and setting or changing a due date logs no activity.

**Validating a date.** A value that is not exactly four ASCII digits, `-`, two ASCII digits, `-`,
two ASCII digits is malformed: 400 `bad due date`. A value of that shape that is not a real
calendar date is 422 `bad due date`: the year must be 0001 to 9999, the month 01 to 12, the day
within the month, and February 29 exists only in leap years (divisible by 4, except years
divisible by 100 that are not divisible by 400). On any error nothing is stored.

**Writing it.**

- `POST /p/{KEY}/issues` takes an optional form field `due` (a line field, so it is trimmed).
  Empty, absent, or the single character `-` means no due date. It is checked after all the
  route's existing checks, so for example a blank title is still 422 `title required` and an
  unknown label still wins over a bad date.
- `POST /p/{KEY}/{N}/edit` takes an optional `due` too, checked after the existing priority
  checks. Empty keeps the current due date (like the other fields of this route), `-` removes it,
  a valid date sets it.

**Showing it.** Below, `{due}` is the date, or `-` when the issue has none.

- `issue_line` (so `/p/{KEY}/issues.txt` and `/p/{KEY}/{N}.txt`): the due date goes between the
  labels and the title: `WEB-7 open P2 alice [bug,ui] 2025-03-01 Fix "login"`, and
  `WEB-8 closed P3 - [] - Typo` when there is none.
- Issue JSON: a key `"due"` right after `"labels"`, a string or `null`:
  `..."labels":["bug","ui"],"due":"2025-03-01","comments":...`.
- CSV: the header becomes `key,title,status,priority,assignee,labels,due,body`; the `due` field is
  the date, or empty when there is none.
- Project page: the issues table header gets `<th>Due</th>` after `<th>Labels</th>`, and each row
  a `<td>{due}</td>` after its labels cell. The new-issue form gets
  ` <input name="due" placeholder="YYYY-MM-DD">` right after the `labels` input.
- Issue page: the meta line ends `| Labels: {labels} | Due: {due}</p>`, and the edit form gets
  ` <input name="due" value="{the date, or empty}">` right after its `priority` input.

**Filtering by it.** The project page and `issues.txt` take a new query value `due_before`.
`parse_filter` reads it after `page`; empty means no filter; otherwise it must be a valid date
(shape and calendar), else 400 `bad due date`. When set, only issues with a due date strictly
before it match (an issue without a due date never matches); it combines with the other filters
and with pagination like they do. In `filter_query` (the pager links) it goes after `q` and before
`page`: `status=open&due_before=2025-02-01&page=2`. The filter form gets
` <input name="due_before" value="{f.due_before}" placeholder="due before">` right after the
search (`q`) input.
