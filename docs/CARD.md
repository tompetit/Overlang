# Overlang card

Overlang is a statically typed, mostly functional language that compiles to C via `ov`. Code lives in a hash-addressed store; `.ov` text is a projection. This is the core card. `ov ctx <name>` ends with `# Syntax notes`: the card sections for the constructs its code uses. `ov card <topic>` prints one section, `ov card --full` all. Topics: records sums generics closures tuples patterns loops contracts effects declare foreign lists strings maps math programs.

1. Every top-level `fn` has a full signature: parameter types, return type, effects (`!io`). Nothing is inferred at top level.
2. No null, no exceptions. Use `Option[T]` / `Result[T, E]` and `?`.
3. `match` must be exhaustive (E0104).

```overlang
fn parse_qty(s: Str) -> Result[Int, Str] {
  match parse_int(trim(s)) {                                   // Option[Int]
    Some(n) if n >= 0 => Ok(n),
    Some(n) => Err("negative: " ++ show(n)),
    None => Err("not a number: " ++ s),
  }
}

fn add_qtys(a: Str, b: Str) -> Result[Int, Str] {
  let x = parse_qty(a)?;                                       // ? returns Err early
  let y = parse_qty(b)?;
  Ok(x + y)
}

fn log(msg: Str) -> Unit !io {                                 // effect label
  println("[log] " ++ msg);
}

test add_qtys_works {
  assert_eq(add_qtys("2", " 3"), Ok(5));
  assert(is_err(add_qtys("x", "1")));
}
```

| Thing | Form |
|---|---|
| Literals | `1_000` `2.5` (Float always has `.`) `"s\n"` `true` `()`; no interpolation, use `++` and `show` |
| Comments | `// to end of line` only |
| Types | `Int` (i64, traps on overflow) `Float` `Bool` `Str` `Unit` `List[T]` `Option[T]` `Result[T, E]` |
| Function | `fn f(a: Int, s: Str) -> Str !io { body }` (last expression is the value) |
| Test | `test name { ... }` (Unit body, any effects) |
| Let | `let x = e;` `let x: T = e;` (immutable) |
| Control | `if c { a } else { b }` (expression, branches agree; `if` without `else` is Unit) `return e;` (value required: `return ();`) |
| match | `match e { P => e, P if g => e, }` commas between arms. Patterns `_` `x` `1` `"s"` `Some(x)` `None` `Ok(v)` `Err(e)` `[]` `[x, ..rest]` |
| Lists | `[1, 2]`; `xs[i]` traps (`bounds`) when out of range; `get(xs, i)` gives `Option` |
| Call | `f(a, b)` equals `a.f(b)` unless `a` has a field `f` |
| Operators (low to high) | `\|\|`, `&&`, `== != < <= > >=` (no chaining), `++` (Str or List), `+ -`, `* / %`, unary `- !`, postfix `? () .f [i]` |
| `?` | on `Result[T, E]` gives `T` or returns `Err(e)`; enclosing fn must return `Result[_, E]` (else E0113). On `Option` returns `None`; fn must return `Option[_]` |

Statements `if`/`match`/block need no `;`. A block ending in `;` has type Unit. `==` is structural. No implicit conversion: `to_float`, `to_int`. Names: values and functions `snake_case` (`fn Solve(N: Int)` is E0002), types and constructors `Capitalized`. Flat namespace, definitions in any order, prelude names cannot be redefined. Keywords are not names: `fn type declare foreign link intrinsic let mut if else match while for in return break continue requires ensures test true false`. `let xs = [];` alone is E0107: write `let xs: List[Int] = [];`.

Effects: a call is legal only if the callee's effects are a subset of the caller's declared effects; no labels means pure. Calling `println` (`!io`) from a fn without `!io` is E0105, whose fix `patch` is the whole signature line with the label added. Apply it, then fix the callers, up to `main`.

## Prelude (basics; more under lists, strings, maps, math)

All pure unless marked, always in scope. **List**: `len` `get(xs,i)->Option[T]` `push(xs,x)` (new list) `concat` `slice(xs,from,to)` `reverse` `head`/`last->Option[T]` `is_empty` `contains(xs,x)` `range(a,b)` (half-open) `sum` `sort` `join(List[Str],sep)->Str` `map(xs,f)` `filter(xs,p)` `fold(xs,init,f)` `any(xs,p)` `all(xs,p)` (lambdas: `|x| x + 1`). **Str**: `str_len` (code points) `split(s,sep)` `trim` `lines` `starts_with` `ends_with` `str_contains` `replace(s,from,to)` `substr(s,from,to)` `parse_int->Option[Int]` `show(x)->Str` (any value; `println(show(n))`). **Option, Result**: `unwrap_or(o,default)` `is_some` `is_none` `is_ok` `is_err` `ok_or(o,e)->Result` `option_map` `result_map`. **Int**: `abs` `min` `max` `pow`. **IO** (`!io`): `print` `println` `read_line->Option[Str]` `read_all` `args`. **Testing**: `assert(c)` `assert_eq(a,b)` `todo()` (placeholder body).

