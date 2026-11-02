# Sort the issue list

The project page (`GET /p/{KEY}`) and `GET /p/{KEY}/issues.txt` always list issues by number. Add a query
parameter `sort` choosing the order: `number` (the default, also when `sort` is absent or empty), `priority`,
`title` or `assignee`, each optionally prefixed with `-` for descending order.

- `priority` ascending puts P1 first. `title` and `assignee` (the assignee's user name) compare code point by
  code point, case-sensitive, so `Zebra` sorts before `apple`. With `assignee`, unassigned issues come after
  all assigned ones in both directions.
- Issues that tie on the sort key are always ordered by number ascending, in both directions.
- Sorting happens before pagination: page 2 continues the sorted order. Filters work as before.
- Any other value is `400` with the body `bad sort`. This check comes after the existing `status`,
  `priority` and `page` checks, so those errors win.
- The project page writes the sort back. The pager links' query gets `sort={value}` after `q` and before
  `page` (for example `/p/WEB?status=open&sort=-priority&page=2`), left out when the sort is `number`. The
  filter form gets a sort input right after the search input, so it reads
  `<input name="q" value="" placeholder="search"> <input name="sort" value="-priority" placeholder="sort"> <button>filter</button>`
  (`value` is empty when the sort is `number`).

The CSV export keeps number order. Nothing else changes.
