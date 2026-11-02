# Issues per user

Add a page per user listing the issues assigned to them across all projects.

- `GET /users/{NAME}`: an HTML page. `GET /users/{NAME}/issues.txt`: text, one issue line per issue in the
  same format as `GET /p/{KEY}/issues.txt` (for example `WEB-7 open P2 alice [bug,ui] Fix login`).
  `{NAME}` is the path segment used verbatim and must be an existing user name exactly (case-sensitive), else
  `404` with the body `no such user: {NAME}`. Both list every issue currently assigned to that user (no
  pagination), ordered by project key, then by issue number. An issue that was reassigned or unassigned no
  longer appears.
- Both take an optional query parameter `status`: empty or absent for all, `open` or `closed` to keep only
  those; anything else is `400` with the body `bad status` (checked after the user lookup).
- The HTML page is the usual page skeleton (`<!doctype html>`, the nav, ...) with the title
  `Issues of {NAME}` and the body
  `<h1>Issues assigned to {NAME}</h1><p class="count">{O} open, {C} closed</p><table class="issues">`, then the
  same header row and issue rows as the project page's table, then `</table>`. `{O}` and `{C}` count all
  issues assigned to the user, whatever the `status` parameter is. Text is HTML-escaped as everywhere else.
- On the users page (`GET /users`, and the page `POST /users` answers with), each user becomes a link:
  `<li><a href="/users/alice">alice</a></li>` instead of `<li>alice</li>`.
- Any other method on these paths, and any other path under `/users/{NAME}`, is `404` `not found`.

Nothing else changes.
