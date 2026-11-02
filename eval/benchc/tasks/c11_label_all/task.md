# Filter by several labels

The `label` filter of the project page (`GET /p/{KEY}`) and of `GET /p/{KEY}/issues.txt` takes one label
name. Let it take a comma-separated list and match only the issues that carry every listed label (AND).

- The value (read and cleaned like today) is split on `,`, each piece is trimmed, empty pieces are dropped,
  and a name given more than once counts once (its first occurrence keeps its place). No names left means no
  label filter; one name behaves exactly as today.
- Every name must be a valid label name (the rule used when creating a label), else `400` with the body
  `bad label`. Otherwise, more than 3 distinct names is `400` with the body `too many labels`. These checks
  come after the existing `status`, `priority` and `page` checks, so those errors win.
- A well-formed name that is not a label of the project matches nothing (`200`, no issues), as today.
- The pager's `({total} issues)` and page count, and which issues appear on which page, follow the new filter.
- Wherever the project page writes the filter back (the `label` input's `value` and the prev/next links), it
  writes the cleaned list joined by `,`: `?label=ui,+bug,ui` gives `value="ui,bug"` and links containing
  `label=ui%2Cbug` (the comma is url-encoded like every other character outside `A-Z a-z 0-9 - _ . ~`).

Nothing else changes: the other filters, the CSV export, and the issue label routes
(`POST /p/{KEY}/{N}/labels` and `/labels/remove` still take one label name).
