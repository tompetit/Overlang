# Overlang evaluation harness

Implements Benchmark A of `docs/PLAN.md` section 6 (single-function tasks, same
problems in Python, Rust and Overlang). Standard library only (Python 3.14).
The Overlang adapter is a stub until the compiler works; everything else is
testable now with a mock agent.

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

`overlang.py` documents what the real adapter will do (`.ov` stub, `ov check` /
`ov test` as the test command, grading with `ov run` on a fixture-style
program) and raises `NotImplementedError("compiler not ready")`; `run.py`
exits with that message.

## Not done / caveats

- No real agent run has been exercised (no authentication); the stream-json
  parsing is tested against a hand-written fixture of the documented event
  shapes. Verify on the first real transcript that `Bash` `tool_result`s carry
  `is_error` for non-zero exits.
- Benchmark B (maintenance tasks) is not part of this harness yet.
- The hidden tests are in a sibling directory of the agent's cwd, not
  sandboxed; an agent could in principle read `../hidden`.
