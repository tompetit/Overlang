# Overlang HumanEval tasks: translation notes

Batch 1: the first 40 ids of `eval/tasks/index.json`. **40 validated, 0 skipped.**
Regenerate and re-validate with `python3 -I eval/tasks/humaneval/overlang/build.py`
(needs `target/release/ov`); it only writes a task file when `ov` confirms that
the stub checks, the stub fails the hidden tests, and the reference solution
passes both the visible and the hidden tests.

## Task file format

`id, benchmark, lang, name, entry_point, prompt, tests, visible_test, solution`
(+ `renamed_from` when the function was renamed).

- `prompt`: `//` comment (docstring prose, then `Examples:` with the doctests
  rewritten in Overlang syntax, e.g. `f([1, 2]) == Some(3)`), then the signature
  with body `todo()`. It is written verbatim to `solution.ov`.
- `tests` / `visible_test`: one `test check_N` block per Python assertion (the
  visible file has only `check_1`). Float results use `assert(tst_close...(got,
  want))` with a 1e-6 tolerance; the helpers (`tst_close`, `tst_close_list`,
  `tst_close_pair`) are defined at the top of the test text, so they are
  prefixed to stay clear of anything a solution defines (flat namespace).
- `solution`: my reference, whole-file (comment, function, helpers).

## Deviations from the Python task

- `HumanEval_34_unique` is renamed `unique_sorted`: `unique` is a prelude
  function and prelude names cannot be redefined (E0109). Of all 156 shared
  HumanEval ids this is the only entry-point clash. The other direction: one
  parameter name is an Overlang keyword, `test` in `HumanEval_111_histogram`
  (`let test = ...` and `fn f(type: Int)` are parse errors E0002); it will need
  a rename when that task is translated.
- Python `int`/`float`/`str`/`bool`/`List`/`Optional`/`Tuple` all map directly,
  so nothing in this batch was untypable. Python `int` is unbounded and Overlang
  `Int` traps on i64 overflow; no test in this batch gets near it (largest
  value: `prime_fib(10) = 433494437`).
- Tests are run as `ov test solution.ov tests.ov` (two files, one program)
  rather than by concatenating text: diagnostics then carry the right file name
  and line numbers. Equivalent for grading. `ov test` stops at the first failing
  block (exit 3), so a failed grade shows only the first failure.

## Compiler bugs

None found in this batch. About 100 extra probes (prelude semantics, recursion
100000 deep, closures, list patterns, `?`, records, tuples in `for`, shadowing,
`i64` boundaries) all behaved per spec; see language gaps for the mismatches
that remain. No fixtures were added to `tests/programs/`.

## Language and tooling gaps

1. **The card describes a store-based CLI that the binary does not have.**
   CARD.md says "always pass `--json`" and lists `ov check [name]`,
   `ov run <name>`, `ov add/view/edit/ctx/deps`. The built `ov` has `check`,
   `build`, `run`, `test`, `fmt`, `parse` over *files*; `--json` is rejected
   (E0000) and `ctx` is an unrecognized subcommand. The adapter's CLAUDE.md
   states the file-based reality after the card, but an agent that follows the
   card first will waste turns. Either implement the store commands or trim the
   card.
2. **Build products land in the cwd**: `ov test` / `ov run` leave `.ovbuild/`,
   `a` and `a.c` in the current directory. The adapter runs `ov` from `.ovtest/`
   (workspace) and `hidden/` (grading) to keep the agent's directory clean.
   Worth an `-o`/build-dir option.
3. **`ov test` prints only `ok <name>` and stops at the first failure.** There
   is no summary line and no pass/fail count; graders must parse `ok` lines. A
   `--keep-going` mode and a summary line would make failed-round diagnostics
   richer.
4. **Prelude name clashes** (see above): a user function named like any of about 100
   prelude names is a hard error. Reasonable, but for benchmark ports it forces
   renames; the diagnostic is clear (E0109 "is a prelude function").
5. **Awkward idioms met while writing the reference solutions**
   - No `count`, `max_by`, `min_by`, `take_while`,
     `last_index`, string `reverse`, `is_digit`/`is_alpha`/`is_upper`: character
     classification goes through `to_upper(c) == c` or string comparison
     (`c >= "0" && c <= "9"`), counting through `len(filter(...))`.
   - `unique` / `sort` are limited to primitive element types, and `sort_by`
     only takes an `Int` key, so sorting by a float or a string key needs a
     hand-written sort.
   - `assert_eq(f([]), [])` is fine, but `assert_eq(max_of([]), None)` and
     `assert_eq(tail([]), [])` give E0107 (both sides are unconstrained); the
     fix suggested is correct but the error is surprising to read.
   - A fn whose last statement is `while true { ... return x; }` still needs a
     dummy tail expression (`0`); the checker does not know the loop never exits.
   - `for (i, x) in enumerate(xs)` works (tuple pattern), which is good; the card
     only shows `for x in list`.
