# Tracker (Python port)

The Python port of [../tracker/SPEC.md](../tracker/SPEC.md): the same HTTP issue tracker over SQLite
as the Overlang version in `../tracker/src/`, with the same names, SQL and responses. It uses only the
standard library and needs Python 3.10 or later.

| File | Lines | Spec | Tests inside |
|---|---|---|---|
| `db.py` | 279 | 3.1 SQLite shims, params encoding, schema | `_TestDb` (5.1, 4 tests) |
| `models.py` | 422 | 3.2 records, validators, queries | `_TestModels` (5.2, 5 tests) |
| `render.py` | 298 | 3.3 HTML, markdown-lite, text/JSON/CSV | `_TestRender` (5.3, 5 tests) |
| `main.py` | 454 | 3.4 routes, `http.server` | (none) |
| `test_tracker.py` | 98 | 5 | `TestRoutes` (5.4, 5 tests), plus the three above |
| total | 1,551 | | 19 |

## Run and test

```
python3 main.py [db_path] [port]       # defaults tracker.db 8080; prints "listening on http://127.0.0.1:<port>"
python3 -I -m unittest                 # 19 tests (or: python3 -m unittest -v test_tracker)
../tracker/smoke.sh python3 main.py    # section 5.6, once smoke.sh exists
```

The arguments are the database first, then the port, as in spec section 2 and the smoke script.

## Where this port differs from the spec's letter

- **Test layout.** Spec section 5 puts all four test classes in `test_tracker.py`. Each module builder
  wrote its tests inside its own module as a private class (`_TestDb`, `_TestModels`, `_TestRender`), so
  `from x import *` does not pass them on. `test_tracker.py` imports them under the spec's names
  (`TestDb`, `TestModels`, `TestRender`). Renaming one of those classes breaks that import.
- **JSON example.** Spec 5.3 `t_r_text_json` calls `issue_json` on `iss` (priority 2) but expects the
  section 2.3 example, which says `"priority":3`. Both ports write the issue's real priority and their
  tests expect `"priority":2`. The example in section 2.3 is what needs fixing.
- **Unbound `?` are NULL.** Python's `sqlite3` refuses a call when the number of values differs from the
  number of `?`. `db._run` reads the two counts from that error message and retries with NULLs added.
  If a future Python changes the wording, a call with too few values returns -1 instead. The models never
  pass too few.
- **First statement only.** `db._first_statement` cuts the SQL at the first `;` that
  `sqlite3.complete_statement` accepts, so a `;` inside a string literal does not count.
- **Not covered by the spec.** `create_issue` returns -1 if the insert fails, and `search_issues`
  counts a failed count query as 0 issues. Neither happens in normal use.
- **Private helpers.** `main._priority` holds the shared 400/422 priority check for routes 11 and 19.
  `main._with_label` holds the shared code for routes 23 and 24.

## HTTP parity with the Overlang build

Check scripts run against both ports, so where the spec leaves room, this port does what the C shims and
the Overlang prelude do. A differential run of 343 requests against both servers matched on status,
Content-Type and body. It covered every route, every error path and markdown edge cases. To get that
match, these four choices were needed:

- **Trim is ASCII.** Spec 0 equates prelude `trim` with `str.strip()`, but `strip()` also removes Unicode
  spaces such as U+00A0 and U+2003. All trims use `strip(_WS)` with `_WS = " \t\n\r\v\f"` (defined in
  `models.py`), the prelude's set. This applies to `clean_line`, `clean_text`, `parse_label_names`,
  `md_render` blank lines, and the `models` asserts. Without it, `name=%C2%A0carl` creates user `carl` in
  Python and gives `bad name` in Overlang.
- **A NUL ends a form value.** `ov_http_form_value` works on C strings, so `%00` cuts the value off at
  that point (`title=a%00b` is `a`). `form_value` does the same, both before and after `parse_qsl`.
- **The port is read with `parse_int`.** `parse_args` uses the prelude's rules (optional sign, ASCII
  digits, int64 range) through `db._parse_int`, not `int()`. So `" 9000"` and `"9_000"` fall back to 8080
  and `"+9000"` is 9000.
- **Every method reaches `route`.** `http.server` answers methods it has no `do_*` for (OPTIONS, HEAD, or
  anything else) with its own 501 page. `Handler.__getattr__` sends every method to `route` instead, so
  they get 404 `not found`, as in the C shim.

Known differences that remain, all outside what the check scripts compare:

- **Invalid UTF-8 in percent escapes** (`title=a%FFb`). The Overlang build stores the raw byte. Python
  decodes it as U+FFFD, because `sqlite3` cannot bind a `str` that is not valid UTF-8. Valid UTF-8 is
  identical in both ports.
- **Headers.** The C shim writes `HTTP/1.1 422 Status` and only Content-Type, Content-Length and
  Connection. `http.server` writes `HTTP/1.0 422 Unprocessable Entity` and adds `Server` and `Date`.
  Status codes, Content-Type and bodies are the same.
- **Request limits.** The C shim rejects headers over 64 KiB (431) and bodies over 1 MiB (413), and
  times out a silent client after 10 s. `http.server` has its own limits.
