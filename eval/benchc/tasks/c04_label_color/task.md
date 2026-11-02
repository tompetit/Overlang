# Label colours

Labels get a colour. Store it in a new `color` column of the `labels` table, always in the form
`#rrggbb` with lowercase hex digits; a label created without one gets `#888888` (a fresh database
is fine; no migration of existing files is needed).

**Valid colours.** After trimming (colours are line fields), `#` followed by exactly six hex digits
(`0-9`, `a-f`, `A-F`). It is stored lowercased. Anything else is 422 `bad color`.

**Writing it.**

- `POST /p/{KEY}/labels` takes an optional form field `color`. Empty or absent means `#888888`. It
  is checked after the existing checks (`name required`, `bad label`, `label exists`).
- New route `POST /p/{KEY}/labels/{NAME}/color` with the form field `color`. Checks in order: the
  usual unknown project 404; 404 `no such label: {NAME}` when the project has no label of that
  name; 422 `color required` when `color` is empty; 422 `bad color`. It changes that label only
  (other projects may have a label with the same name) and answers with the HTML project page with
  `no_filter()`. Any other method on this path is 404 `not found`.

**Showing it.** A label "chip" is `<span class="label" style="background:{color}">{name}</span>`.

- `labels_text` (`/p/{KEY}/labels.txt`): `{name} {color}` per label: `bug #888888`.
- Everywhere the HTML pages showed label names through `labels_display` (the labels cell of each
  issue row on the project page, the `Labels:` part of the issue page meta line, and the
  `<p class="labels">` paragraph of the project page), show chips instead, joined by `, ` in the
  same order, and still `-` when there are none. An issue's chips use the current colour of its
  own project's labels.
- The add-label form of the project page gets ` <input name="color" placeholder="#rrggbb">` right
  after its `name` input.
- Issue JSON: a key `"label_colors"` right after `"labels"`: the colours in the same order as the
  names: `..."labels":["bug","ui"],"label_colors":["#ff0000","#888888"],"comments":...`.

`issue_line`, the CSV and the label filter do not change; they still use the names only.
