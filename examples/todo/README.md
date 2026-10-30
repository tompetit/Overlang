# Overlang todo app

An HTTP todo list backed by SQLite, in one file (`main.ov`), using the `foreign c` shims in `runtime/ext/`.

## Run

```
cargo build --release
target/release/ov run examples/todo/main.ov                            # port 8080, ./todo.db
target/release/ov build examples/todo/main.ov -o todo && ./todo 9000   # custom port
target/release/ov test examples/todo/main.ov                           # 3 unit tests (also exercises the contracts)
examples/todo/test.sh                                                  # end-to-end with curl, fresh db under examples/todo/.tmp/
```

`ov run` takes only files and forwards no program arguments, so use `ov build` to pass a port.

## Routes

| Route | Effect |
|---|---|
| `GET /` | HTML page: add form, per-todo toggle and delete forms |
| `GET /todos.txt` | plain text, one `id [x] title` line per todo |
| `POST /add` | form field `title`; 400 if blank |
| `POST /toggle` | form field `id` |
| `POST /delete` | form field `id` |
| anything else | 404 |

Mutating routes answer with the re-rendered page (200), not a redirect, since `http_respond` cannot set headers.
`route` takes the db handle as its first parameter (`route(db, method, path, body)`) because Overlang has no
globals; the 404 test passes a dummy handle `0`.

## What it was like to write this in Overlang

No compiler bugs found (no `bug_*` fixtures). Friction, most important first:

- **The card says "always pass `--json`"; `ov` rejects it** (`unexpected argument '--json'`). Diagnostics are
  already JSON. The card also shows `ov run <name>` / `ov test [name]` / `ov check [name]`; the real CLI takes files.
- **`args()` does not include the program name** and `ov run` forwards no arguments. Neither is documented. I
  guessed `[prog, port]` and the server silently listened on 8080.
- **A list-pattern rest must be a name**: `[p, ..]` and `[p, .._]` are parse errors (E0002 "expected
  identifier"), so an unused rest needs `.._rest`. A bare `..` would be natural.
- **stdout is buffered with no flush**, so `println("listening on ...")` never appears when the server is
  killed or piped. A long-running server needs a `flush()` (or line buffering). `test.sh` polls the port instead.
- **No response headers in the shim**, so no redirect-after-POST; reloading a result page re-submits the form.
- **No globals or module state**: the db handle is threaded through every function, which also makes `route`
  effectful (`!fs`).
- **No string interpolation or multi-line literals**: building HTML is a wall of `++`.
- **Prelude gaps**: no `filter_map`/`fold`-with-early-skip style helper for "parse rows, drop bad ones" (I used
  `let mut` plus `push` in a `for`), no `split_once`, no `flush`.
- **`ov test` is silent about fuzzing** when everything passes, so I cannot tell whether the effectful
  `requires` functions (`toggle_todo`, `delete_todo`, `add_todo`) were fuzzed. A failing `ensures` is reported
  well (JSON trap), though `"hash"` is empty.
- Worked first try: `foreign c ... = "c_symbol"` overrides, `Option[Str]` for NULL query results,
  `\u{1e}`/`\u{1f}` escapes, matching on a `(Str, Str)` tuple for routing, `ensures all(result, |t| ...)`.
