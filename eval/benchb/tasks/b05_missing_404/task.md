# Toggling or deleting a todo that does not exist should say so

`POST /toggle` and `POST /delete` with a well-formed id (greater than 0) that matches no todo currently
re-render the page with `200`, as if something had happened. Answer `404` with the body `no such todo` instead.
A bad id (missing, not a number, zero or negative) is still `400` with `bad id`, and toggling or deleting an
existing todo works as before.
