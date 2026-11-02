# Archiving projects

Finished projects should be archived: still readable, no longer writable. Store the state in a new
`archived` column of the `projects` table (an integer, 0 or 1, default 0; a fresh database is fine,
no migration of existing files is needed).

**Routes.**

- New `POST /p/{KEY}/archive` and `POST /p/{KEY}/unarchive` (no form fields). An unknown project is
  the usual 404 `no such project: {KEY}`. They set the project's state (doing nothing when it is
  already in that state, which is not an error) and answer with the HTML project page with
  `no_filter()`. Any other method on these paths is 404 `not found`.
- An archived project is read-only. These routes answer 422 `project archived` and change nothing:
  `POST /p/{KEY}/issues`, `POST /p/{KEY}/labels`, and every POST under `/p/{KEY}/{N}/` (`edit`,
  `status`, `assign`, `comments`, `labels`, `labels/remove`). This check comes right after the
  unknown-project check and, for routes under `/p/{KEY}/{N}`, the unknown-issue check, and before
  every other check of the route: on an archived project `POST /p/{KEY}/issues` with an empty title
  is 422 `project archived`, and `POST /p/{KEY}/9/status` for a missing issue 9 is still 404
  `no such issue: {KEY}-9`.
- Every GET route works on an archived project exactly as before (apart from the display changes
  below). `POST /projects` with the key of an archived project is still 422 `project exists`.

**Showing it.**

- `projects_text`: an archived project's line ends with ` (archived)`: `OLD Old site (archived)`.
  Projects stay ordered by key.
- Home page: `<ul class="projects">` lists only the projects that are not archived. When at least
  one project is archived, right after that list's `</ul>` (before the create form) comes
  `<h2>Archived</h2><ul class="archived">`, then the same `<li>` per archived project, then
  `</ul>`; with no archived project nothing is added.
- Project page and issue page: when the project is archived,
  `<p class="archived">This project is archived.</p>` right after the `</h1>`.
- Project page: at the very end of the body (after the new-issue form),
  `<form method="post" action="/p/{key}/archive"><button>archive</button></form>` when the project
  is active, `<form method="post" action="/p/{key}/unarchive"><button>unarchive</button></form>`
  when it is archived.
- Issue JSON: a key `"archived"` right after `"project"`, `true` or `false`:
  `{"key":"WEB-1","project":"WEB","archived":false,"number":1,...`.

`issues.txt`, `{N}.txt`, the CSV and the other text formats do not change.
