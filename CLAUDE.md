# Overlang: conventions for agents working in this repo

Read docs/SPEC.md before touching the compiler. It is the contract. If the
code and the spec disagree, the spec wins; if the spec is wrong, change the
spec in the same commit and say why in the commit message.

## Layout

- `src/` Rust compiler, one crate, binary `ov`. Modules: `lexer`, `parser`,
  `ast`, `printer`, `types`, `effects`, `exhaust`, `mono`, `emit`, `store`,
  `diag`, `fuzz`, `cli`.
- `runtime/` C runtime (`rt.h`, `rt.c`) linked into every program, with its
  own tests in `runtime/tests/`.
- `tests/programs/` `.ov` fixtures. Each has an `.expect` file: either the
  program's stdout, or the JSON diagnostic it must produce.
- `docs/` plan and spec.
- `eval/` evaluation harness and benchmark tasks.

## Rules

- `cargo test` must pass before every commit. Runtime C tests run under
  AddressSanitizer.
- Every bug found by a fixture, the corpus or the evaluation gets a
  regression fixture in `tests/programs/` before it is fixed.
- Diagnostics are JSON. Never print prose errors from the compiler.
- Keep the grammar small. A feature not in the spec is not added without
  changing the spec first.
- No new crate dependencies without a one-line justification in the commit.
- Commit messages: one line what, one paragraph why.
