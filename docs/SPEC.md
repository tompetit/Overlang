# Overlang language specification, v0.1

Overlang is a statically typed, mostly functional language whose every
top-level definition carries a full signature: types, effects and contracts.
Code lives in a content-addressed store; text is a projection. The compiler,
`ov`, emits C and speaks JSON.

This document is the contract between the spec and the implementation. Where
the implementation disagrees, the implementation is wrong, or the spec is
changed in the same commit.

The grammar is deliberately small. Features marked **shortcut** are known
simplifications with a named upgrade path.

## 1. Lexical structure

- Source is UTF-8. Whitespace is insignificant except as a separator.
- Comments: `//` to end of line. No block comments.
- Identifiers: value and function names match `[a-z_][A-Za-z0-9_]*`. Type
  and constructor names match `[A-Z][A-Za-z0-9_]*`. Effect names match
  `[a-z][a-z0-9_]*` and appear only after `!`.
- Keywords: `fn type declare foreign link intrinsic let mut if else match while
  for in return requires ensures test true false result`.
  `result` is a keyword only inside an `ensures` clause.
- Literals:
  - Int: decimal digits with optional `_` separators (`1_000_000`). Must fit
    in a signed 64-bit integer, otherwise lex error E0001.
  - Float: digits, `.`, digits, optional exponent (`1.5`, `2.0e10`). A float
    literal always contains `.`.
  - Str: `"..."` with escapes `\n \t \r \\ \" \0` and `\u{XXXX}`. No raw
    strings, no interpolation. Use `++` and `show`.
  - Bool: `true`, `false`. Unit: `()`.
- Punctuation: `( ) { } [ ] , ; : . .. -> => | ! ? = == != < <= > >= + - * / %
  && || ++ _`.

## 2. Program structure

A program is a sequence of top-level definitions in a single flat namespace.
There are no modules in v0.1 (**shortcut**: add a module system when a
project exceeds a few hundred definitions). Definitions may reference each
other in any order. Duplicate names are error E0109. Prelude names (section 9)
are in scope everywhere and cannot be redefined.

Top-level definitions:

```
type Name[T, U] = { field: Type, field: Type }         // record
type Name[T] = Ctor(field: Type, field: Type) | Ctor   // sum
fn name[T, U](param: Type, param: Type) -> Type !eff !eff
  requires Expr
  ensures Expr
{ Block }
declare fn name(param: Type) -> Type !eff requires Expr ensures Expr
foreign c "header.h" link "libname" fn name(param: Type) -> Type !eff
test name { Block }
```

- Type parameters are listed in `[...]` after the name. Effect parameters
  are written `!e` inside the same brackets: `fn map[T, U, !e](...)`.
- `declare fn` introduces a signature with no body. Calls to it typecheck.
  `ov check` reports it as warning W0001. If it is executed, the program traps
  with kind `unimplemented`.
- `foreign c` binds a C function. The header is included verbatim; `link`
  adds `-l<libname>`. Parameters and return must be Int, Float, Bool, Str or
  Unit (**shortcut**: no struct marshalling). Str crosses as `const char*`
  (NUL-terminated, UTF-8); a returned `const char*` is copied. Effects are
  whatever the declaration says; the compiler trusts it.
- `test name { ... }` is a test block. Its body has type Unit and may use any
  effects. `ov test` runs every test block.
- `intrinsic fn name[T](params) -> Type !eff = "symbol"` binds a signature to
  a runtime C function (`"ov_..."`) or to a compiler-generated instance
  (`"@show"`, `"@sort"`). It is legal only in `src/prelude.ov`, which is
  compiled into `ov` and loaded before every program. Elsewhere it is E0002.
- Comments are kept by the parser and printer (`ov fmt` never drops one); a
  comment inside an expression or signature moves to the next statement, arm
  or definition. Hashing strips them.

## 3. Types

Built-in types: `Int` (signed 64-bit, overflow traps), `Float` (IEEE double),
`Bool`, `Str` (immutable UTF-8), `Unit`, `List[T]` (immutable sequence),
`Map[K, V]` and `Set[T]` (immutable; K and T must be Int, Str or Bool,
**shortcut**), tuples `(A, B, ...)` of arity 2 to 8, and function types
`fn(A, B) -> R !eff`.