## Toolchain

Diagnostics are JSON, one object per line on stdout: `code`, `severity`, `name`, `span` (`line col end_line end_col`), `message`, and when relevant `expected`, `actual`, `fixes` (`rank`, `description`, `patch`). A `patch` is often exact text to apply. E0100 unknown name, E0101 type mismatch, E0104 missing match arm (patch: the arms), E0107 cannot infer, E0109 duplicate name. W-codes are warnings. A trap (failed assert, `bounds`, ...) prints one JSON object to stderr and exits 3. Exit codes: 0 ok, 1 errors (nothing stored or built), 2 stored but a dependent has errors, 3 trap or failed test.

| Command | Use |
|---|---|
| `ov check [file.ov... \| name]` | typecheck, effects, exhaustiveness |
| `ov test [file.ov... \| name]` | run `test` blocks in order (`ok <name>`), stop at the first failure, then fuzz contracted functions |
| `ov run <file.ov... \| name> [-- args]` | build and run `main` |
| `ov ls` | bound names and hashes |
| `ov ctx <name>` | source of `name`, signatures and types of what it uses, syntax notes (`--depth 1` adds bodies, `--no-notes`) |
| `ov view <name>` | one definition |
| `ov add <file.ov>...` | store new definitions (functions, types, tests), print `name hash` |
| `ov edit <name> [--file f]` | replace a bound definition (new names go through `add`, else E0100) |
| `ov rm <name>` `ov dependents <name>` | unbind / callers |

With `.ovstore/` present, `check`, `test` and `run` take a stored name. After `add`/`edit`, dependents are re-checked and their errors follow the output, with the dependent's `name` and `hash`; fix those next.

<!-- topic records: record-type record-expr record-pattern field-access -->
## Records

`type P[T] = { x: T, y: Int }`; make `P { x: 1, y: 2 }`; update `P { ..p, y: 3 }` (copy with `y` changed); read `p.x`; pattern `P { x: p, .. }` (`..` ignores the other fields). Records are immutable values; `==` compares them field by field. `p.f(a)` calls the field `f` if `P` has one, else `f(p, a)`.

<!-- topic sums: sum-type ctor -->
## Sum types

`type S = Circle(r: Float) | Rect(w: Float, h: Float) | Empty`; build `Circle(1.0)`, `Rect(2.0, 3.0)`, `Empty` (positional arguments, no parens when no fields). Match with `Circle(r) =>`, `Empty =>`. A `match` must cover every constructor: a missing one is E0104, whose fix `patch` is the missing arms as `Ctor(..) => todo(),`; paste it and fill in the bodies. Unreachable arms are warning W0003.

<!-- topic generics: type-params -->
## Generics

Type parameters go in brackets after the name: `fn find[T](xs: List[T], p: fn(T) -> Bool) -> Option[T]`, `type Pair[A, B] = { a: A, b: B }`. They are inferred at call sites; when nothing fixes them (`assert_eq(max_of([]), None)`, `let xs = [];`) it is E0107: annotate one side.

```overlang
fn find[T](xs: List[T], p: fn(T) -> Bool) -> Option[T] {
  match xs {
    [] => None,
    [x, ..rest] => if p(x) { Some(x) } else { find(rest, p) },
  }
}
```

<!-- topic closures: lambda-expr fn-type -->
## Lambdas and function types

Lambda: `|x, y| e` or `|x: Int| { block }`; captures by value; its effects are inferred from its body. Function type: `fn(A, B) -> R`, with effects `fn(A) -> R !io`. `["a:2", "b"].map(|s| str_len(s))` is `map(xs, f)` in uniform call form. `break`/`continue` inside a lambda do not see an enclosing loop (E0117).

<!-- topic tuples: tuple -->
## Tuples

`(A, B)` (2 to 8 elements); make `(1, "a")`; read `t.0`, `t.1`; destructure `let (a, b) = t;` (pattern must be irrefutable) or `for (i, x) in enumerate(xs) { }`; pattern `(p, q)`.

<!-- topic patterns: list-pattern or-pattern -->
## Patterns

`_` `x` `1` `"s"` `true` `Ctor(p, q)` `Ctor` `P { x: p, .. }` `(p, q)` `[]` `[p, q]` `[p, ..rest]` `[..init, p]` `[p, ..]` `[.., p]` (a bare `..` binds nothing) `p | q` (alternatives binding the same names). Guards: `P if cond => e`. A `let` pattern must be irrefutable.

