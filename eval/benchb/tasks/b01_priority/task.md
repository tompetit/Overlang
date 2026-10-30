# Priorities

Todos need a priority. `POST /add` takes an optional form field `priority`: `1`, `2` or `3` (3 is the most
urgent). Missing or empty means `2`. Anything else is `400` with the body `bad priority`, and nothing is
stored. Show it in both listings, right after the done mark: `/todos.txt` lines look like `2 [x] P3 pay rent`
and the HTML page shows `[ ] P3 pay rent` the same way. Store it in the database (a fresh database is fine,
no migration of existing files needed). Toggle and delete are unchanged.
