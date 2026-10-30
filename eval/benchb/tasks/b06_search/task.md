# Search

Add `POST /search` with a form field `q`. It answers `200`, `text/plain`, in the same line format as
`/todos.txt` (original ids, done marks), listing only the todos whose title contains `q`, ignoring case. No
match gives an empty body. A blank `q` (empty or whitespace) is `400` with the body `query required`.
