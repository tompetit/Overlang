# Overlang evaluation harness

Implements Benchmarks A and B of `docs/PLAN.md` section 6 (A: single-function tasks, same
problems in Python, Rust and Overlang; B: maintenance tasks on one app, see the Benchmark B
section). Standard library only (Python 3.14).
Everything is testable with a mock agent; the Overlang adapter needs the built
compiler (`cargo build --release`).

## How it works

For each (task, run): `run.py` builds a fresh workspace, runs the agent in it,
grades with hidden tests, and appends one JSON line to the output file.

```
eval/runs/<out-stem>/<task>/<run>/
    work/         agent cwd: CLAUDE.md, the stub (solution.*), test_visible.*
    hidden/       hidden tests and build products (outside the agent's cwd)
    transcript.jsonl   raw agent output
```

The agent is asked to read `CLAUDE.md`, implement the stub in the one solution
file, run the visible test command until it passes, and stop. Grading never
trusts the agent's own test file: it builds the hidden tests against the final
solution file.

## Tasks

`fetch_tasks.py` writes `tasks/<benchmark>/<lang>/<task_id>.json` for benchmarks
`humaneval` (156 tasks) and `mbpp` (354 tasks), languages `python` and `rust`,
plus `tasks/index.json` listing the ids present in both languages (all of them
today). `--tasks 0:5` in `run.py` slices that shared list, so every language
gets the same problems. Fields: `id`, `benchmark`, `lang`, `name`,
`entry_point`, `prompt` (stub + docstring/doc comment), `tests` (hidden), `raw`,
and for Python `solution` (reference; HumanEval only).

Sources. Rust: MultiPL-E `humaneval-rs`/`mbpp-rs` rows from the datasets-server
rows API. Python: the rows API has no `humaneval-py`/`mbpp-py` config, so the
Python variant is the MultiPL-E original `.py` file of the same task name,
fetched from the nuprl/MultiPL-E GitHub repo. MultiPL-E has 156 of the 164
HumanEval problems (8 are dropped upstream). `tasks/_cache/` (git-ignored)
holds raw pages. Total size about 5 MB, all committed.

Known data quirks: HumanEval_151_double_the_difference fails its own reference
solution (the upstream test is wrong); MBPP has no reference solutions; the
MBPP Rust prompts read "Write a rsthon function" (upstream search-and-replace).

### Visible / hidden split

- Python (both datasets): the dataset has `check(candidate)` made of asserts.
  The first `assert` statement in `check` becomes `test_visible.py`; the full
  `check` runs hidden under `python3 -I`.