`Option[T]` and `Result[T, E]` are ordinary prelude sum types:

```
type Option[T] = Some(value: T) | None
type Result[T, E] = Ok(value: T) | Err(error: E)
```

Records: `type P = { x: Int, y: Int }`. A record type has at least one
field. Construct with `P { x: 1, y: 2 }`.
Update with `P { ..p, y: 3 }` (the base comes first). Access with `p.x`.
Field order in construction is free; the printer canonicalizes to declaration
order.

Sum types: `type Shape = Circle(r: Float) | Rect(w: Float, h: Float) | Empty`.
Constructor fields are named in the declaration for documentation and
diagnostics but positional in use: `Circle(1.0)`, `match s { Circle(r) => ... }`.
A constructor with no fields is written without parentheses.

Generics: type parameters on types and functions. No bounds, no traits.
Equality `==` and `!=` are structural and defined on every type except
function types (E0101 if attempted). Ordering `< <= > >=` is defined on Int,
Float and Str only. `show[T](x: T) -> Str` is defined structurally on every
non-function type by monomorphization. `show` prints Int as decimal, Float as
the shortest decimal that round-trips with no trailing `.0` (`3.0` shows as
`3`, `2.5` as `2.5`), Bool as `true`/`false`, Str quoted and escaped, Unit as
`()`, lists as `[a, b]`, tuples as `(a, b)`, records as `Name { f: v }`, and
constructors as `Ctor(a, b)` or `Ctor`. `print` and `println` take a Str and
print it raw, so `println("x")` prints `x` without quotes.

There is no subtyping and no implicit conversion. `to_float` and `to_int`
convert explicitly.

## 4. Type inference

Every top-level signature is complete; nothing is inferred at the top level.
Inside a body, inference is bidirectional and local: let bindings and lambda
parameters take their type from the expression or the expected type. Generic
instantiation at call sites is solved by unification. If a type cannot be
determined, error E0107 names the expression and asks for an annotation.

Lambda parameters may be annotated: `|x: Int, y| x + y`.

## 5. Effects

A function's signature lists the effects it may perform, as `!name` labels
after the return type. No labels means pure. The effect checker enforces, for
every call inside a body, that the callee's effects are a subset of the
caller's declared effects. Violation is error E0105, whose fix suggestion is
the exact label to add.

Effect names are open: any identifier may be used, so a program can declare
`!db` or `!gpu` for its own foreign functions. The prelude uses `io`
(console), `fs`, `net`, `rand`, `time`, `exit`.

Function types carry effects: `fn(Int) -> Int !io`. A lambda's effects are
inferred from its body.

Effect polymorphism: a signature may declare one effect parameter, `!e`, and
use it on function-typed parameters and its own effect list:

```
fn map[T, U, !e](xs: List[T], f: fn(T) -> U !e) -> List[U] !e
```

At a call site, `!e` is solved as the union of the effects of the arguments
bound to it. The caller is charged the callee's declared effects with `!e`
substituted. One effect parameter per signature (**shortcut**).

Local mutation (`let mut`, assignment, `while`, `for`) is not an effect; it
is invisible outside the function.

## 6. Expressions and statements

Precedence, lowest to highest: `||`, `&&`, comparisons, `++`, `+ -`, `* / %`,
unary `- !`, postfix (`?`, call, field, index). Comparisons do not chain.

- Literals, variables, tuples `(a, b)`, lists `[a, b, c]`.
- Call: `f(a, b)`. Uniform call syntax: `a.f(b)` means `f(a, b)` whenever `a`
  has no field named `f`. The printer preserves whichever form was written.
- Field: `p.x`. Tuple field: `t.0`, `t.1`.
- Index: `xs[i]` on List returns `T` and traps with kind `bounds` when out of
  range; `get(xs, i)` returns `Option[T]`. `m[k]` on Map likewise traps;
  `map_get` is the safe form.
