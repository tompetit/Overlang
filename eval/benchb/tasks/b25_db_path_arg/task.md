# Choose the database file on the command line

The server always uses `todo.db` in the current directory. Let the program take an optional second command-line
argument after the port: `<program> 8080 /path/to/other.db` opens that file instead (it is created if missing).
Without it, nothing changes (`todo.db`). Add a pure function `db_path_from(argv)` that takes the list of
command-line arguments (port first, as `port_from` receives them) and returns the database path, and use it in
`main`.
