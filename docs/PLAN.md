# Overlang: an AI-native programming language, built for about $10k in tokens

Name: **Overlang** (everything overt: types, effects, contracts). CLI `ov`, extension `.ov`.

## 1. What we are testing

Hypothesis: a language designed for agents, not humans, lets a coding agent
finish the same tasks with fewer tokens and fewer failed rounds than Python or
Rust, *even though the agent has never seen the language in training*.

The literature says the levers are, in order of value:

1. **Locality.** Agents spend ~75% of tokens reading. A content-addressed store
   that hands the agent exactly the definitions it needs attacks the biggest
   cost.
2. **First-pass correctness.** Mandatory signatures, effects, no null, no
   exceptions, exhaustive matching, contracts. Failed attempts cost ~4x
   successful ones, so every error moved to compile time is leveraged.
3. **Verification.** Contracts the toolchain checks, so review moves from code
   to spec. Full SMT verification is out of budget; we do runtime contracts plus
   automatic property-based testing, with an SMT path as a stretch goal.
4. **Terse encoding.** Weakest lever. We do not pursue it beyond canonical form.

The experiment is a success if the evaluation in section 6 shows (1) and (2)
clearly, whatever the overall token count says. A negative result is also a
result: it would mean the familiarity tax dominates for a one-shot language,
which is worth knowing.

## 1b. Prior art to read before phase 0

**Aver** (averlang.dev, github.com/jasisz/aver) is a statically typed language
"for AI to write and humans to review": effects in function signatures,
colocated test blocks, a bytecode VM plus Rust and WASM backends, and Lean and
Dafny export. It overlaps with our signature and effect design, so read it
first and borrow what works. What it does not appear to have, and what this
experiment is actually about, is the content-addressed store, the minimal
context command, and a measured agent-cost comparison against Python and Rust.
If Aver's syntax is good, adopting pieces of it is cheaper than inventing.

**MoonBit** for the mandatory-signature and declaration-first rationale.
**Unison** for the store design. **Lean 4** and **Koka** for how a modern
language emits C.

## 2. Decisions, made up front

| Decision | Choice | Rejected alternative and why |
|---|---|---|
| Implementation language | Rust, single crate, binary `ov` | Python (fastest to build, but a Python compiler for a native language makes no sense) and OCaml (the classic, but agents are measurably weakest at it). Rust gives one static binary, strong types for the AST, and agents are good at it. |
| Compile target | **C11**, built with clang or gcc into a native binary | Python source (cheapest, but not a real language) and LLVM IR (the production route, but harder to emit and debug from Python). C is the classic path for a new language: Lean 4, Koka, Nim, Vala, Chicken, V and Zig's C backend all do it. LLVM is the upgrade if we ever need it. |
| Memory | Boehm GC (`libgc`), one dependency | Perceus-style reference counting as in Lean 4 and Koka. Correct and faster, but a month of work. `shortcut: Boehm GC, upgrade to refcounting if the language gets real.` |
| Value representation | Int is int64 with overflow trapping. Float is double. Str is immutable UTF-8 with length. ADTs are tag plus union. List is an immutable array. Closures are lambda-lifted with an environment struct. Generics are monomorphized. | Uniform boxed values. Monomorphization is simpler to emit and we do not care about code size. |
| Type system | Static, nominal ADTs, simple generics, bidirectional local inference, **mandatory top-level signatures** | Full Hindley-Milner. Global inference is a rabbit hole and hides information agents want to see. |
| Effects | Labeled set on the arrow (`!io`, `!mut`, `!net`, `!fs`). Pure by default. Callers must declare a superset. | Full algebraic effects. Too much for v1; labels give 80% of the locality benefit. |
| Errors | `Result[T, E]` and `Option[T]` only. No null, no exceptions, no panics except contract failure. | Exceptions. Invisible control flow is exactly what agents get wrong. |
| Contracts | `requires` / `ensures` clauses compiled to runtime checks that fail with a structured JSON error. Fuzzed on `ov test`: `ov` derives a value generator from each Overlang type, serializes generated values, and the compiled binary's `--fuzz <fn>` entry point decodes and calls them. Shrinking is greedy (halve ints, drop list elements), implemented in `ov`. | SMT verification. Stretch goal for a subset (integer arithmetic, no loops). |
| Source of truth | SQLite store of ASTs keyed by SHA-256 of the name-stripped tree. Names in a separate table. Append-only. | Text files in git. Files are what we are testing against. |
| Formatting | One pretty-printer. The store is the only source of text, so there is no formatting choice. | Formatter plus linter. Unnecessary when text is a projection. |
| Diagnostics | JSON only: code, span, expected, actual, ranked candidate fixes | Prose. The reader is a model. |
| Syntax prior | Close to Rust and ML, so the agent's existing prior transfers | Novel syntax. Would pay the familiarity tax twice. |
| Constrained decoding | Out of scope. We cannot control the frontier model's decoder. | |
| Self-hosting | Out of scope. | |

