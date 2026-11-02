# Title and body limits

Issue titles and bodies have no length limits, and titles keep whatever spacing was typed into
them. Tighten both. Lengths below are counted in code points (not bytes), on the value after all
cleaning and normalisation.

**Titles.** After the existing cleaning (a title is a line field, so it is already trimmed), every
run of one or more spaces or tabs inside the title becomes a single space. This normalised title
is what is stored, so it is what every page, text line, JSON and CSV shows. A title longer than 120
code points is 422 `title too long`.

**Bodies.** A body is cleaned as today (a text field: `\r\n` and lone `\r` become `\n`, then
trimmed) and is not otherwise changed (inner spaces and newlines stay). A body longer than 2000
code points is 422 `body too long`. This applies to issue bodies and to comment bodies.

**Where, and in which order.** Each route keeps its existing checks; the new ones go in as follows,
and the first failure is the response:

- `POST /p/{KEY}/issues`: `title required`, `title too long`, `body too long`, then the priority,
  assignee and label checks as today.
- `POST /p/{KEY}/{N}/edit`: a title or body that is empty after cleaning still means "keep the
  current value". Otherwise: `title too long`, `body too long`, then the priority checks as today.
  A non-empty new title is normalised like on create.
- `POST /p/{KEY}/{N}/comments`: `author required`, unknown author, `body required`, then
  `body too long`.

A rejected request changes nothing: no issue, no edited field, no comment, no activity entry.
Existing rows are not migrated (a fresh database is fine).
