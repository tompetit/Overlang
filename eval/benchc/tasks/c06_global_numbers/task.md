# Global issue numbers

Issue numbers are counted per project today (`WEB-1`, `API-1`, ...). We want one sequence for the whole
tracker, so that a number alone names an issue: a new issue in any project gets the highest number used by
any issue in any project, plus one (the very first issue is 1). Keys keep their project prefix, so creating
issues in WEB, API, WEB gives `WEB-1`, `API-2`, `WEB-3`. Numbers are never shared: the `issues` table itself
must reject a row whose number is already used by another issue, in any project. Assigning the number stays
atomic with the insert.

Every route under `/p/{KEY}/{N}` still requires issue `N` to belong to project `KEY`: when issue 1 is in WEB,
`GET /p/API/1` (and every other route under `/p/API/1`) is `404` with the body `no such issue: API-1`.

Add two routes:

- `GET /i/{N}`: the HTML issue page of issue `N`, the same page as `GET /p/{KEY}/{N}`.
- `GET /i/{N}.txt`: its text line, the same as `GET /p/{KEY}/{N}.txt`.

`{N}` is read as everywhere else (ASCII digits, value >= 1; a segment that is not, such as `/i/0` or
`/i/abc`, falls through to `404` `not found`, and so does any other suffix or method on `/i/...`). A number
that names no issue is `404` with the body `no such issue: {N}` (for example `no such issue: 99`).

Nothing else changes: the JSON `number` field, the CSV, ordering by number within a project, filters and
pagination behave as before. A fresh database is fine, no migration needed. Update the unit tests that
expect per-project numbering.