- Rust (both datasets): the dataset tests are a `fn main()` of `assert_eq!`
  lines (after a leading `}` that closes the prompt's function, which we strip).
  The first assertion becomes `test_visible.rs` (which `include!`s
  `solution.rs`); grading concatenates `solution.rs` + the full `fn main()` and
  builds it with `rustc`.

## Running

Smoke test, no model, no cost:

```
python3 -I -m unittest discover eval/tests      # harness tests, ~5 s
python3 eval/run.py --benchmark humaneval --lang python --agent mock --mock-solve \
        --tasks 0:5 --runs 1 --out eval/runs/smoke.jsonl     # all pass (reference copied)
python3 eval/run.py --benchmark humaneval --lang rust --agent mock \
        --tasks 0:5 --runs 1 --out eval/runs/smoke-rs.jsonl  # all fail (stub left)
python3 eval/report.py eval/runs/smoke.jsonl
```

Real run:

```
python3 eval/run.py --benchmark humaneval --lang python --model sonnet \
        --tasks 0:100 --runs 3 --agent claude --out eval/runs/humaneval-sonnet-py.jsonl
python3 eval/report.py eval/runs/*.jsonl
python3 eval/report.py eval/runs/*.jsonl --compare python rust   # ratios of medians, rust/python
```

Runs already in the output file are skipped, so an interrupted batch restarts
where it stopped. Timeouts: `--agent-timeout` 900 s, `--grade-timeout` 120 s.
`--help` on every script.

### The agent invocation

```
claude -p "<instruction>" --output-format stream-json --verbose --model <model> \
       --max-turns <30> --dangerously-skip-permissions --setting-sources project,local
```

with `CLAUDE_CODE_DISABLE_AUTO_MEMORY=1`, cwd = `work/`. I chose `stream-json`
over `json`: its last line is the same `result` record (cost, turns, duration,
usage), and the stream also contains every tool call, which is the only way to
count failed test rounds. `--setting-sources project,local` keeps user plugins,
hooks and settings out of the agent (without it, a personal plugin such as a
SessionStart hook leaks into every trajectory; confirmed on this machine).

Contamination risk: `claude` loads `CLAUDE.md` files from parent directories,
and `eval/runs/` is inside the repo, so the repo's own `CLAUDE.md` is seen by the
agent. For clean runs either use `--bare` (skips CLAUDE.md discovery and passes
the workspace one explicitly; needs `ANTHROPIC_API_KEY`, OAuth is not read) or
run the repo checkout somewhere with no parent `CLAUDE.md`. This is untested
(no authentication here); check the first transcript.

Unauthenticated `claude -p` exits 1 with a `result` record `is_error: true`,
`result: "Failed to authenticate: OAuth session expired and could not be
refreshed"`, zero usage. The harness records that as `agent_error`, and
`report.py` excludes such runs (unless `--include-errors`).

### Driving the agent from an interactive Claude Code session

`--agent subagent` is for a box where `claude -p` cannot authenticate but an
interactive Claude Code session can spawn subagents. The same command is run
twice. The first time it writes each workspace plus `<run>/prompt.txt` and prints
`PENDING ev <out-stem> <task> <run>`. The session then spawns one subagent per
pending run (Agent tool or a Workflow script) with that prompt and with the
description/label set to the exact key. The second time, with `--transcripts`
pointing at the session's `subagents/` directory
(`~/.claude/projects/<project>/<session>/subagents`, scanned recursively so
workflow runs count), it finds each finished transcript by key, copies it to
`<run>/transcript.jsonl`, derives the metrics from the per-message `usage`
records, grades, and appends the record. `total_cost_usd` is 0 in this mode.
Caveats: every subagent carries the session's ~50k-token system prompt
(same for all languages, so it cancels in comparisons but inflates absolute
totals); there is no `--max-turns` or timeout; the agent's cwd is the
session's, so the prompt tells it to `cd` into the workspace. This is how the
numbers in `docs/RESULTS.md` were produced.

## Metrics recorded (one JSON line per run)

`benchmark, lang, task, run, model, agent, passed, num_turns, total_cost_usd,
input_tokens, output_tokens, cache_creation_input_tokens,
cache_read_input_tokens, duration_ms, failed_rounds, test_rounds, agent_error,
grade_output` (+ `timeout: true` when the agent timed out).

`failed_rounds` = Bash tool calls whose command contains the adapter's test
marker (`test_visible`) and whose result was an error (non-zero exit). The
agent running its own ad hoc scripts is not counted. `test_rounds` is all such
calls. A timed-out run keeps `passed` from grading whatever is on disk, and its
token fields are 0 (no result record is emitted).

`report.py` prints per (benchmark, lang, model): n, pass rate, median and mean of
each token field, `total_tokens` (all four summed), `cache_hit_rate`
(cache_read / all input), cost, duration, turns and failed_rounds. `--compare A B`
prints median ratios B/A and the pass-rate difference in points, the form the
plan's predictions take.

## Adding a language adapter

Create `adapters/<lang>.py` (and add the name to `--lang` choices in `run.py`)
exposing:

| name | meaning |
|---|---|
| `TASK_LANG` | which `tasks/<benchmark>/<dir>` to read the tasks from |
| `SOLUTION` | file the agent edits |
| `TEST_MARKERS` | substrings identifying a test invocation in a transcript |
| `test_command()` | the command put in CLAUDE.md and the instruction |
| `workspace(task, dir)` | write agent-visible files into `dir` (= `<run>/work`) and hidden tests into `adapters.hidden_dir(dir)` |
| `grade(dir, timeout)` | run hidden tests, return `{"passed": bool, "output": str}`; use `adapters.run` (kills the process group on timeout) |
| `reference(task)` | full solution text for `--mock-solve`, or `None` |

### Overlang

`tasks/humaneval/overlang/<id>.json` are hand translations of 155 of the 156
shared HumanEval tasks (`HumanEval_92` is untypable; see `NOTES.md` there for what was skipped or renamed and
the compiler/language gaps found; `build.py` regenerates and validates them with
the real `ov`). Fields: the common ones plus `prompt` (`//` comment + signature
with a `todo()` body, written verbatim to `solution.ov`), `tests` (hidden `test
check_N` blocks), `visible_test` (first block only, plus float-tolerance helpers
if needed) and `solution` (reference, used by `--mock-solve`).

Workspace: `solution.ov`, `test_visible.ov`, `run_tests.sh`, `ov` (a wrapper
exec'ing the compiler), `CLAUDE.md` (= `docs/CARD.md` + rules). Test command:
`./run_tests.sh`, which runs `ov test ../solution.ov ../test_visible.ov` from
`.ovtest/` so build products stay out of the agent's directory. Overlang has no
modules, so `ov` loads the two files as one program. Grading runs
`ov test solution.ov tests.ov` in `hidden/` under the grade timeout; passed iff
exit 0 and every hidden block printed `ok <name>`. The compiler is found via
`$OV_BIN`, else `target/release/ov` of this checkout, else `ov` on `PATH`
(`cargo build --release` first). Example:

```
python3 eval/run.py --benchmark humaneval --lang overlang --agent mock --mock-solve \
        --tasks 0:156 --runs 1 --out eval/runs/smoke-ov.jsonl  # all 155 pass
```

`run.py` skips a shared id that has no task file for the language (a `skip <id>` line on stderr).

## Benchmark B: maintenance tasks

The same todo app in two languages, 25 modification tasks, graded by language-agnostic HTTP checks.
`--benchmark benchb --lang python|overlang`. Nothing is downloaded; all data is in `eval/benchb/`.

```
examples/todo/       the Overlang app (main.ov, 178 lines)      examples/todo_py/  its Python port
eval/benchb/lib.sh   helpers sourced by every check             eval/benchb/validate.py
eval/benchb/tasks/<id>/        task.md  check.sh  [probe.py probe.ov]  [setup/<lang>/...]
eval/benchb/solutions/<id>/<lang>/   reference solution (full replacement files; rm.txt = names to unbind)
```

The Python port keeps the Overlang app's function names and decomposition (`db_init`, `parse_rows`,
`list_todos`, `add_todo`, `toggle_todo`, `delete_todo`, `render_*`, `route`, `serve`, `main`), contracts as
asserts, the three unit tests in `test_todo.py` (`python3 -I -m unittest`), and a curl-driven `test.sh`.
Both apps pass their own `test.sh`.

Tasks (statement = `task.md`, which goes into the agent prompt verbatim):

| id | task | touches |
|---|---|---|
| b01_priority | priority 1..3 field, shown in both listings, `400 bad priority` | type, schema, parse, list, add, render x2, route, tests |
| b02_due_date | optional `due` YYYY-MM-DD, `400 bad due date` | same breadth, plus a validator |
| b03_notes | notes field, escaped `<small>` on the page, new `POST /notes` | same breadth plus a route |
| b04_blank_422 | blank title answers 422 with a new body | route |
| b05_missing_404 | toggle/delete of an unknown id is 404 | route, uses the rows-changed result |
| b06_search | `POST /search`, case-insensitive substring | new pure fn, route |
| b07_count | `GET /count` | new render fn, route |
| b08_clear_done | `POST /clear-done` + `clear_done(db)` | db fn, route |
| b09_rename | `POST /rename` | db fn (needs a string and an int bound), route |
| b10_tsv_format | tab-separated `/todos.txt` | `render_text` |
| b11_title_length | titles over 80 characters rejected (characters, not bytes) | route |
| b12_duplicate_title | 409 on a duplicate title, case-insensitive | new fn, route |
| b13_pure_respond_page | `respond_page(todos)` without the db | signature, 4 call sites |
| b14_split_route | `route_get`, `route_post`, `route` dispatches | whole routing |
| b15_add_ensures | postcondition on `add_todo` | contract; check looks at the trap |
| b16_soft_delete | `deleted` column, `/undelete` | schema, delete, list, route |
| b17_created_at | `created_at` column and page date | type, schema, parse, list, render, tests |
| b18_bug_escape | planted bug: title unescaped on the page | `render_todo` |
| b19_bug_toggle | planted bug: toggle never reopens | SQL in `toggle_todo` |
| b20_bug_trim | planted bug: title stored untrimmed | `add_todo` |
| b21_db_error_500 | failed writes answer 500 | route x3 |
| b22_status_cycle | `done` flag becomes open/doing/done, toggle cycles | type, schema, parse, list, toggle, render x2, tests |
| b23_max_todos | at most 20 todos, 409 `list full` | route |
| b24_query_filter | `?done=1` filter; the query must be plumbed through `route` | route signature, serve, tests |
| b25_db_path_arg | optional second CLI argument, pure `db_path_from` | `main`, new fn |

Bug tasks (b18 to b20) have a `setup/<lang>/` directory with the app files that carry the planted bug (the
same bug in both ports; the unit tests are adjusted so they do not catch it). Any file in `setup/<lang>/`
replaces the base file of the same name when the workspace is built.

### Checks

`check.sh <app dir> <lang>` (or `check_<lang>.sh`; exit 0 = pass) sources `lib.sh`, which exports the Overlang
store to a file if needed, builds (`ov build main.ov`) or runs `python3 main.py`, starts the server on a free
port with cwd = a fresh scratch directory (so `todo.db` is new), and drives it with curl. A check asserts the
new behavior, then that the old behavior holds (`bb_baseline`: add, toggle, delete, 404, blank title, restart
persistence; skipped or adapted where the task changes it), then runs the app's own unit tests (`bb_unit`:
`python3 -I -m unittest` / `ov test main.ov`). Unit-level tasks add `probe.py` / `probe.ov` (extra `test`
blocks loaded with the app); b15 builds a separate Overlang program with `--entry` to observe the trap;
b16/b21/b22 read or break the SQLite file directly with `bb_sql`. Scratch lives under `$BB_SCRATCH`
(default `<app dir>/../scratch`); servers are killed on exit.

