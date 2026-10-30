# Record when a todo was created

Store a creation time for every todo: a new text column `created_at` in table `todos`, set by the database
when the row is inserted, in UTC as `YYYY-MM-DD HH:MM:SS` (SQLite's `datetime('now')`). Show the date on the
HTML page after the title as `<small>added YYYY-MM-DD</small>`. `/todos.txt` does not change. Toggling must not
touch the creation time. A fresh database is fine, no migration needed.
