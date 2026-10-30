# Bug: stray spaces around titles

Adding a todo with spaces around the title (for example `  buy milk  `) stores them, and `/todos.txt` shows
`1 [ ]   buy milk  `. Titles should be stored trimmed. Find and fix it. Titles that are blank after trimming are
still rejected, and spaces inside a title stay.
