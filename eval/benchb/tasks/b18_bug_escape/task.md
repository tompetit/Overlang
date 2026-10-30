# Bug: markup in a title runs on the page

A tester added a todo titled `<script>alert(1)</script>` and the script ran when the page loaded; a title with a
quote or an ampersand also renders wrongly. Audit how todo text gets into the HTML page and fix it, so that
titles show up literally. `GET /todos.txt` is plain text and must keep the raw title.
