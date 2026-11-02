# Page size as a parameter

The page size of 20 is hard-coded in several places in the models: the page count, the page offset
and the `limit` of the page query. Make the size a parameter of those functions and of the search
instead (the default of 20 is then written in exactly one place), and let clients choose it:

- `GET /p/{KEY}` and `GET /p/{KEY}/issues.txt` accept a query value `per_page`, read and cleaned like
  the other query values. Empty or absent means 20. Otherwise it must be 1 to 9 ASCII digits with a
  value from 1 to 100, else the answer is `400` with the body `bad per_page`. It is checked after the
  existing `status`, `priority` and `page` checks, so `page=0&per_page=0` is still `bad page`.
- The page of issues, the page count and the pager use the size: with 21 issues and `per_page=10`,
  page 3 holds one issue and the pager reads `Page 3 of 3 (21 issues)`.
- The pager's prev and next links keep the size: the link query gets `per_page={n}` right before
  `page=` whenever the size is not 20, for example `status=open&per_page=5&page=2` (escaped in the
  HTML like the rest of the link). With size 20, given or not, links are exactly as before.
- A page past the end is still an empty page, not an error. The filter form on the project page is
  unchanged, and `GET /p/{KEY}/issues.csv` is unchanged (never paginated).
