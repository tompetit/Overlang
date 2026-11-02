# Issue meta fragment

A dashboard wants to refresh an issue's status line without loading the whole page. That line is the
meta paragraph of the issue page,

    <p class="meta">Status: <span class="status">{status}</span> | Priority: P{priority} | Assignee: {assignee or -} | Labels: {labels joined by ", ", or -}</p>

(values HTML-escaped as everywhere), and today it is built inline inside `render_issue`. Move it into
its own pure render function of the issue alone, `render_issue_meta(i: Issue) -> Str` (Python
`render_issue_meta(i)`), have `render_issue` use it so the issue page stays byte for byte the same,
and serve it on a new route:

- `GET /p/{KEY}/{N}/meta` answers `200` with content type `text/html; charset=utf-8`, and the body is
  exactly that paragraph for the issue as it is now in the database: no page around it, no trailing
  newline.
- Unknown project: `404` `no such project: {KEY}`. Unknown issue: `404` `no such issue: {KEY}-{N}`.
  An `{N}` that is not an issue number (`/p/WEB/0/meta`, `/p/WEB/x/meta`, `/p/WEB/1.json/meta`), any
  other method on the path, and any longer or different path (`/p/WEB/1/meta/x`, `/p/WEB/1/metas`)
  answer `404` `not found`, as plain-text errors like the others.
- As on the other routes, a trailing slash does not matter and the query string is ignored.
