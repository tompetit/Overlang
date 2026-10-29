# Runtime extensions

Small C shims bound from Overlang with `foreign c`. They do not depend on `rt.h`; link them alongside it
(`ov_sqlite.o`, `ov_http.o`, plus `-lgc`). `make` builds the objects, `make test` runs the tests under ASan/UBSan.
Handles are Ints; errors are negative Ints (or NULL for a Str result: how the caller sees NULL is up to the `foreign c` lowering, so check
`sqlite_error`). Returned strings are GC memory, copied by the caller.

## SQLite (`ov_sqlite`, effect `!fs`, link `sqlite3`)

```
foreign c "ov_sqlite.h" link "sqlite3" fn sqlite_open(path: Str) -> Int !fs
foreign c "ov_sqlite.h" link "sqlite3" fn sqlite_close(db: Int) -> Unit !fs
foreign c "ov_sqlite.h" link "sqlite3" fn sqlite_error(db: Int) -> Str !fs
foreign c "ov_sqlite.h" link "sqlite3" fn sqlite_exec(db: Int, sql: Str) -> Int !fs
foreign c "ov_sqlite.h" link "sqlite3" fn sqlite_exec_1s(db: Int, sql: Str, p1: Str) -> Int !fs
foreign c "ov_sqlite.h" link "sqlite3" fn sqlite_exec_2s(db: Int, sql: Str, p1: Str, p2: Str) -> Int !fs
foreign c "ov_sqlite.h" link "sqlite3" fn sqlite_exec_1i(db: Int, sql: Str, p1: Int) -> Int !fs
foreign c "ov_sqlite.h" link "sqlite3" fn sqlite_query(db: Int, sql: Str) -> Str !fs
foreign c "ov_sqlite.h" link "sqlite3" fn sqlite_query_1s(db: Int, sql: Str, p1: Str) -> Str !fs
foreign c "ov_sqlite.h" link "sqlite3" fn sqlite_query_1i(db: Int, sql: Str, p1: Int) -> Str !fs
```

The Overlang name is free; the C symbol is `ov_` plus the name above (the compiler's name-to-symbol rule for
`foreign c` decides whether these need an explicit symbol; adjust when that is settled).

- `sqlite_open` returns a handle >= 1 or -1; the reason is `sqlite_error(0)`.
- `sqlite_exec*` return rows changed or -1 (reason: `sqlite_error(db)`). `sqlite_exec` runs a whole script; the
  bound variants run only their first statement.
- `sqlite_query*` return text on success and NULL on error. Encoding: a header row of column names, then
  one row per result, all rows joined by `\x1e`, columns by `\x1f`, NULL as the empty string. A query with zero
  rows is just the header row. Split on `\x1e` first, then on `\x1f`. Values containing those two control
  characters are not escaped.
- Always bind user input with the `_1s/_2s/_1i` variants; never concatenate it into SQL.

## HTTP (`ov_http`, effect `!net`, no link library)

```
foreign c "ov_http.h" link "c" fn http_listen(port: Int) -> Int !net
foreign c "ov_http.h" link "c" fn http_port(server: Int) -> Int !net
foreign c "ov_http.h" link "c" fn http_accept(server: Int) -> Int !net
foreign c "ov_http.h" link "c" fn http_method(req: Int) -> Str !net
foreign c "ov_http.h" link "c" fn http_path(req: Int) -> Str !net
foreign c "ov_http.h" link "c" fn http_query(req: Int) -> Str !net
foreign c "ov_http.h" link "c" fn http_header(req: Int, name: Str) -> Str !net
foreign c "ov_http.h" link "c" fn http_body(req: Int) -> Str !net
foreign c "ov_http.h" link "c" fn http_respond(req: Int, status: Int, content_type: Str, body: Str) -> Int !net
foreign c "ov_http.h" link "c" fn http_close(server: Int) -> Unit !net
foreign c "ov_http.h" link "c" fn http_url_decode(s: Str) -> Str
foreign c "ov_http.h" link "c" fn http_form_value(body: Str, key: Str) -> Str
foreign c "ov_http.h" link "c" fn http_html_escape(s: Str) -> Str
```

(`link "c"` is a placeholder for "nothing extra": libc is always linked. If the spec requires `link` to name a
real library, make it optional rather than invent one. The last three are pure and carry no effect.)

- The server binds 127.0.0.1 only; `http_listen(0)` picks a free port, read it with `http_port`.
- `http_accept` blocks for one request and returns a handle >= 1. A malformed or oversized (> 1 MB body) request
  is answered with 400/413 by the shim and `http_accept` returns a negative number: loop and accept again.
- `http_respond` sends the response, closes the connection and frees the handle (use each request handle once).
  Typical loop: `accept`, route on `method`/`path`, build HTML with `html_escape`, `respond`.
- `http_form_value` reads a form body or a raw query string (`http_query`) and returns the decoded value.
