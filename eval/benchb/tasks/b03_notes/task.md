# Notes on todos

Todos can carry a free-text note. `POST /add` takes an optional form field `notes`. The HTML page shows a
non-empty note right after the title as `<small>NOTE</small>` (escaped like titles); todos without a note get
no `<small>`. Add `POST /notes` with fields `id` and `notes` that replaces a todo's note (blank clears it) and
answers with the page; a bad id (missing, not a number, zero or negative) is `400` with `bad id`. `/todos.txt`
does not change. Notes live in the database (fresh database is fine).
