# Case-insensitive project keys

People keep typing `web` for the project `WEB` and get `bad key` or a 404. Make project keys case
insensitive by normalising them to upper case. "Upper-casing" here means ASCII letters only: `a` to
`z` become `A` to `Z` and every other character is left as it is.

**Creating a project** (`POST /projects`). The `key` is cleaned as today, then upper-cased, then
validated with the existing rules, so `key=web` creates `WEB`, `key=Web` when `WEB` exists is 422
`project exists`, and `key=w` or `key=3web` is still 422 `bad key`. Project names also get a limit:
a `name` longer than 60 code points (after cleaning) is 422 `name too long`. The checks, in order,
the first failure being the response: `key required`, `bad key`, `name required`, `name too long`,
`project exists`.

**Every route under `/p/{KEY}`** upper-cases the `{KEY}` path segment before looking the project
up, so `/p/web`, `/p/Web/issues.txt`, `POST /p/web/3/comments` and so on all act on `WEB`. The 404
messages use the upper-cased key: `GET /p/nope` is 404 `no such project: NOPE`, and `GET /p/web/9`
(when WEB has no issue 9) is 404 `no such issue: WEB-9`.

Everything the app shows (issue keys, links, form actions, page titles, text, JSON and CSV) keeps
using the key as stored, which is always upper case. Existing rows are not migrated (a fresh
database is fine).
