# Blank titles: 422, not 400

`POST /add` with a blank title (empty or only whitespace) currently answers `400` with the body `title required`.
Change it to answer `422` with the body `title must not be blank`. The other `400` answers (`bad id` on
`/toggle` and `/delete`) stay as they are, and a valid add still works as before.
