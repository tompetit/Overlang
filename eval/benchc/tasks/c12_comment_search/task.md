# Search comments too

The `q` filter of the project page (`GET /p/{KEY}`) and of `GET /p/{KEY}/issues.txt` matches issues whose
title or body contains `q`. Make it also match issues that have at least one comment whose body contains `q`.

- Same matching rule as today for every text searched: substring, ASCII letters case-insensitive, `%` and `_`
  taken literally.
- An issue appears once however many of its title, body and comments match, and the pager's
  `({total} issues)` and page count count issues, not matching texts.
- Comment authors and assignee names are not searched. Every other filter combines with `q` as today.

Also add `GET /p/{KEY}/comments.txt?q=...`: a text response (`text/plain; charset=utf-8`) with one line per
comment of that project whose body contains `q` (same rule; only comment bodies are searched), oldest comment
first, each line `{issue key} {author}: {body}` followed by `\n`, where `{issue key}` is like `WEB-3` and each
newline in the body is written as a space. For example `WEB-3 bob: looks fine`. No match is `200` with an
empty body. An unknown project is `404` `no such project: {KEY}` (checked first); a `q` that is empty after
the usual cleaning is `400` with the body `q required`. Any other method on that path is `404` `not found`.

Nothing else changes (the per-issue `GET /p/{KEY}/{N}/comments.txt` included).
