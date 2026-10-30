# Report database write failures

`add_todo`, `toggle_todo` and `delete_todo` return -1 when the database refuses the write, but the routes ignore
it and answer with the page as if all was well. Make `POST /add`, `POST /toggle` and `POST /delete` answer `500`
with the body `db error` when the write fails (an error, not merely zero rows changed). Validation errors keep
their `400`s, successful writes are unchanged, and reading routes still answer `200` (empty) if the database is
broken.