<!-- topic loops: while for break continue let-mut assign -->
## Loops and mutation

`let mut x = e;` then `x = e;` (assigning to a non-`mut` binding is E0112; the fix `patch` is `let mut x`). `while c { }`, `for x in list { }`, `break;` `continue;` (innermost loop, no labels, not from inside a lambda). Local mutation is not an effect. A function whose body ends in a loop still needs a tail expression: `while true { ... return x; } 0`.

```overlang
fn total(xs: List[Int]) -> Int {
  let mut acc = 0;
  let mut i = 0;
  while i < len(xs) {
    acc = acc + xs[i];                                         // [i] traps if out of range
    i = i + 1;
  }
  acc
}
```

<!-- topic contracts: requires ensures -->
## Contracts and traps

```overlang
fn line_total(qty: Int, cents: Int) -> Int
  requires qty >= 0 && qty < 1000 && cents >= 0               // checked on entry
  ensures result >= 0                                          // checked on exit
{
  qty * cents
}
```

- `requires c` / `ensures c`: pure Bool expressions, any number. `ensures` may use `result`. No effects inside (E0108).
- A trap prints one JSON object to stderr and exits with code 3: `{"kind": "requires", "fn": "total", "hash": "a3f9…", "clause": "…", "args": {"o": "…"}}`. Kinds: `requires ensures assert todo unimplemented bounds overflow div_zero foreign_null`. Fix the caller or the contract, not the trap.
- `ov test` fuzzes every function with contracts using generated inputs; a failing input is shrunk and reported. Flags: `--cases N` (default 200), `--seed S` or `OV_SEED`, `--name f` (fuzz only f's closure).

<!-- topic effects: effect-params effect-label -->
## Effects

Labels: `io` (print, println, flush, read_line, read_all, args), `fs`, `net`, `rand` (random_int), `time` (now_millis), `exit` (exit). Any identifier is allowed as your own label (`!db`). List several as `!io !net`. A lambda's effects are inferred from its body.

Higher-order functions take one effect parameter `!e` in the same brackets as type parameters and use it on the function argument and the result: `fn map[T, U, !e](xs: List[T], f: fn(T) -> U !e) -> List[U] !e`. The caller pays for the effects of the lambda passed in. E0105 has `expected: ["!io"]` (the missing labels).

<!-- topic declare: declare -->
## declare

`declare fn f(a: Int) -> Int !net` is a signature with no body (contracts allowed). Calls to it typecheck; `ov check` warns W0001, and executing it traps `unimplemented`. Give it a body (or `todo()`) before running.

<!-- topic foreign: foreign -->
## foreign

`foreign c "header.h" link "lib" fn name(p: Int) -> Str !eff = "c_symbol"` binds a C function. The header is included verbatim (looked up next to the source file, or in the current directory for store builds); `link "lib"` (optional) adds `-llib`; `= "c_symbol"` (optional) names the C function when it differs. Parameters are Int, Float, Bool, Str or Unit; the return may also be `Option[Str]`, where a NULL becomes `None`. Str crosses as a NUL-terminated UTF-8 `const char*`; a returned string is copied, and NULL for a plain `Str` traps `foreign_null`. The effects are whatever the declaration says; the compiler trusts them. A bad parameter type is E0114.

<!-- topic lists: take drop tail for_each flat_map flatten sum_float zip enumerate index_of count filter_map take_while drop_while max_by min_by count_by sum_by min_of max_of sort_by sort_by_str unique -->
## More list functions

`take(xs,n)` `drop(xs,n)` `tail` `index_of(xs,x)->Option[Int]` `for_each(xs,fn(T)->Unit !e)->Unit !e` `flat_map(xs,fn(T)->List[U] !e)` `flatten(List[List[T]])` `sum_float(List[Float])->Float` `zip(xs,ys)->List[(A,B)]` `enumerate(xs)->List[(Int,T)]` `count(xs,p)->Int !e` `filter_map(xs,fn(T)->Option[U] !e)->List[U] !e` `take_while(xs,p)` `drop_while(xs,p)` `max_by(xs,key:fn(T)->Int)->Option[T]` `min_by` (first wins ties) `count_by(xs,key:fn(T)->K)->Map[K,Int]` `sum_by(xs,fn(T)->Int !e)->Int !e` `min_of(xs)->Option[T]` `max_of(xs)->Option[T]` (`sort`, `min_of`, `max_of`: Int, Float, Str only) `sort_by(xs,key:fn(T)->Int)` `sort_by_str(xs,key:fn(T)->Str)` (both stable) `unique(xs)` (Int, Str, Bool only). Full signatures of the basics: `map(xs,fn(T)->U !e)->List[U] !e` `filter(xs,fn(T)->Bool !e) !e` `fold(xs,init:A,fn(A,T)->A !e)->A !e`.

<!-- topic strings: chars to_upper to_lower repeat char_at code_point from_code_point parse_float split_once str_index_of is_digit is_alpha is_alnum is_space is_upper is_lower split_ws -->
## More string functions

`chars(s)->List[Str]` `to_upper` `to_lower` `repeat(s,n)` `char_at(s,i)->Option[Str]` `code_point(s)->Option[Int]` `from_code_point(Int)->Str` `parse_float(s)->Option[Float]` `split_once(s,sep)->Option[(Str,Str)]` `str_index_of(s,needle)->Option[Int]` (code points) `is_digit` `is_alpha` `is_alnum` `is_space` `is_upper` `is_lower` (Str->Bool, ASCII, first code point) `split_ws(s)` (runs of whitespace, no empty pieces). `lines(s)` drops one trailing empty line. `show` quotes a Str and shows `3.0` as `3`; it takes any non-function type.

<!-- topic maps: Map Set map_new map_get map_set map_remove map_contains map_keys map_values map_len map_from_list map_to_list set_new set_add set_remove set_contains set_len set_to_list set_from_list count_by -->
## Map and Set

Types `Map[K, V]` and `Set[T]`; keys Int, Str or Bool only; all immutable, updates return a new value. `map_new()` `map_get(m,k)->Option[V]` `map_set(m,k,v)` `map_remove(m,k)` `map_contains(m,k)` `map_keys(m)` `map_values(m)` `map_len(m)` `map_from_list(List[(K,V)])` `map_to_list(m)`; `m[k]` traps when absent, `map_get` is safe. `set_new()` `set_add(s,x)` `set_remove(s,x)` `set_contains(s,x)` `set_len(s)` `set_to_list(s)` `set_from_list(xs)`.

<!-- topic math: Float float-literal to_float to_int div_floor mod_floor gcd lcm is_prime digits from_digits to_base from_base fabs fmin fmax fpow sqrt floor ceil round -->
## Numbers

Literals `2.5`, `2.0e10`: a Float literal always has a `.`. Int and Float never mix: `1 + 2.5` and `len(xs) * 0.5` are E0101; convert with `to_float(Int)->Float` and `to_int(Float)->Int` (truncates). `/` truncates toward zero and `%` takes the dividend's sign; `div_floor`, `mod_floor` floor instead. Division by zero traps `div_zero`, Int overflow traps `overflow`. `< <= > >=` work on Int, Float and Str only. Int: `abs` `min` `max` `pow(base,exp)` (exp >= 0) `div_floor` `mod_floor` `gcd` `lcm` `is_prime` `digits(n)->List[Int]` (of `abs(n)`, `[0]` for 0) `from_digits(ds)` `to_base(n,base)->Str` (2..36, lowercase) `from_base(s,base)->Option[Int]`. Float: `fabs` `fmin` `fmax` `fpow` `sqrt` `floor` `ceil` `round` (all Float to Float).

<!-- topic programs: main-fn args exit flush read_line read_all now_millis random_int -->
## Programs and the rest of the toolchain

`ov run` needs a no-parameter `main` (`--entry f` to pick another) returning `Unit` or `Result[Unit, E]`; `Err(e)` prints `{"kind":"error",...}` to stderr, exit 1. `ov run main -- a b` gives the program `args() == ["a", "b"]` (no program name). `print(s)` `println(s)` `flush()` `read_line()->Option[Str]` `read_all()->Str` `args()->List[Str]` are `!io`; stdout is block-buffered unless a terminal, so `flush()` before a server blocks. `exit(code)->Unit !exit`; `now_millis()->Int !time`; `random_int(lo,hi)->Int !rand`.

`ov check|build|run|test` take `.ov` files (several are one program) or, when `.ovstore/` exists, a stored name. `--json` is taken by `add view edit rm ls deps dependents ctx history diff` and `test`; not by `check` (always JSON), `build`, `run`, `fmt`, `export`. Warnings from `add`/`edit` go to stderr; with `--json`, dependents' errors are in `dependents_diagnostics`.

| Command | Use |
|---|---|
| `ov build <file.ov... \| name> [-o out]` | native executable |
| `ov fmt <file.ov> [--write]` | print the canonical text (`--write`: rewrite the file) |
| `ov view <name> --raw` `ov history <name>` `ov diff <a> <b>` | as written, with comments / past hashes / diff |
| `ov ls [--kind k] [--sort name]` | in the order first added (`--sort name`: alphabetical) |
| `ov export [file]` | every bound definition in `ov ls` order as one `.ov` file (no comments) |

