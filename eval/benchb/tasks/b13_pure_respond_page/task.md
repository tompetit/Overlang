# Make respond_page pure

`respond_page(db)` reads the todos from the database itself, which makes it (and every route that uses it)
depend on the database for something that is just rendering. Change it to take the todo list instead:
`respond_page(todos)` takes the list of todos and returns the page `Response` without touching the database;
`route` fetches the list with `list_todos(db)` where it needs it. No change in behavior over HTTP.
