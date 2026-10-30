# Three states instead of done/not done

Todos get a middle state. Replace the done flag with a status that is `open`, `doing` or `done`, stored in a text
column `status` of table `todos` (default `open`; the `done` column goes away). `POST /toggle` now cycles the
status: open, then doing, then done, then back to open. Marks in both `/todos.txt` and the HTML page: `[ ]` for
open, `[~]` for doing, `[x]` for done. Everything else behaves as before; a fresh database is fine.
