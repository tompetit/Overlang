# Results

Status: no agent runs yet. The harness, the tasks, and the predictions are
in place; the headless `claude` login on the build machine has to be
refreshed before Benchmark A or B can run. This file is filled in from
`eval/report.py` output once they have.

## What was built, and what it cost

Built between 2026-10-29 and 2026-10-30 by Claude agents (Opus and Sonnet
implementing, Fable overseeing and writing the spec), on a plan budgeted at
$12,300 in tokens. The ledger in [PLAN.md](PLAN.md) section 10 has the
per-agent token counts.

- A compiler, `ov`, in Rust: lexer, parser, canonical printer, bidirectional
  typechecker with one effect parameter, exhaustiveness checking,
  monomorphization, C emission, build with a per-definition object cache.
- A C runtime with Boehm GC, plus SQLite and HTTP shims.
- A content-addressed SQLite store: definitions keyed by the SHA-256 of their
  canonical form, names as metadata, propagation of edits to dependents, and
  `ov ctx`, which hands an agent the minimal context for one definition.
- Contracts checked at runtime and fuzzed by generated per-type generators
  with shrinking, run by `ov test`.
- JSON-only diagnostics with ranked fixes.
- A 178-line HTTP todo app over SQLite, the first real program, with a
  Python port for the maintenance benchmark.
- An evaluation harness with 155 HumanEval tasks in Python, Rust and
  Overlang, and 25 maintenance tasks on the todo app in Python and Overlang.

Compiler bugs found by the two translation passes, the app and the
maintenance tasks: zero. Bug found by the fuzzer in the spec's own example:
one (an Int overflow the contract did not exclude).

## Predictions, written before any run

| Measure | Prediction | Result |
|---|---|---|
| A: pass rate | Overlang within 10 points of Python, above Rust | |
| A: compile-fix rounds | Overlang below Rust | |
| A: total tokens | Overlang 20% to 50% above Python | |
| B: read tokens | Overlang 30% or more below Python | |
| B: pass rate | Overlang at or above Python | |

Preliminary, before any run: on the todo app `ov ctx` returns a mean of 4.4%
of the file per definition (24% for the hub function `route`), which bounds
what the locality lever can save (PLAN.md section 6).

## Benchmark A: single-function tasks

Not yet run.

## Benchmark B: maintenance tasks

Not yet run.

## What the agents said about writing Overlang

Collected from the todo app README, the translation notes and the
maintenance-task report, before any measured run:

- Missing conveniences dominated the friction: `count`, digit and base
  helpers, character classes, `break`/`continue`, whitespace split, string
  interpolation. The first two revisions added most of these.
- The store workflow cost an extra round after a type change until
  dependents were re-checked on edit (fixed in the third revision).
- No agent hit a compiler bug. The C backend, the typechecker and the store
  held up under about 250 probes and 180 validated programs.
