# Split the create-issue handler, with a shared title limit

`h_create_issue` has grown into one long function: it reads the form, validates every field, looks
up the assignee and the labels, and creates the issue. Split it: a helper reads and validates the
new-issue form and gives back either the cleaned values (title, body, priority, assignee id, label
ids) or the error response, and `h_create_issue` only calls it, creates the issue and answers the
issue page. Call the helper `rt_issue_form` in Overlang and `_issue_form` in Python.

While doing so, add one rule that creating and editing share (one function checks a title and both
handlers use it):

- A title longer than 120 code points is rejected with status `422` and the body `title too long`.
  Length is counted on the cleaned value (after the usual line-field cleaning and trimming), so
  surrounding spaces do not count, and a title of exactly 120 code points is accepted however many
  bytes it takes in UTF-8.
- `POST /p/{KEY}/issues`: the check comes right after `title required` and before every priority,
  assignee and label check (so a too-long title with a bad priority is `title too long`).
- `POST /p/{KEY}/{N}/edit`: the check applies only when `title` is non-empty (an empty title still
  keeps the current one), and comes before the priority checks.
- A rejected request changes nothing: no issue is created, no field of an edited issue changes, and
  no activity is logged.

Unknown projects and issues keep answering `404` first, and everything else over HTTP is unchanged.
