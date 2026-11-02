# One error helper, and JSON errors for JSON paths

Every handler builds its own error responses with `resp_error(status, message)`. Send every error
response through one shared function that also knows what was requested, so that clients of the JSON
route get errors they can parse:

- When the last segment of the request path (split as the router does: on `/`, empty segments
  dropped, so a trailing slash does not matter) ends in `.json`, an error response has the content
  type `application/json` and the body `{"error":` + the message as a JSON string (the same escaping
  as the issue JSON) + `}` and a newline. For example `GET /p/WEB/9.json` with no issue 9 answers
  `404` with `{"error":"no such issue: WEB-9"}` and a newline. This holds for every error status,
  including the catch-all `not found` for an unknown path or method.
- The match is case-sensitive (`.JSON` does not count) and looks at the path only, never the query
  string or an earlier segment.
- Status codes and messages are unchanged. Every other request gets its errors exactly as before
  (`text/plain; charset=utf-8`, the message and a newline), and successful responses are unchanged
  everywhere.
