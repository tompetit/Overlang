# Read and delete one comment

Add two routes that address a single comment through its issue. `{C}` is a path segment of ASCII
digits with value >= 1, read like `{N}` (so `007` is 7); a segment that is not, such as `x` or `0`,
falls through to 404 `not found`. `{C}` is the comment's `id` as stored (the database ids, which
are unique across all issues and projects and are not reused after a delete).

- `GET /p/{KEY}/{N}/comments/{C}.txt`: text, the comment's line in the `comments.txt` format:
  `{author}: {body with each "\n" replaced by " "}\n`.
- `POST /p/{KEY}/{N}/comments/{C}/delete` (no form fields): deletes the comment, then answers the
  HTML issue page (re-read, so the comment is gone from it). The activity log is not changed (the
  issue's `commented` entry stays).

Checks for both, in order, the first failure being the response: unknown project 404
`no such project: {KEY}`; unknown issue 404 `no such issue: {KEY}-{N}`; then 404
`no such comment: {C}` (with `{C}` as its decimal value) when there is no comment with that id on
**that** issue. A comment that exists but belongs to another issue, of the same project or of
another project, is "no such comment" and is never shown or deleted through this issue's path; so
is one that was already deleted. Any other method on these two paths is 404 `not found`.

Nothing else changes: `comments.txt`, the issue page and JSON keep their formats (they simply no
longer include a deleted comment).
