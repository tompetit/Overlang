# Milestones

Projects get milestones, and each issue can belong to at most one milestone of its own project. Store them
in a new table `milestones` (id, project id, name; a name is unique within its project, and two projects may
use the same name) and reference the milestone from the issue. A fresh database is fine, no migration needed.

A valid milestone name is 1 to 20 code points, each an ASCII letter, digit, `.`, `-` or `_` (so `v1.0`,
`Beta` and `release_2026-11.rc_1` are valid).

New routes. As for every route under `/p/{KEY}`, an unknown project is checked first (`404`
`no such project: {KEY}`), and under `/p/{KEY}/{N}` an unknown issue next (`404` `no such issue: {KEY}-{N}`).

- `POST /p/{KEY}/milestones`, form field `name` (a line field): empty: `422` `name required`; not a valid
  milestone name: `422` `bad milestone`; already a milestone of this project: `422` `milestone exists`.
  On success, the HTML project page with no filter (as after creating a label).
- `GET /p/{KEY}/milestones.txt` (text): one line per milestone of the project, ordered by name ascending in
  byte order (`Z` before `a`): `{name} {open}/{total}\n`, where `total` is the number of issues in the
  milestone and `open` how many of them have status `open`. For example `v1.0 1/2` or `Beta 0/0`. No
  milestones is the empty body.
- `POST /p/{KEY}/{N}/milestone`, form field `milestone` (a line field): empty takes the issue out of its
  milestone; otherwise a name that is not a milestone of this project is `422` `no such milestone: {milestone}`.
  On success, the HTML issue page.

When an issue's milestone actually changes, log an activity row of kind `milestone` whose detail is the new
milestone's name, or `-` when the issue left its milestone, so `activity.txt` shows `milestone v1.0` or
`milestone -` and the JSON `{"kind":"milestone","detail":"v1.0"}`. Setting the milestone it already has (or
clearing an issue that has none) changes nothing and logs nothing.

Display:

- Issue page: the meta paragraph ends with ` | Milestone: {name}`, or ` | Milestone: -` when none, after the
  labels: `<p class="meta">Status: <span class="status">open</span> | Priority: P3 | Assignee: - | Labels: bug | Milestone: v1.0</p>`.
  Right after the remove-label form comes
  `<form method="post" action="B/milestone"><input name="milestone" value="{milestone name, or empty}"> <button>set milestone</button></form>`
  (`B` as in the other forms, the name HTML-escaped like every value).
- Issue JSON: a new key `"milestone"` right after `"labels"`: `null` when none, else the name as a JSON string.

Everything else is unchanged: issue lines, `issues.txt`, the CSV, the project page and its filters. Update
the unit tests.
