# Count route

Add `GET /count`: plain text (`text/plain`), status `200`, body `<total> todos, <done> done` followed by a
newline, for example `3 todos, 1 done`. With no todos it is `0 todos, 0 done`. Always the word `todos`, no
singular form.