- Arithmetic on Int and Float separately; mixing is E0101. Int `/` truncates
  toward zero and `%` takes the sign of the dividend, as in C. Division by
  zero traps with kind `div_zero`. Overflow traps with kind `overflow`.
  `div_floor` and `mod_floor` are in the prelude.
- `++` concatenates two Str or two List[T].
- `if c { a } else { b }` is an expression; both branches must agree. `if`
  without `else` has type Unit and the branch must be Unit. `else if` chains.
- `match e { Pat => Expr, Pat => Expr }`. Arms are separated by commas; a
  trailing comma is allowed. Matches must be exhaustive (E0104; the fix lists
  the missing patterns). Unreachable arms are warning W0003. Guards:
  `Pat if cond => Expr`.
- Patterns: `_`, binding `x`, literals, `Ctor(p, p)`, `Ctor`, record
  `P { x: p, .. }` (`..` ignores remaining fields), tuple `(p, p)`, list `[]`,
  `[p, p]`, `[p, ..rest]`, `[..init, p]`, and `p | p` alternatives binding
  the same names.
- Lambda: `|x, y| expr` or `|x, y| { block }`. Captures by value.
- Block: `{ stmt; stmt; expr }`. The final expression is the value; a block
  ending in `;` has type Unit.
- Statements: `let p = e;` (pattern must be irrefutable, E0110), `let x: T = e;`
  (annotation on a plain name, the fix E0107 points to), `let mut x = e;`,
  `x = e;` (x must be `mut`), `while c { }`, `for p in e { }` where e is a
  List, `return e;` (a value is required; write `return ();`), `e;`. As in
  Rust, a statement that is an `if`, `match` or block needs no `;` after it.
- `e?` on `Result[T, E]` evaluates to `T` or returns `Err(e)` from the
  enclosing function, whose return type must be `Result[_, E]`. On
  `Option[T]` it returns `None` likewise.

## 7. Contracts

```
fn total(o: Order) -> Int
  requires all(o.lines, |l| l.qty > 0)
  ensures  result >= 0
{ ... }
```

- Any number of `requires` and `ensures` clauses. Each is a pure Bool
  expression (E0108 if it uses effects). `requires` sees the parameters.
  `ensures` sees the parameters and `result`.
- Contracts are compiled to runtime checks on entry and exit. A failed check
  traps with kind `requires` or `ensures`.
- `assert(c)` and `assert_eq(a, b)` are prelude functions that trap with kind
  `assert` and, for `assert_eq`, include `show` of both values.
- `todo[T]() -> T` traps with kind `todo`. It is how a `declare`d function is
  implemented until it has a body.

A trap prints one JSON object to stderr and exits with code 3:

```json
{"kind": "requires", "fn": "total", "hash": "a3f9…", "clause": "all(o.lines, |l| l.qty > 0)", "args": {"o": "Order { ... }"}}
```

`args` holds `show` of each parameter. For `ensures`, `result` is included.
For `bounds`, `overflow`, `div_zero`, `assert`, `todo`, `unimplemented`:
`fn`, `hash` and a `detail` string.

### 7.1 Fuzzing

