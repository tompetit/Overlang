# Read-only mode

Operations wants to run a copy of the tracker that nobody can change. Add a command-line flag
`--read-only`.

**Arguments.** The flag may appear anywhere on the command line (`tracker --read-only data.db 9000`
and `tracker data.db 9000 --read-only` mean the same). Every argument equal to `--read-only` is
removed, and the remaining ones are read exactly as before (`[db_path] [port]`, same defaults, a bad
port still falls back to 8080). Without the flag nothing about the app changes.

**Startup.** In read-only mode the database must already exist: when there is no file at `db_path`,
print `cannot open <db_path>` and exit with status 1, without creating the file. Otherwise print the
line `read-only mode` right before the usual `listening on ...` line. A read-only server never
changes the database file: its bytes are identical before the server starts and after it stops,
whatever requests it answered.

**Requests.**

- Every POST whose method and path match a POST row of the route table (rows 5, 7, 11, 13 and 19
  to 24) answers `403` with the body `read-only mode` (content type and trailing newline like the
  other errors). This check comes before every other one, so a POST to an unknown project or issue,
  or with missing or invalid fields, is `403` as well.
- Requests that match no row are still `404` `not found` (for example `POST /users.txt`,
  `POST /p/WEB/abc/status`, `POST /health`, `DELETE /`).
- GET routes answer exactly as they do without the flag, except for the two changes below.

**What it looks like.**

- `GET /health` answers `ok read-only` (one line) in read-only mode, and `ok` as before otherwise.
- Every HTML page leaves out all its POST forms: each `<form method="post" ...>...</form>` of
  section 2.4 is omitted entirely, and so is the `<h2>New issue</h2>` heading of the project page.
  Everything else stays, including the GET filter form of the project page and the
  `<h2>Comments</h2>` and `<h2>Activity</h2>` sections of the issue page. For example the users page
  body ends `<li>bob</li></ul></body></html>`.
- The navigation bar of every page gets a marker after the Users link:
  `<nav><a href="/">Tracker</a> <a href="/users">Users</a> <span class="mode">read-only</span></nav>`.
  Without the flag it is unchanged.