## 3. The language, by example

```
// Pure by default. Signatures mandatory on every top-level definition.
type Line  = { sku: Str, qty: Int, unitCents: Int }
type Order = { id: Int, lines: List[Line], paid: Bool }

type ChargeError = Declined(reason: Str) | Timeout | AlreadyPaid

fn total(o: Order) -> Int
  requires all(o.lines, |l| l.qty > 0 && l.unitCents >= 0)
  ensures  result >= 0
{
  sum(map(o.lines, |l| l.qty * l.unitCents))
}

// Effects are part of the signature. A pure function cannot call this.
fn charge(o: Order, gw: Gateway) -> Result[Receipt, ChargeError] !net
  requires !o.paid
{
  gw.charge(o.id, total(o))
}

// Spec-first: declare the interface, implement later. The store holds it as
// a stub; `ov check` lists every unimplemented declaration.
declare fn refund(r: Receipt, gw: Gateway) -> Result[Unit, ChargeError] !net

fn describe(e: ChargeError) -> Str {
  match e {
    Declined(reason) => "declined: " ++ reason
    Timeout => "timed out",
    // Missing AlreadyPaid: compile error, with the missing arm as a suggested fix.
  }
}
```

Grammar size target: under 60 productions. If it does not fit on two pages,
cut it.

Standard library for v1: Int, Float, Str, Bool, Unit, List, Map, Set, Option,
Result, and the usual higher-order functions, implemented in a small C runtime
(`rt.h`, `rt.c`) plus Overlang itself where possible. Plus one FFI form:

```
foreign c "sqlite3.h" fn sqlite3_libversion() -> Str
foreign c "math.h"    fn cbrt(x: Float) -> Float
```

The agent declares the signature and effects; the compiler trusts it and emits
the include and link flags. This is how the whole C ecosystem is one line away.
Marshalling in v1 covers scalars, Str and opaque pointers only.

## 4. The toolchain

One binary, `ov`. Every command emits JSON when `--json` is set, and the agent
always sets it.

| Command | What it does |
|---|---|
| `ov add <file>` | Parse, typecheck, hash each definition, append to the store. Prints name → hash. |
| `ov view <name>` | Pretty-print the current definition. The only way text exists. |
| `ov edit <name> < text` | Parse, typecheck, store as a new hash, repoint the name. |
| `ov deps <name>` / `ov dependents <name>` | Transitive dependency graph by hash. |
| `ov ctx <name>` | **The key feature.** Emit the minimal context to work on a definition: its body, the full signatures (with effects and contracts) of everything it calls, and nothing else. |
| `ov check [name]` | Typecheck, effect check, exhaustiveness, unimplemented declarations. JSON diagnostics with ranked fixes. |
| `ov test [name]` | Run test blocks plus fuzzing of every `requires` / `ensures` reachable from `name`. Cached by hash: unchanged definitions are not retested. |
| `ov build <name>` | Monomorphize, emit C, invoke the C compiler, link the runtime and GC. Emitted C is cached by hash, so an unchanged definition is never re-emitted. |
| `ov run <name>` | `ov build` then execute. |
| `ov diff <hash1> <hash2>` | Structural diff of two definitions. |

