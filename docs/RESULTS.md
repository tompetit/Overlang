# Results

Status, 2026-11-02: the evaluation in [PLAN.md](PLAN.md) section 8 has been
run with three models (Haiku, Sonnet and Opus 5.5) as the agent, Sonnet three
times, on four benchmarks: A (HumanEval, 100 tasks), A-hard (30 harder
function-level tasks, written for this run), B (25 maintenance tasks on the
178-line todo app) and C (40 maintenance tasks on the 1,464-line tracker,
built for this run). The agents were subagents of an interactive session
(`eval/README.md`, "Driving the agent from an interactive Claude Code
session"); every run's record is in `eval/results/*.jsonl` and every
transcript in `eval/runs/` (not committed).

## Headline

| Measure | Prediction | Result | |
|---|---|---|---|
| A: pass rate | Overlang within 10 points of Python, above Rust | at the ceiling for every model: Sonnet 99.7 / 100 / 99.0 over 3 runs; Haiku and Opus 100 / 100 / 99 | met, uninformative |
| A: compile-fix rounds | Overlang below Rust | 0 to 3 failed test rounds per 300 runs in every language | ceiling |
| A: total tokens | Overlang 20% to 50% above Python | +9% in total; the card is the difference | met |
| B: read tokens | Overlang 30% or more below Python | 2.26x above Python over 75 runs; the same with Haiku and Opus; the same with `ov ctx` as the only read path | failed |
| B: pass rate | Overlang at or above Python | Sonnet 75/75 vs 75/75; Haiku 25/25 vs 25/25; Opus 23/25 vs 25/25 | met for two models |
| C: read tokens on a 10x program | (no prediction written in advance) | Overlang 1.9x Python (Sonnet), 1.3x (Opus), 1.5x (Haiku); agents still enumerate every name | locality not observed |

The one-line reading: the correctness levers the design bet on (mandatory
signatures and effects, no null, exhaustive match, contracts) cannot be
measured because every model passes everything in every language, and the
locality lever did not fire on a small program because agents read the whole
program whatever tool they are given. The 10x program is the test that was
missing; its numbers are in the Benchmark C section.

## Benchmark A: HumanEval, 100 tasks

Medians per run. "Fresh after turn 1" is the input the agent pulled in
itself (the card, the files it read), excluding the shared system prompt.

| Sonnet, 3 runs | Python | Rust | Overlang |
|---|---|---|---|
| n | 300 | 300 | 297 |
| pass rate | 100.0% | 99.0% | 99.7% |
| failed test rounds, total | 1 | 1 | 0 |
| turns | 3 | 3 | 3 |
| output tokens | 400 | 451 | 512 |
| fresh after turn 1 | 1,426 | 1,604 | 8,309 |

| 1 run each | Haiku P / R / Ov | Opus P / R / Ov |
|---|---|---|
| pass rate | 100 / 99 / 100 | 100 / 99 / 100 |
| failed test rounds, total | 3 / 0 / 0 | 1 / 0 / 7 |
| output tokens | 1,642 / 1,846 / 2,179 | 526 / 551 / 596 |

The Rust failure is HumanEval_92, whose upstream test asserts both outcomes
for one input; it fails for every model. The one Overlang failure in 297
Sonnet runs (HumanEval_95, run 3) is a wrong answer that passed the visible
test, with no failed round. The Opus Overlang runs had 7 failed test rounds
across 99 tasks, all recovered; Sonnet had none.

## Benchmark A-hard: 30 harder function-level tasks

Written because HumanEval saturates: parsers, state machines, interval and
graph problems, dynamic programming, encodings, with 40 to 120 line
solutions and 10 to 21 hidden assertions generated from one case table per
task (`eval/tasks/ahard/`).

| | Haiku P / R / Ov | Sonnet P / R / Ov | Opus P / R / Ov |
|---|---|---|---|
| pass rate (n=30) | 93.3 / 100 / 100 | 100 / 100 / 96.7 | 100 / 100 / 100 |
| failed test rounds, total | 3 / 0 / 0 | 0 / 1 / 0 | 0 / 0 / 6 |
| turns | 4 / 4 / 5 | 3 / 3 / 4 | 3 / 3 / 3 |
| output tokens | 5,527 / 6,687 / 11,994 | 1,116 / 1,498 / 2,265 | 1,307 / 1,918 / 2,145 |

Still the ceiling. Haiku's two Python failures and Sonnet's one Overlang
failure are wrong answers that passed the visible test. Haiku writes twice
as many output tokens in Overlang as in Python, mostly test blocks and
retries of the card's idioms; Sonnet and Opus do not. Function-level tasks
do not separate these languages for these models, at any difficulty the
authors managed to write.

## Benchmark B: 25 maintenance tasks on the todo app

| Sonnet, 3 runs | Python | Overlang | Overlang, `ov ctx` only (1 run) |
|---|---|---|---|
| n | 75 | 75 | 25 |
| pass rate | 100% | 100% | 92% |
| failed test rounds, total | 1 | 3 | 3 |
| turns | 5 | 5 | 4 |
| output tokens | 1,241 | 1,298 | 1,281 |
| read tokens | 5,737 | 12,942 | 13,440 |

| 1 run each | Haiku Py / Ov / Ov-ctx | Opus Py / Ov / Ov-ctx |
|---|---|---|
| pass rate | 100 / 100 / 96 | 100 / 92 / 92 |
| turns | 7 / 12 / 8 | 5 / 6 / 6 |
| read tokens | 10,216 / 20,331 / 20,026 | 6,227 / 13,950 / 14,396 |

The ratio is 2.0x to 2.3x for every model and does not move when `ov view`,
`ov export`, `ov history` and `ov diff` are disabled and `ov ctx` is the
only way to read code. The transcripts say why: with the whole-program view
available, 15 of 25 Sonnet agents listed the store and viewed every
definition in a shell loop; with it disabled, 14 of 25 listed the store and
called `ov ctx` on every name, which costs slightly more because each slice
repeats its dependencies. On a 178-line program, reading everything is
cheap and a model that has never seen the language does it. The computed
context is not the bottleneck; the agent's strategy is, and the 8k-token
language card read once per task is the other half of the ratio (first run,
Sonnet: 214k of 371k Overlang read tokens were the card).

The forced-context runs also answer whether the computed context is
sufficient: 23, 24 and 23 of 25 for Sonnet, Haiku and Opus. The failures are
wrong answers with passing unit tests, not missing context; two of the
three Sonnet failures are on tasks (b17, b25) that also failed for Opus with
the full view.

## Benchmark C: 40 maintenance tasks on the tracker

| Sonnet, n=40 | Python | Overlang | Overlang, `ov ctx` only |
|---|---|---|---|
| pass rate | 100% | 100% | 100% |
| failed test rounds, total | 0 | 5 | 2 |
| turns | 7 | 9.5 | 12 |
| output tokens | 3,174 | 4,628 | 5,303 |
| read tokens | 13,914 | 27,018 | 30,763 |

| | Opus Py / Ov / Ov-ctx (n=40) | Haiku Py / Ov (n=40 / 27) |
|---|---|---|
| pass rate | 100 / 100 / 100 | 100 / 100 |
| turns | 7 / 10 / 11.5 | 10.5 / 16 |
| read tokens | 18,768 / 25,004 / 30,218 | 37,626 / 54,605 |

Every run of every model passed every task, including Haiku's; the tasks
the authors aimed at a 60 to 80% pass rate are at the ceiling too. The
read-token ratio on the 10x program is 1.9x for Sonnet, 1.3x for Opus and
1.5x for Haiku: narrower than the 2.2x on the todo app, because the card is a
smaller share of a bigger read, but nowhere near the predicted saving. The
attribution for Sonnet's 40 Overlang runs: 55% of read tokens were `ov view`
and `ov ls` output (66 shell loops over every bound name in 40 runs), 29%
the card, 13% files the agent wrote itself, 1% `ov ctx`. With `ov ctx` as
the only read path the agents loop over `ov ctx` instead and read more, not
less, since every slice repeats its dependencies: 49% of read tokens were
`ov ctx` output and the median run took 12 turns against Python's 7. The
Haiku set was stopped after 67 of 120 agents (its 40 Python and 27 Overlang
runs are judged; none of its forced-context runs ran) because it was taking
an hour longer than the other two.

So the locality lever did not fire at 1,464 lines either. Agents given a
program they have never seen the language of enumerate it; the compiler's
computed context is not what decides how much they read. What would change
that is either a program too big to enumerate (the Python agents read 16k
tokens per task here, so "too big" means tens of thousands of lines), a
model that has learned to ask for slices, or a workspace that returns
slices by default and makes enumeration the thing that costs an extra
step.

The program: an issue tracker over SQLite with users, projects, issues,
labels, comments, an activity log, filters, search, pagination, text, JSON
and CSV endpoints and 25 routes, specified in `examples/tracker/SPEC.md`
(1,083 lines) and built from that spec by eight parallel agents, four per
language, then assembled by two more: 1,464 lines of Overlang in four
layers and 1,551 lines of Python, byte-identical over a 100-request smoke
run. The 40 tasks (`eval/benchc/tasks/`) were written against the spec
before either implementation existed, weighted toward schema changes and
cross-module features; reference solutions in both languages pass every
check.

## What the bug hunt found

Eight Opus agents, one per compiler area, probed the compiler with programs
and reported only bugs they reproduced; a second agent tried to refute each
one. 56 reported, 38 confirmed, 18 refuted. The confirmed ones, by area:
effects 7 (an effect-variable escape that let an effectful lambda run in a
pure function, and let the fuzzer execute effects), parser 8 (chained
comparisons accepted on the right of `&&`, `result` bindable inside
`ensures`, stack overflow at 2,400 nested parentheses), store 6 (a hash that
depended on an unrelated global's name, a stored constructor clash accepted),
typechecker and codegen 5 (polymorphic and non-regular recursion hanging the
build), runtime 5 (invalid UTF-8 breaking string invariants, NUL bytes
truncating SQLite results), exhaustiveness 3 (exponential time on wide
tuples), contracts 4. Each got a regression fixture before its fix; the fixes
are on one branch per area and merged as their test suites pass. This is the
first time the compiler was probed adversarially: the 2,090 evaluation runs
before it had found no compiler bug, because agents write the same ordinary
programs.

## Language changes made in this pass

- The card is lazy: `ov card` prints a 1,500-token core, `ov card <topic>`
  one of 16 topic sections, and `ov ctx` appends the sections for the
  constructs in what it printed (SPEC section 10.4). Not yet used by the
  benchmark workspaces, which still hand the agent the full card.
- SQL is checked at `ov add`: `schema` declarations are store definitions,
  `sql "..."` literals are checked against them for unknown tables and
  columns with ranked fixes, and a function whose SQL mentions a table is a
  dependent of that table's schema, so editing the schema re-checks it
  (SPEC section 6.1). The todo app uses it; the tracker does not yet.
- A home page generated by an Overlang program through a 197-line HTML
  builder (`examples/site/`), served from `docs/` by GitHub Pages.

## What this does and does not show

- The pass-rate and compile-fix predictions are met trivially at every
  model size and task difficulty tried. If a language's guardrails help an
  agent that would otherwise fail, the failure has to come from somewhere
  these benchmarks do not reach: longer programs, more state, worse
  specifications.
- Locality on a small program is a strategy problem, not a tool problem. No
  agent in 175 Overlang maintenance runs asked the compiler for a slice and
  stopped there; they all enumerated. Benchmark C is the test of whether
  that changes when enumeration costs 10x more.
- The card tax is real, constant per task, and the largest single cost of
  a language with no training data. The lazy card halves it on paper; it
  has not been measured in a run.
- Three runs of Benchmark B narrow the read-token ratio to 2.26x with
  medians within a few percent of the single run. The single-run numbers
  elsewhere are of that reliability.

## Caveats on the method

- Every agent carried the interactive session's system prompt, identical
  across languages. It cancels in comparisons and inflates every absolute
  total; "fresh after turn 1" and "read tokens" exclude it.
- No `--max-turns` or timeout applied; no run needed either.
- The hidden tests sit in a sibling directory of the agent's workspace; a
  scan of the first 349 transcripts found no tool call that mentions it.
- Benchmark A-hard and C tasks were written by Opus agents and validated
  only by their reference solutions passing and their stubs failing; a task
  the reference and an agent both misread would pass undetected.
- Workflow-driven subagents may differ from `claude -p` in system prompt and
  tool set. All languages ran the same way, so comparisons hold; absolute
  numbers are not comparable to a `claude -p` run.

## What it cost

2,397 judged evaluation runs (307 of them Benchmark C), about 490M tokens as measured by
the harness, 85% of them cache reads of the system prompt: 221M on Sonnet,
142M on Haiku, 95M on Opus. At list rates that is roughly $200 for Sonnet,
$40 for Haiku and $420 for Opus. The build and authoring agents of this
pass (tracker, tasks, solutions, bug hunt and fixes, language changes) used
about 19M tokens of Opus, mostly uncached. The ledger in [PLAN.md](PLAN.md)
has the lines.

## What was built, and what it cost

Built between 2026-10-29 and 2026-11-02 by Claude agents (Opus and Sonnet
implementing, Fable overseeing and writing the specs), on a plan budgeted at
$12,300 in tokens.

- A compiler, `ov`, in Rust: lexer, parser, canonical printer, bidirectional
  typechecker with one effect parameter, exhaustiveness checking,
  monomorphization, C emission, build with a per-definition object cache
  (a rebuild after editing one function takes under 0.1 s).
- A C runtime with Boehm GC, plus SQLite and HTTP shims.
- A content-addressed SQLite store: definitions keyed by the SHA-256 of their
  canonical form, names as metadata, propagation of edits to dependents,
  `ov ctx` with syntax notes, `schema` and checked `sql` literals.
- Contracts checked at runtime and fuzzed by generated per-type generators
  with shrinking, run by `ov test`.
- JSON-only diagnostics with ranked fixes.
- Programs: the 178-line todo app, the 1,464-line tracker, the site
  generator, with Python ports of the two apps.
- An evaluation harness with 155 HumanEval tasks in three languages, 30
  harder tasks, 25 todo and 40 tracker maintenance tasks.

## What the agents said about writing Overlang

From the todo app README, the translation notes, the tracker builders'
notes (`examples/tracker/LIMITS.md`) and the maintenance-task reports:

- Missing conveniences dominated the friction: `count`, digit and base
  helpers, character classes, `break`/`continue`, whitespace split, string
  interpolation. The first two revisions added most of these.
- The store workflow cost an extra round after a type change until
  dependents were re-checked on edit (fixed in the third revision).
- The tracker builders hit two limits: a `++` chain of about 50 operands
  overflows the stored AST's depth limit (they used `join` over a list), and
  the linked SQLite 3.40 lacks ordered `group_concat`.
- No agent hit a compiler bug in ordinary use; the adversarial hunt found
  38 in an afternoon.
