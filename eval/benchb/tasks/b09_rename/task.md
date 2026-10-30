# Rename a todo

Add `POST /rename` with form fields `id` and `title`. It sets the todo's title (trimmed; its done state and id
stay) and answers with the re-rendered page (`200`). A bad id (missing, not a number, zero or negative) is
`400` with `bad id`; a blank title is `400` with `title required`. An id that matches no todo just changes
nothing and still answers `200`. Put the database part in a function `rename_todo(db, id, title)`.
