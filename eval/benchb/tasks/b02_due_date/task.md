# Due dates

Todos get an optional due date. `POST /add` takes an optional form field `due` in the form `YYYY-MM-DD`
(four digits, a dash, two digits, a dash, two digits; no need to check that the date exists). Anything else
non-empty is `400` with the body `bad due date`. Missing or empty means no due date. `/todos.txt` appends
` (due 2026-01-31)` after the title of a todo that has one; todos without one print as before. The HTML page
shows `<small>due 2026-01-31</small>` after the title. Store it in the database (fresh database is fine).