`ov test` runs every `test` block and then fuzzes every function that has at
least one contract. `ov` derives a generator from each parameter type:
integers biased toward small magnitudes and boundaries, strings from a small
alphabet, lists up to length 16, sums choosing constructors uniformly,
records and tuples componentwise. Generated inputs that fail `requires` are
discarded (up to a bound, then the function is reported as "unreachable
precondition", warning W0004). An input that traps on `ensures` or any other
kind is shrunk greedily (halve ints, drop list elements, shorten strings) and
reported. A user may override generation for a type with
`gen Name { expr }` returning `List[Name]` (**shortcut**: not in v0.1).

The compiled binary exposes `--fuzz <name>` which reads inputs as JSON lines
on stdin and reports outcomes as JSON lines on stdout. Results are cached by
hash: a function whose hash and whose dependencies' hashes are unchanged is
not re-fuzzed.

## 8. Entry points

`ov run <name>` requires `name` to be a function with no parameters returning
`Unit` or `Result[Unit, E]`, any effects. An `Err(e)` return prints
`{"kind": "error", "error": show(e)}` to stderr and exits 1. `ov build
<name>` produces a native executable `<name>` in the current directory.

## 9. Prelude

All prelude functions are pure unless marked. Generic signatures use `T`,
`U`, `K`, `V`. Full signatures are in `src/prelude.ov`, which is the source of
truth; this list is the required set.

**List**: `len`, `get(xs, i) -> Option[T]`, `push(xs, x)` (returns new list),
`concat`, `map`, `filter`, `fold(xs, init, f)`, `any`, `all`, `sum(List[Int])`,
`sum_float`, `range(a, b)` (half-open), `reverse`, `sort` (Int, Float, Str),
`sort_by(xs, key: fn(T) -> Int)`, `zip`, `enumerate -> List[(Int, T)]`, `join(List[Str], sep)`,
`contains`, `index_of -> Option[Int]`, `head -> Option[T]`, `last -> Option[T]`,
`tail`, `take`, `drop`, `slice(xs, from, to)`, `flat_map`, `flatten`, `is_empty`,
`min_of -> Option[T]`, `max_of -> Option[T]` (Int, Float, Str), `unique`
(Int, Str, Bool), `for_each[T, !e](xs, f: fn(T) -> Unit !e) !e`.

**Str**: `str_len` (code points), `chars -> List[Str]`, `split(s, sep)`,
`trim`, `to_upper`, `to_lower`, `starts_with`, `ends_with`, `str_contains`,
`replace(s, from, to)`, `parse_int -> Option[Int]`, `parse_float -> Option[Float]`,
`show[T](x: T) -> Str`, `char_at(s, i) -> Option[Str]`, `substr(s, from, to)`,
`repeat(s, n)`, `code_point(s) -> Option[Int]`, `from_code_point(Int) -> Str`.

**Map and Set**: `map_new`, `map_get -> Option[V]`, `map_set` (returns new),
`map_remove`, `map_contains`, `map_keys`, `map_values`, `map_len`,
`map_from_list(List[(K, V)])`, `map_to_list`; `set_new`, `set_add`, `set_remove`,
`set_contains`, `set_len`, `set_to_list`, `set_from_list`.

**Option and Result**: `unwrap_or(o, default)`, `is_some`, `is_none`, `is_ok`,
`is_err`, `ok_or(o, e) -> Result`, `option_map`, `result_map`.

**Math**: `abs`, `min`, `max` (Int), `fabs`, `fmin`, `fmax` (Float), `pow(Int, Int)`,
`fpow`, `sqrt`, `floor`, `ceil`, `round`, `to_float`, `to_int` (truncate),
`div_floor`, `mod_floor`.

**IO** (`!io`): `print(s)`, `println(s)`, `read_line() -> Option[Str]`,
`read_all() -> Str`, `args() -> List[Str]`. `exit(code) -> Unit !exit`.
`now_millis() -> Int !time`. `random_int(lo, hi) -> Int !rand`.

**Testing**: `assert`, `assert_eq`, `todo`.

## 10. The store

Code is stored in `.ovstore/store.sqlite` in the project root.

Tables:

- `defs(hash TEXT PRIMARY KEY, kind TEXT, ast TEXT, created_at INTEGER)`.
  `ast` is the canonical JSON serialization (section 10.1). `kind` is
  `fn`, `type`, `declare`, `foreign` or `test`.
- `names(name TEXT PRIMARY KEY, hash TEXT, updated_at INTEGER)`.
- `history(name TEXT, hash TEXT, at INTEGER)`. Append-only.
- `deps(hash TEXT, dep_hash TEXT)`. Direct dependencies.
- `cache(hash TEXT, kind TEXT, blob BLOB, at INTEGER)`. Emitted C, fuzz
  results, keyed by hash.

### 10.1 Hashing

A definition's hash is the SHA-256 of its canonical form, hex-encoded. The
canonical form is the AST serialized as JSON with:

- the definition's own name removed;
- every local variable replaced by its de Bruijn index;
- every reference to another top-level definition replaced by that
  definition's hash;
- every reference to a type replaced by that type's hash (built-in types
  hash to their name);
