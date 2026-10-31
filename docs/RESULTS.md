# Results

Status: both benchmarks run on 2026-10-31, one run per task, with Claude
Sonnet 5.5 as the agent. The headless `claude -p` login on the build machine
could not be refreshed, so the agents ran as subagents of an interactive
Claude Code session instead (`eval/README.md`, "Driving the agent from an
interactive Claude Code session"). Raw records: `eval/results/A-sub-*.jsonl` and
`eval/results/B-sub-*.jsonl`, one JSON line per run (copied from
`eval/runs/`, which is not committed and also holds every workspace and
transcript).

## Headline

| Measure | Prediction | Result | |
|---|---|---|---|
| A: pass rate | Overlang within 10 points of Python, above Rust | 100.0% Overlang, 100.0% Python, 99.0% Rust | met |
| A: compile-fix rounds | Overlang below Rust | 0 failed test rounds in every run of every language | not testable, ceiling |
| A: total tokens | Overlang 20% to 50% above Python | +9% in total; 5.8x in what the agent pulled in itself | see below |
| B: read tokens | Overlang 30% or more below Python | 2.1x above Python (12.8k vs 6.1k median) | failed |
| B: pass rate | Overlang at or above Python | 100% Overlang, 100% Python | met |

The central hypothesis of the project, that a content-addressed store with a
minimal-context command cuts what an agent has to read on a maintenance task,
failed at this scale. Two things explain it, and both are measured below: a
language the model has never seen costs about 8k tokens of context per task
before any work starts, and the agents did not use `ov ctx`.

The strongest result is the one that was not in doubt in the plan but was in
doubt in the literature review: Sonnet solved 99 of 99 HumanEval tasks in a
language it had never seen, from a 3.5k-token card, with zero failed test
rounds, in three turns each (read the card, write the function, run the
tests). First-pass correctness in a new language is not the problem.

## Benchmark A: single-function tasks

100 HumanEval tasks (the first 100 of the 155 shared ids), one run each.
Medians per run. "Fresh after turn 1" is the context the agent itself pulled
in after its first turn: `input_tokens + cache_creation_input_tokens` summed
over turns two onward, which is CLAUDE.md, files read and tool output. The
first turn is the same ~51.6k-token system prompt in every language and is
excluded from it.

| | Python | Rust | Overlang |
|---|---|---|---|
| n | 100 | 100 | 99 |
| pass rate | 100.0% | 99.0% | 100.0% |
| turns | 3 | 3 | 3 |
| failed test rounds | 0 | 0 | 0 |
| output tokens | 403 | 462 | 509 |
| fresh after turn 1 | 1,426 | 1,604 | 8,309 |
| total tokens (all turns, incl. cache reads) | 157,842 | 158,273 | 171,630 |
| wall time | 7.6 s | 7.6 s | 8.0 s |

Rust's one failure is `HumanEval_92_any_int`, whose MultiPL-E hidden test
asserts both `true` and `false` for `(3.0, 4.0, 7.0)`; it cannot pass. The
Overlang translation dropped that task for the same reason, hence n=99.

The Overlang agents' extra context is the language card: the workspace
CLAUDE.md is about 14.7k characters for Overlang against 340 for Python, and
reading it costs about 7k tokens once per task. Nothing else differs: the
agents did not consult anything beyond CLAUDE.md and the stub in any language.
Output tokens are 26% above Python, which is the verbosity of mandatory
signatures and effect labels on a ten-line function.

## Benchmark B: maintenance tasks

25 changes to the todo app (features, bug fixes, refactors; `eval/benchb/`),
one run each. Python edits `main.py`; Overlang works through the store with
`ov ls`, `ov view`, `ov ctx` and `ov edit`. Grading rebuilds the app from the
agent's final state and runs the hidden end-to-end check.

| | Python | Overlang |
|---|---|---|
| n | 25 | 25 |
| pass rate | 100% | 100% |
| turns | 5 | 4 |
| failed test rounds (runs with one) | 1 | 1 |
| output tokens | 1,304 | 1,298 |
| read calls | 3.6 | 4.6 |
| read tokens (fresh tokens after read calls) | 6,085 | 12,755 |
| total tokens | 269,847 | 241,130 |
| wall time | 15.3 s | 18.5 s |

Where Overlang's read tokens go, summed over the 25 runs and attributed to
what the previous turn read:

| What was read | Python | Overlang |
|---|---|---|
| CLAUDE.md | 36.7k | 214.4k |
| application code (`main.py` / `ov view`, `ov ls`) | 94.6k | 154.1k |
| `ov ctx` | | 2.3k |

Two findings:

- The card tax is 58% of Overlang's read tokens. The Overlang CLAUDE.md is
  the language card plus the store workflow, about 16.6k characters, read
  once per task at about 8.6k tokens. Python's is 1.2k characters. This cost
  is per task, not per project: every fresh agent pays it again.
- The agents did not use the locality tool. `ov ctx` was called once in 25
  runs. The typical Overlang agent ran `ov ls` and then `ov view` over most
  names in a shell loop, reading the whole program through the store, and
  paid 63% more for it than the Python agent paid for reading `main.py` in
  two `Read` calls. The preliminary measurement that `ov ctx` returns 4.4% of
  the program per definition is still true and still unused. On a 178-line
  app, reading everything is cheap, and a model that has never seen the
  language reads everything.

The pass rates and the near-identical output tokens say the store workflow
itself did not get in the way: editing a definition by name, having the
compiler re-check dependents, and building from the store all worked
first time in 24 of 25 runs.

## What this does and does not show

- Locality has to be forced or the program has to be big. The experiment as
  run cannot tell whether `ov ctx` saves tokens, because the agents did not
  take it. The next test is either a program ten times larger, where whole
  program reads stop being affordable, or a workspace where the store is
  the only interface and `ov view` of the whole program is not offered.
- The card tax is the dominant cost of a new language for short tasks, and
  it is in neither the plan's token prediction nor the literature. It would
  shrink with a model fine-tuned on the language, or with a card half the
  size; it does not shrink with better tooling.
- HumanEval is at the ceiling for this model in all three languages. The
  pass-rate and compile-fix-round predictions are met trivially and tell
  little; the token and read measures are where the signal is.
- One run per task, not the three the plan called for. With pass rates at
  100% the extra runs could only tighten the token medians, which at n=100
  and n=25 already vary by a few percent between tasks.

## Caveats on the method

- Every agent carried the interactive session's ~51.6k-token system prompt,
  identical across languages. It cancels in comparisons and inflates every
  absolute total; "fresh after turn 1" and "read tokens" exclude it.
- No `--max-turns` or timeout applied; no run needed either.
- The hidden tests sit in a sibling directory of the agent's workspace. A
  scan of all 349 transcripts found no tool call that mentions it.
- The Overlang tasks are hand translations of the HumanEval problems
  (`eval/tasks/humaneval/overlang/NOTES.md`), validated by running the
  reference solutions through the hidden tests; the visible test for each
  task is a single smoke check in every language.
- Workflow-driven subagents may differ from `claude -p` in system prompt and
  tool set. All three languages ran the same way, so the comparison holds;
  the absolute numbers are not comparable to a `claude -p` run.

## What it cost

349 agent runs, 64.0M tokens as measured by the harness, 84% of them cache
reads of the system prompt; about $57 at Sonnet list rates (billed to a
subscription here). Benchmark A: 48.7M tokens across 299 runs. Benchmark B:
15.3M across 50. Wall time: about 8 minutes for A and 4 for B with six agents
at a time. The plan budgeted $1,000 to $1,500 for three runs through
`claude -p`; one run through cached subagents cost a twentieth of that.

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

Compiler bugs found by the two translation passes, the app, the maintenance
tasks and the 349 evaluation runs: zero. Bug found by the fuzzer in the
spec's own example: one (an Int overflow the contract did not exclude).

## What the agents said about writing Overlang

Collected from the todo app README, the translation notes and the
maintenance-task report, before the measured run:

- Missing conveniences dominated the friction: `count`, digit and base
  helpers, character classes, `break`/`continue`, whitespace split, string
  interpolation. The first two revisions added most of these.
- The store workflow cost an extra round after a type change until
  dependents were re-checked on edit (fixed in the third revision).
- No agent hit a compiler bug. The C backend, the typechecker and the store
  held up under about 250 probes, 180 validated programs and 124 evaluation
  runs that compiled Overlang.
