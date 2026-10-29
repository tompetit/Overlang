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

Phase 0. Nothing compiles yet.

## Build

```
cargo build --release
```

Requires a C compiler and the Boehm GC development headers (`gc-devel` on
Fedora and Amazon Linux, `libgc-dev` on Debian and Ubuntu).

## License

MIT.