```
python3 eval/benchb/validate.py                         # every task, both languages, ~2 min
python3 eval/benchb/validate.py --tasks b04_blank_422 --lang python -v
```

`validate.py` builds the real workspace with the adapter, requires the check to FAIL on it, applies the
reference solution and requires the check to PASS. All 25 tasks pass this in both languages.

### Workspaces

Python: `main.py`, `test_todo.py`, `CLAUDE.md` (app overview, test command). The agent edits files.

Overlang: the store is populated before the agent starts (`ov add` of the app, in the workspace), and there
is no `main.ov` anywhere the agent can reach (the seed file is deleted; the only copy of the code is
`.ovstore/`). `CLAUDE.md` is the language card plus the store workflow (`./ov ls|ctx|view|edit|add|rm|check|test`,
`./ov build main`, `./ov run main -- 8080`) and a note on shims declared in the C runtime but not in the app
(for example `ov_sqlite_exec_2s`). `./ov` is a wrapper around the compiler binary (`$OV_BIN`, else
`target/release/ov`). Grading exports the store (`ov export`) to
`hidden/export/main.ov` and runs the check on it, so a store that does not typecheck or build fails.
Comments are not kept by `ov export`, which does not matter for grading.

The prompt is "Read CLAUDE.md, then make this change to the todo app: <task.md> Run `<test command>` before
you stop." Test command: `python3 -I -m unittest` or `./ov test`. `failed_rounds` counts Bash calls with those
markers (and `ov check`) that failed.

