# Filter /todos.txt with a query string

`GET /todos.txt?done=1` should list only the done todos and `GET /todos.txt?done=0` only the open ones, in the
usual line format. No `done` parameter, or any other value, lists everything as before. Other parameters are
ignored. Right now the query string is cut off before routing, so `route` never sees it: pass it through.
The other routes ignore the query string.
