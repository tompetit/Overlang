# Priority ranges in the filter

The `priority` filter of the project page (`GET /p/{KEY}`) and of `GET /p/{KEY}/issues.txt` takes one
priority. Let it also take a range `A-B`, matching issues with `A <= priority <= B`.

- Accepted values (after the usual cleaning): empty (no filter), `N`, or `A-B`, where `N`, `A` and `B` are each
  what is accepted today for a single priority (a number from 1 to 4) and `A <= B`. Anything else, for example
  `3-1`, `0-2`, `1-5`, `1-`, `-2`, `1-2-3` or `1 - 2`, is `400` with the body `bad priority`, at the same point
  of the checks as today.
- The pager's `({total} issues)` and page count, and which issues appear on which page, follow the filter.
- Wherever the project page writes the filter back (the `priority` input's `value` and the prev/next links),
  a range is written as `A-B`, except that a range with `A == B` is written as the single number `A`:
  `?priority=1-2` gives `value="1-2"` and links containing `priority=1-2`; `?priority=3-3` gives `value="3"`
  and `priority=3`.
- Writes are unchanged: `POST /p/{KEY}/issues` and `POST /p/{KEY}/{N}/edit` still take a single priority,
  and a range there is `400` `bad priority` with nothing stored.

Nothing else changes.