Diagnostic shape:

```json
{"code": "E0104", "severity": "error", "name": "describe", "hash": "a3f9…",
 "span": {"line": 3, "col": 3}, "message": "non-exhaustive match on ChargeError",
 "expected": ["AlreadyPaid"], "actual": ["Declined", "Timeout"],
 "fixes": [{"rank": 1, "description": "add arm", "patch": "AlreadyPaid => todo(),"}]}
```

## 5. Phases and budget

Budget is API tokens for the agent sessions that build this. Dollar figures
assume frontier-model pricing and a roughly $25 to $40 per agent-hour burn.

| Phase | Deliverable | Exit criterion | Budget |
|---|---|---|---|
| 0. Spec | Grammar, type rules, effect rules, store schema, diagnostic codes. One document. | A second agent, given only the spec, writes ten valid programs and predicts the type errors in ten invalid ones. | $300 |
| 1. Core compiler | Lexer, parser, typechecker, exhaustiveness, effect checker, monomorphizer, C emitter, `ov build` / `ov run`. | 200 unit tests pass. The example in section 3 compiles and runs as a native binary. | $2,500 |
| 1b. C runtime | `rt.c`: Boehm GC integration, Str, List, Map, Set, Option, Result, overflow-trapping Int, closures, structured contract-failure output. | Runtime has its own C test suite. Valgrind or ASan clean on the test suite. | $1,000 |
| 2. Store and context | SQLite store, hashing, pretty-printer, `add` / `view` / `edit` / `deps` / `ctx` / `diff`, JSON diagnostics with fixes, hash-cached C emission. | Round-trip property: parse → store → view → parse yields the same hash, fuzzed over 1,000 generated programs. | $1,500 |
| 3. Contracts and FFI | `requires` / `ensures` as runtime checks, fuzz bridge through `--fuzz`, hash-cached `ov test`, `foreign c`, stdlib. | Contract fuzzing finds a seeded bug in each of 20 planted-bug programs. | $1,800 |
| 3b. Web app milestone | A runnable example, `examples/todo/`: an HTTP todo app backed by SQLite. Two small C shims under `runtime/ext/` (`ov_sqlite.c`: open, exec, query returning rows as delimited text; `ov_http.c`: listen, accept one request as text, respond) bound with `foreign c`, plus the Overlang app on top: routing, HTML rendering, CRUD. Opaque handles are Ints. This is the "does it feel like a real language" check, and the app doubles as the Benchmark B codebase. | `ov run main` serves on a port; `curl` can create, list and delete todos; data survives a restart. | $800 |
| 4. Corpus | MultiPL-T loop: translate ~1,500 Python functions with tests (MBPP+, HumanEval+) into Overlang, keep only those whose tests pass. Produce the **language card**: a 3k-token reference that goes into the agent's CLAUDE.md. | ≥70% of translations pass on first or second attempt. Every compiler bug found gets a regression test. | $900 |
| 5. Evaluation | See section 6. | Results table and a short write-up. | $2,500 |
| Reserve | Compiler bugs found in phase 5 always exist. | | $1,300 |
| **Total** | | | **$12,300** |

The C backend adds about $1,500 over the compile-to-Python version: the
runtime, the monomorphizer, and the fuzz generator and shrinker. To hold $10,000, drop Map and Set from the runtime and cut the
corpus to 1,000 programs.

Outside the token budget: about $200 of GPU time if we also fine-tune
Qwen2.5-Coder-7B on the phase 4 corpus as a side experiment. Optional.

Calendar: five to seven weeks of agent sessions, one phase per week, phase 5
taking two.

## 6. Evaluation design

Two benchmarks, because the two hypotheses need different tasks.

**Benchmark A: single-function tasks.** 100 problems from LiveCodeBench or
MBPP+. Same agent harness (Claude Code with a fixed CLAUDE.md, or mini-SWE-agent
for reproducibility) solves each in Python, Rust, and Overlang. Overlang gets the
language card in context; the others get nothing, which is the honest
comparison. This tests first-pass correctness and the familiarity tax. Expect
Overlang to lose on total tokens here. The number to watch is compile-fix rounds
versus Rust.

