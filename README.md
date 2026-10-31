# Overlang

An AI-native programming language. Everything is overt: types, effects and
contracts are mandatory in every signature, code lives in a content-addressed
store rather than files, and the compiler talks to its user (a coding agent)
in JSON.

Compiles to C. The compiler is written in Rust and ships as one binary, `ov`.

This is an experiment, built almost entirely by coding agents on a budget of
roughly $10k in tokens, to test one hypothesis: a language designed for agents
lets them finish tasks with fewer failed rounds and fewer read tokens than
Python or Rust, even with no training data for the language.

- [docs/PLAN.md](docs/PLAN.md): the hypothesis, the design decisions, the
  budget, and the evaluation design with predictions written down in advance.
- [docs/SPEC.md](docs/SPEC.md): the language specification.

## Status

Working, two days in. Programs compile to native binaries through C; the
content-addressed store, typechecker, effect checker, exhaustiveness
checker, contracts and the JSON diagnostics all work; the first real
program, an HTTP todo app over SQLite in 178 lines, is in
[examples/todo](examples/todo), with a Python port for the maintenance
benchmark. `ov test` fuzzes every contracted function and found an overflow
in the spec's own example. The evaluation harness holds 155 HumanEval tasks
in Python, Rust and Overlang and 25 maintenance tasks, and both have been
run once with Sonnet as the agent: 100% pass in every language on both
benchmarks, but the locality prediction failed, because the agents read the
whole program instead of using `ov ctx` and paid an 8k-token language card
per task on top. [docs/RESULTS.md](docs/RESULTS.md) has the numbers and
what they do and do not show; the ledger at the end of the plan has the
spend.

```
ov add app.ov        # typecheck, hash and store every definition
ov ctx route         # the minimal context to work on one definition
ov edit route        # replace it; callers are re-hashed automatically
ov test              # test blocks, then contract fuzzing
ov run main          # build a native binary from the store and run it
```

## Build

```
cargo build --release
```

Requires a C compiler and the Boehm GC development headers (`gc-devel` on
Fedora and Amazon Linux, `libgc-dev` on Debian and Ubuntu).

## License

MIT.
