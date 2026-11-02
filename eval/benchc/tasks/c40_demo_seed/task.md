# Seed demo data from the command line

For demos and screenshots we want a ready-made tracker. Add a command-line flag `--demo`.

**Arguments.** The flag may appear anywhere on the command line (`tracker --demo demo.db 9000`,
`tracker demo.db 9000 --demo`). Every argument equal to `--demo` is removed and the remaining ones
are read exactly as before (`[db_path] [port]`, same defaults). Without the flag nothing about the app
changes, and nothing is ever seeded.

**When it seeds.** With the flag, after the database is opened and before the server listens: if the
database has no users and no projects at all, insert the demo data below and print the line
`seeded demo data`; otherwise (at least one user or at least one project) change nothing and print
`demo skipped: database not empty`. Either line comes before the usual `listening on ...` line. The
server then runs normally (POST routes work as usual).

**The demo data.** Its effect on the database (row ids, issue numbers, labels, comments and activity
rows, in id order) is exactly that of these requests sent one after the other to the normal app on
an empty database. Values are shown decoded; `\n` is a newline.

1. `POST /users` with `name` = `alice`, then `bob`, then `carol`.
2. `POST /projects` with `key=DEMO`, `name=Demo project`; then `key=OPS`, `name=Operations`.
3. `POST /p/DEMO/labels` with `name` = `bug`, then `docs`, then `ui`; then `POST /p/OPS/labels` with
   `name=infra`.
4. `POST /p/DEMO/issues` with `title=Login button does nothing`,
   `body=Click **Login** on the home page.\n\nNothing happens.`, `priority=1`, `assignee=alice`,
   `labels=bug,ui`.
5. `POST /p/DEMO/issues` with `title=Write the README`, `labels=docs` (priority 3, no body, no
   assignee).
6. `POST /p/DEMO/issues` with `title=Typo in footer`, `body="Copyright" is misspelled.`,
   `priority=4`, `assignee=bob`.
7. `POST /p/DEMO/1/comments` with `author=bob`, ``body=Reproduced in `Firefox`.``.
8. `POST /p/DEMO/3/status` with `status=closed`.
9. `POST /p/DEMO/2/assign` with `assignee=carol`.
10. `POST /p/OPS/issues` with `title=Rotate backups`, `priority=2`, `labels=infra`.

So, for example, `GET /p/DEMO/issues.txt` then answers

```
DEMO-1 open P1 alice [bug,ui] Login button does nothing
DEMO-2 open P3 carol [docs] Write the README
DEMO-3 closed P4 bob [] Typo in footer
```

and `GET /p/DEMO/2/activity.txt` answers `created` then `assigned carol`.
