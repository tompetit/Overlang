# Issue versions and conflict detection

Two people editing the same issue silently overwrite each other. Give every issue a version number and
let clients say which version they are changing.

- **Version.** A new issue has version `1`. Every request that actually changes an issue adds 1: an edit
  that changes at least one of title, body or priority, a status change, an assignee change, a label
  actually attached or detached, and a comment added. A request that changes nothing (same status, same
  assignee, a label already attached or not attached, an edit to the same values) and a request answered
  with an error leave the version alone. Store it in the database; a fresh database is fine, existing
  database files need not be migrated.
- **JSON.** `GET /p/{KEY}/{N}.json` gets a `version` key right after `number`:
  `{"key":"WEB-1","project":"WEB","number":1,"version":3,"title":...`. Nothing else in the JSON changes.
- **Check on write.** The six POST routes under `/p/{KEY}/{N}` (`edit`, `status`, `assign`, `comments`,
  `labels`, `labels/remove`) accept an optional form field `version` (a line field). Absent or empty: no
  check, the route behaves as before. Otherwise, right after the unknown-project and unknown-issue
  checks and before any of the route's own checks: a value that is not 1 to 9 ASCII digits is 400
  `bad version`; a value different from the issue's current version is 409 with the body
  `version conflict: current is {current version}` (followed by `\n`, content type
  `text/plain; charset=utf-8` like the other errors), and nothing is changed. A matching version lets
  the request proceed as usual.
- **HTML.** On the issue page, each of the six POST forms starts with
  `<input type="hidden" name="version" value="{version}">` right after its opening `<form ...>` tag, so
  the page's forms carry the version that was shown.

The text, CSV and list outputs and the activity log are unchanged.