### Running

```
python3 eval/run.py --benchmark benchb --lang python --agent mock --mock-solve --tasks 0:25 --runs 1 --out eval/runs/b-py.jsonl
python3 eval/run.py --benchmark benchb --lang overlang --agent mock --mock-solve --tasks b04_blank_422,b17_created_at --runs 1 --out eval/runs/b-ov.jsonl
python3 eval/run.py --benchmark benchb --lang overlang --model sonnet --tasks 0:25 --runs 3 --agent claude --out eval/runs/b-ov-sonnet.jsonl
```

`--tasks` is a start:stop slice of the sorted task ids or a comma-separated list of ids. `--mock-solve`
applies the reference solution (Overlang: `ov add` of the solution file into the store, then `ov rm` of
`rm.txt` names). Without it the mock leaves the task unsolved (the check must fail).

### read_tokens

Benchmark B's hypothesis is about reading. Every benchb record also has `read_tokens` and `read_calls`.
A call is read-type if it is `Read`, `Grep`, `Glob`, or a `Bash` command using `cat head tail less grep rg ls
find wc nl awk sed -n` or `ov ctx|view|ls|deps|dependents|history|diff`. `read_tokens` = sum, over assistant
turns (merged by `message.id`) that directly follow a turn containing a read-type call, of
`input_tokens + cache_creation_input_tokens` of that turn. Rationale: those are the fresh tokens the
read result adds to the context; `cache_read_input_tokens` is excluded because it re-counts the whole
conversation on every turn and would swamp the signal. Limits: it is an approximation (the fresh tokens of a
following turn also include the model's own previous message and any other tool results in the same turn),
it ignores that a long read result is then re-read from cache on every later turn, and it has only been tested
on a hand-built fixture, not on a real transcript. `report.py` prints the median and mean per language.

## Not done / caveats

- No real agent run has been exercised (no authentication); the stream-json
  parsing is tested against a hand-written fixture of the documented event
  shapes. Verify on the first real transcript that `Bash` `tool_result`s carry
  `is_error` for non-zero exits.
- Benchmark B is implemented (below) but no real agent run has been exercised either.
- The hidden tests are in a sibling directory of the agent's cwd, not
  sandboxed; an agent could in principle read `../hidden`.
