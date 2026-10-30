# Limit title length

Titles longer than 80 characters (after trimming) should be rejected by `POST /add` with `400` and the body
`title too long`. Exactly 80 is fine. Count characters, not bytes: a title of 80 `é` is allowed. Nothing is
stored when a title is rejected.