**Benchmark B: maintenance tasks.** Use the todo web app from phase 3b (plus
a Python port of it written in phase 4, so both languages have the same
codebase, ~1,000 lines, 30 to 50 functions), and write 25 modification tasks
against it ("add a field and propagate it", "change this
error type", "make this function pure"). Same harness, both languages. This is
the only place the locality hypothesis can show, because single-function tasks
do not involve reading.

Metrics per trajectory: pass/fail, total tokens, input versus output split,
cache hit rate, turns, number of failed compile or test rounds, wall time.
Three runs per task per language, since token use varies widely between runs.

Predictions, written down before running:

| Measure | Prediction |
|---|---|
| Benchmark A pass rate | Overlang within 10 points of Python, above Rust. |
| Benchmark A compile-fix rounds | Overlang below Rust, because diagnostics carry fixes. |
| Benchmark A total tokens | Overlang 20% to 50% above Python. The familiarity tax. |
| Benchmark B read tokens | Overlang 30% or more below Python, via `ov ctx`. |
| Benchmark B pass rate | Overlang at or above Python, because effects and exhaustiveness catch propagation mistakes. |

If B's read-token prediction fails, the central hypothesis of the project is
wrong and we say so.

## 7. Risks

- **Familiarity tax swamps everything.** Most likely failure. Mitigation: syntax
  near the Rust and ML prior, a tight language card, and reporting Benchmark B
  separately from A.
- **Type-system scope creep.** Generics plus inference eats budget without end.
  Mitigation: mandatory top-level signatures make local inference enough.
  No higher-kinded types, no traits in v1.
- **The store fights the agent harness.** Claude Code wants to edit files.
  Mitigation: `ov view` writes to a scratch file and `ov edit` reads it back;
  a thin wrapper makes this look like a file edit to the harness.
- **The generator cannot produce values for every type.** Mitigation: derive
  generators from ADTs automatically, and allow an explicit `gen` clause as
  an escape hatch.
- **The C runtime is where the budget leaks.** Strings, hash maps and closures
  in C are each a known rabbit hole. Mitigation: immutable arrays instead of
  persistent structures, Boehm instead of our own GC, and ASan on from day one
  so bugs surface where they happen.
- **C compile time slows the agent loop.** Mitigation: one C file per
  definition, cached by hash, so an edit recompiles one translation unit.
- **Evaluation noise.** Token use varies up to 30x between runs of the same
  task. Mitigation: three runs each, report medians, and report the
  distribution.

## 8. Out of scope for v1

Constrained decoding, self-hosting, SMT verification (stretch only),
concurrency, a module system beyond a flat namespace, a package manager, a
terse transport encoding, an editor plugin.

## 9. Working method

One git repo. The compiler is a Rust crate with `cargo test`, built by Claude
agents (Opus and Sonnet executing, a Fable overseer reviewing), one phase per
milestone. Every compiler bug found by the corpus or
the evaluation becomes a regression test before it is fixed. The plan document
is updated at each phase boundary with what was actually spent.

## 10. Spend ledger

Subagent token totals as reported by the harness per agent (input plus
output, all models), updated at phase boundaries. The overseer's own tokens
are not included.

| Date | Agent | Model | Tokens |
|---|---|---|---|
| 2026-10-29 | C runtime | Sonnet | 106k |
| 2026-10-29 | Front end (lexer, parser, printer) | Opus | 163k |
| 2026-10-29 | SQLite and HTTP shims | Sonnet | 94k |
| 2026-10-29 | Evaluation harness | Sonnet | 133k |
| 2026-10-29 | Store and hashing | Opus | 174k |
| 2026-10-29 | Typechecker, effects, exhaustiveness | Opus | 272k |
| 2026-10-29 | Store follow-up (propagation, rendering) | Opus | 155k |
| 2026-10-29 | Language card | Sonnet | 95k |
| 2026-10-29 | Monomorphization and C emitter | Opus | 254k |
| | **Total so far** | | **1.45M** |
