# Overlang HumanEval tasks: translation notes

All 156 shared ids of `eval/tasks/index.json`. **155 validated, 1 skipped** (HumanEval_92).
Batch 1 = ids 0..39 (40 validated); batch 2 = ids 40..155 (115 validated, 1 skipped).
Regenerate and re-validate with `python3 -I eval/tasks/humaneval/overlang/build.py [--tasks a:b]`
(needs `target/release/ov`); it only writes a task file when `ov` confirms that
the stub checks, the stub fails the hidden tests, and the reference solution
passes both the visible and the hidden tests. The mock run
(`run.py --benchmark humaneval --lang overlang --agent mock --mock-solve`) passes 155/155;
`--tasks 0:156` itself fails because the harness loads `index.json` ids without skipping
a missing task file, so run it as `0:88` and `89:156` (88 is the skipped id).

## Task file format

`id, benchmark, lang, name, entry_point, prompt, tests, visible_test, solution`
(+ `renamed_from` when the function was renamed, `renamed_params` = {python name: Overlang name}).

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

## Batch 2 (ids 40..155): what changed in the generator

- Doc comments: a doctest result is the shortest run of lines that parses, the rest is prose
  (`Explanation:` lines); `Examples:` is inserted before the first rewritten example, not
  before the first line that merely looks indented; a doctest that cannot be typed is
  dropped (HumanEval_95: one example with an int key in a `Dict[str, str]`).
- Literals: `Dict[K, V]` becomes `map_from_list([(k, v), ...])` / `map_new()`;
  `-Some(3)` bug fixed (negative inner value of an `Option` is `Some(-3)`); non-assert
  statements in `check` (`print`) are ignored.
- `sol(..., pren={...})` renames parameters.

## Renames forced by the language (batch 2)

| task | python | Overlang | why |
|---|---|---|---|
| 111_histogram | param `test` | `s` | `test` is a keyword (E0002) |
| 84_solve | param `N` | `n` | capitalised identifiers are type names (E0002) |
| 91_is_bored | param `S` | `s` | same |
| 153_Strongest_Extension | fn `Strongest_Extension` | `strongest_extension` | same, `renamed_from` set |

Names like `digitSum`, `minPath`, `specialFilter`, `minSubArraySum` (lower camel case) are fine.
No entry point of batch 2 clashed with a prelude name.

## Skipped

- **HumanEval_92_any_int**: `x, y, z: float`, but the tests include `any_int(3, 4, 7) == True`
  and `any_int(3.0, 4, 7) == False`: the contract is about the runtime type (int vs float) of a
  number, not its value. With `Float` parameters those two calls are the same call. Untypable.

## Faithful oddities (kept as in the Python task)

- 47_median: the doc example says `15.0`, the test says `8.0` for the same input (the test is right).
- 116_sort_array: the doc examples describe plain ascending order, the tests sort by popcount of
  the absolute value. Solution follows the tests.
- 122_add_elements: "at most two digits" is `len(str(x)) <= 2` in the reference, so a negative
  two-digit number is excluded; the solution does the same.
- 129_minPath: reference solution alternates `1, min_neighbour_of_1`; so does mine.
- 162_string_to_md5: a real MD5 in Overlang (no bit operators, hex literals, or `sin`; 32-bit
  words are Ints, bit operations loop over 32 bits, the 64 constants are decimal literals
  generated by `build.py`). Validated against the four test vectors.

## Compiler bugs (batch 2)

None found. About 150 more probes (arithmetic signs and rounding, `div_floor`/`mod_floor`, unicode
strings, `parse_int`/`parse_float` edge cases, `split`/`replace`/`slice`/`take` out of range,
`Map`/`Set` equality and key order, recursion 200000 deep, closures, tuple/nested patterns,
`return` inside lambdas, shadowing, generics, mutual recursion) behaved per spec; `ov fmt` is idempotent
and formatted output still passes on all 155 solutions. No fixtures added to `tests/programs/`.

## Language and tooling gaps, ranked by tasks affected in batch 2

Counts are tasks whose reference solution had to work around the gap (approximate, from the
solutions; a task can appear under several gaps).

1. **No `count`/`count_by`** (16): `len(filter(xs, p))` everywhere (47..163: 64, 73, 98, 113, 117, 136, ...).
2. **No digit helpers** (11): `chars(show(n))` + `unwrap_or(parse_int(c), 0)` to get the digits of an
   Int (84, 94, 104, 107, 108, 131, 145, 155, 67, 146, ...); no `to_base`/binary formatting, so
   44, 79, 84 hand-roll `show(m % 2) ++ bits` loops.
3. **No character classification** (9): `is_upper`, `is_lower`, `is_alpha`, `is_digit`; written as
   `to_upper(c) == c && to_lower(c) != c`, `c >= "0" && c <= "9"`, or via `code_point` ranges
   (66, 93, 95, 134, 141, 153, 161, 98, 89).
4. **No number-theory helpers** (8): `is_prime`, `gcd`, factorisation hand-written (59, 75, 82, 94,
   96, 127, 143, 150); every solution defines its own `is_prime`.
5. **No `break`/`continue`** (22 use `return` inside loops instead). Works, but forces extraction of a
   helper when the loop is followed by more code.
6. **`split` keeps empty pieces and there is no whitespace split/`words`** (4): `filter(split(s, " "),
   |w| str_len(w) > 0)` (101, 111, 117, 143-style); Python's `s.split()` is a one-liner.
7. **`sort`/`sort_by` limits** (3): `sort_by` takes only an `Int` key and `sort` only Int/Float/Str,
   so compound keys (116, 145, 149) are done by `sort` followed by a stable `sort_by`.
   Stability is true of the implementation (merge sort in `emit.rs`) but is **not in the spec**
   or the card; three solutions depend on it.
8. **No `min_by`/`max_by`/`argmax`** (4): 12, 68, 153, 158 hand-roll a loop with a "best so far".
9. **Diagnostic for a capitalised function/parameter name is unhelpful** (3 + `test` param 1):
   E0002 "unexpected type name `S`, expected identifier". Python names such as `N`, `S`,
   `Strongest_Extension` hit it; a hint "values are snake_case" would save a round.
10. **No bitwise operators, hex literals or trig** (1 task, 162, but it is ~100 lines): also costs
    `popcount` (116) a recursion, and rules out efficient binary conversion.
11. **No map literal** (2): tests and doc examples use `map_from_list([("a", 1), ...])` (95, 111).
12. **Harness**: `run.py` cannot skip a task whose file is missing (see top); and
    `--out ./ev[a]l/...` (a glob-guard workaround) creates a literal `ev[a]l` directory when `eval/runs`
    does not exist yet.
13. **Python's int/float distinction cannot be expressed** (1, the skip): a possible `Num` union is
    not worth adding; keep it as an expected "untypable" class for the benchmark.

(The gaps listed for batch 1 above still stand: card vs. CLI mismatch, build products in cwd,
no summary line from `ov test`, prelude clashes, `while true` tail.)
