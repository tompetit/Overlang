# JSON issue list

Scripts that read `issues.txt` cannot see bodies or tell where the list ends. Add
`GET /p/{KEY}/issues.json`, the JSON counterpart of `GET /p/{KEY}/issues.txt`: the same query parameters
(`status assignee label priority q page`), the same filter errors (400 with the same messages: `bad status`,
`bad priority`, `bad page`), the same unknown-project 404, and the same pages of 20 issues in the same
order. Answer with content type `application/json` and a body of one compact JSON object followed by
`\n`, with keys in exactly this order:

```
{"project":"WEB","total":23,"page":2,"pages":2,"per_page":20,"prev":"/p/WEB/issues.json?page=1","next":null,"issues":[...]}
```

- `project` is the project key; `total`, `page`, `pages` are those of the page shown (the same numbers the
  HTML pager prints); `per_page` is always `20`.
- `prev` is `null` on page 1, otherwise `/p/{KEY}/issues.json?` followed by the filter query for page
  `page - 1`, built exactly like the query string of the HTML pager links (same parameter order, same
  encoding, not HTML-escaped). For example a request for `?q=a+b&status=open&page=2` gives
  `"prev":"/p/WEB/issues.json?status=open&q=a%20b&page=1"`.
- `next` is `null` when `page >= pages`, otherwise the same with page `page + 1`.
- `issues` is the page's issues; each is an object with the keys `key`, `project`, `number`, `title`,
  `body`, `status`, `priority`, `assignee`, `labels`, written exactly as in `GET /p/{KEY}/{N}.json` (same
  string escaping, `assignee` `null` when unassigned, `labels` an array), just without `comments` and
  `activity`.

A page past the last one is not an error: `issues` is `[]`, `prev` points to the page before the one
asked for and `next` is `null`. Any method other than GET on this path is 404 `not found`. Existing routes
are unchanged.
