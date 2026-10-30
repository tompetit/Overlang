# Soft delete

Deleting a todo should be undoable. `POST /delete` must no longer remove the row: keep it in table `todos` and
set a new integer column `deleted` to 1 (default 0). Deleted todos disappear from `GET /` and `GET /todos.txt`.
Add `POST /undelete` with a form field `id` that brings one back (same id, same place in the order) and answers
with the page; a bad id is `400` with `bad id`, an id that matches nothing is a harmless `200`. Everything else
behaves as before. A fresh database is fine, no migration needed.