- constructor references replaced by `(type hash, constructor index)`;
- record field names kept (they are part of the type's identity);
- type parameter names replaced by position;
- all source positions and comments removed.

Renaming a definition therefore never changes its hash. Renaming a local
variable never changes the hash. Changing the body does.

Mutually recursive definitions form a cycle. The cycle's canonical form is the
list of its members' canonical forms, each member referring to the others by
position in the cycle, ordered by the lexicographic order of their canonical
forms before cycle references are resolved. The cycle hash is the SHA-256 of
that list, and member `i` has hash `<cycle hash>.<i>`.

Short hash prefixes are accepted anywhere a hash is expected when they are
unambiguous.

### 10.2 Commands

All commands take `--json` for machine output. Without it, output is
human-readable text with the same content. Diagnostics (section 11) are
always JSON.

| Command | Behavior |
|---|---|
| `ov add <file.ov>...` | Parse and typecheck the files against the store, hash each definition, append to `defs`, bind names. A definition that already exists by hash is not duplicated; its name is rebound. Prints `name hash` per definition. |
| `ov view <name-or-hash>` | Pretty-print the definition in canonical form. |
| `ov edit <name> [--file f]` | Read one definition's text from the file or stdin, parse, typecheck, store, rebind the name. The definition's name in the text must match. |
| `ov rm <name>` | Unbind the name. The definition stays in `defs`. |
| `ov ls [--kind k]` | List bound names with hashes. |
| `ov deps <name>` / `ov dependents <name>` | Transitive dependency graph by name and hash, as a list ordered by depth. |
| `ov ctx <name> [--depth N]` | The minimal context for working on `name`: its full source, then the signature (including effects and contracts) of each definition it references directly, then the full definition of every type those signatures mention. `--depth 1` adds the bodies of direct dependencies. This is the command the agent uses instead of reading files. |
| `ov check [name]` | Typecheck, effect-check, exhaustiveness-check one definition or the whole store. Emits diagnostics. Exit 0 if no errors. |
| `ov build <name>` | Monomorphize from `name`, emit one C file per reachable definition (cached by hash), compile with `cc`, link `runtime/rt.c` and `-lgc`. Produces `./<name>`. |
| `ov run <name>` | `ov build` then execute, forwarding stdin, stdout, stderr and exit code. |
| `ov test [name]` | Run test blocks and fuzz contracted functions, restricted to `name`'s dependency closure if given. Results cached by hash. |
| `ov diff <a> <b>` | Structural diff of two definitions (names or hashes): changed subtrees as before and after text. |
| `ov history <name>` | Every hash the name has pointed to, newest first. |
| `ov fmt <file.ov>` | Rewrite a file in canonical form without touching the store. |

### 10.3 Canonical printing

`ov view` and `ov fmt` produce the canonical text. Rules: two-space
indentation; one blank line between definitions; signature on one line if it
fits in 100 columns, otherwise one parameter per line; `requires` and
`ensures` each on their own line indented two spaces; match arms one per
line; record fields in declaration order; trailing commas in multi-line
lists; `a.f(b)` preserved as written. `parse(print(ast))` must equal `ast`,
and the hash must be unchanged.

## 11. Diagnostics

Every diagnostic is one JSON object. `ov check` prints one per line.

```json
{
  "code": "E0104",
  "severity": "error",
  "name": "describe",
  "hash": "a3f9c2…",
  "span": {"line": 3, "col": 3, "end_line": 7, "end_col": 4},
  "message": "non-exhaustive match on ChargeError",
  "expected": ["AlreadyPaid"],
  "actual": ["Declined(_)", "Timeout"],
  "fixes": [
    {"rank": 1, "description": "add missing arm", "patch": "    AlreadyPaid => todo(),"}
  ]
}
```

Fields `expected`, `actual` and `fixes` are present when they apply. `span` is
relative to the definition's canonical text, which is what the agent sees
from `ov view`. For `ov add` and `ov fmt`, `span` is relative to the file and
a `file` field is added.

Codes:

| Code | Meaning | Required content |
|---|---|---|
| E0000 | Command-line or file error | `message` |
| E0001 | Lexical error | `message` |
| E0002 | Parse error | `expected` tokens |
| E0100 | Unknown name | `fixes`: nearest names by edit distance |
| E0101 | Type mismatch | `expected`, `actual` types |
| E0102 | Wrong argument count | `expected`, `actual` counts |
| E0103 | Unknown field or constructor | `fixes`: nearest field names |
| E0104 | Non-exhaustive match | `expected`: missing patterns; `fixes`: arms to add |
| E0105 | Missing effect | `expected`: the label; `fixes`: the signature line with it added |
| E0106 | Unknown type | `fixes`: nearest type names |
| E0107 | Cannot infer type | `fixes`: where to annotate |
| E0108 | Effect in contract | which call |
| E0109 | Duplicate definition | the other location |
| E0110 | Refutable pattern in let | `fixes`: rewrite as match |
| E0111 | Invalid effect parameter use | |
| E0112 | Assignment to immutable | `fixes`: add `mut` |
| E0113 | `?` outside a compatible function | `expected` return type |
| E0114 | Invalid foreign signature | which type |
| E0115 | Recursive type without indirection through a constructor | |
| W0001 | Unimplemented `declare` | |
| W0002 | Unused variable | `fixes`: rename to `_x` |
| W0003 | Unreachable match arm | |
| W0004 | Unreachable precondition during fuzzing | |

## 12. Compilation to C

- Monomorphization: every generic definition is instantiated per concrete
  type argument tuple reached from the entry point. Instances are named
  `ov_<name>_<hash8>_<instance8>`.
- One C translation unit per definition instance, `#include "rt.h"`, plus a
  generated `ov_types.h` with every monomorphized struct. Emitted C is cached
  in the store by `(hash, instance)`.
- Representation: `Int` is `int64_t`, `Float` is `double`, `Bool` is `bool`,
  `Unit` is `uint8_t` (value 0), `Str` is `ov_str*`, `List[T]` is
  `ov_list*` holding `ov_value` slots (**shortcut**: unboxed specializations
  later), records are structs, sums are `struct { uint32_t tag; union {...} }`,
  tuples are structs, closures are `struct { void* fn; void* env; }`.
- All heap allocation goes through `GC_MALLOC`. Nothing is freed.
- Int arithmetic uses `__builtin_*_overflow` and traps on overflow.
- Contracts become checks at function entry and before every return.
- `show` is generated per monomorphized type.
- Traps call `ov_trap(kind, fn, hash, detail_json)` which prints and exits 3.
- `main` is generated for the entry point: initializes the GC, captures argv
  for `args()`, calls the entry, maps `Err` to exit 1.
- The `--fuzz <name>` protocol (section 7.1) is generated for every
  contracted function reachable from the entry point when building for `ov
  test`.

## 13. Example

```
type Line  = { sku: Str, qty: Int, unit_cents: Int }
type Order = { id: Int, lines: List[Line], paid: Bool }
type ChargeError = Declined(reason: Str) | Timeout | AlreadyPaid

fn total(o: Order) -> Int
  requires all(o.lines, |l| l.qty > 0 && l.unit_cents >= 0)
  ensures result >= 0
{
  sum(map(o.lines, |l| l.qty * l.unit_cents))
}

fn describe(e: ChargeError) -> Str {
  match e {
    Declined(reason) => "declined: " ++ reason,
    Timeout => "timed out",
    AlreadyPaid => "already paid",
  }
}

declare fn charge(o: Order) -> Result[Int, ChargeError] !net

fn main() -> Unit !io {
  let o = Order { id: 1, lines: [Line { sku: "a", qty: 2, unit_cents: 150 }], paid: false };
  println(show(total(o)));
  println(describe(Timeout));
}

test total_of_empty_order {
  assert_eq(total(Order { id: 0, lines: [], paid: false }), 0);
}
```

`ov run main` prints `300` then `timed out`.
