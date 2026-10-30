# Overlang card

Overlang is a statically typed, mostly functional language that compiles to C via `ov`. Code lives in a hash-addressed store; `.ov` text is a projection. Never forget:

1. Every top-level `fn` has a full signature: parameter types, return type, effects (`!io`). Nothing is inferred at top level.
2. No null, no exceptions. Use `Option[T]` / `Result[T, E]` and `?`.
3. `match` must be exhaustive (E0104).

## Example

```overlang
type Item = { name: Str, qty: Int, cents: Int }                // record
type Cmd = Add(item: Item) | Remove(name: Str) | Clear         // sum type

fn line_total(i: Item) -> Int
  requires i.qty >= 0 && i.cents >= 0                          // checked on entry
  ensures result >= 0                                          // checked on exit
{
  i.qty * i.cents
}

fn find[T](xs: List[T], p: fn(T) -> Bool) -> Option[T] {       // generic
  match xs {
    [] => None,
    [x, ..rest] => if p(x) { Some(x) } else { find(rest, p) },
  }
}

fn parse_item(s: Str) -> Result[Item, Str] {
  match split(s, ":") {
    [name, q] if str_len(name) > 0 => {                        // list pattern + guard
      let qty = ok_or(parse_int(q), "bad qty: " ++ q)?;        // ? returns Err early
      Ok(Item { name: name, qty: qty, cents: 100 })
    },
    _ => Err("want name:qty, got " ++ s),
  }
}

fn total(items: List[Item]) -> Int {
  let mut acc = 0;
  let mut i = 0;
  while i < len(items) {
    acc = acc + line_total(items[i]);                          // [i] traps if out of range
    i = i + 1;
  }
  acc
}

fn log(msg: Str) -> Unit !io {                                 // effect label
  println("[log] " ++ msg);
}

fn main() -> Unit !io {
  let rows = ["a:2", "b:x", "c"].map(|s| show(parse_item(s))); // uniform call, lambda
  log(join(rows, "; "));
  match Add(Item { name: "pen", qty: 3, cents: 150 }) {
    Add(item) => log(show(total([item]))),
    Remove(n) => log(n),
    Clear => log("clear"),
  }
  log(show(find([1, 5, 8], |n| n > 4)));
}

test total_empty {
  assert_eq(total([]), 0);
}
```

## Syntax

| Thing | Form |
|---|---|
| Literals | `1_000` `2.5` `2.0e10` (Float always has `.`) `"s\n\u{e9}"` `true` `()`; no interpolation, use `++` and `show` |
| Comments | `// to end of line` only |
| Types | `Int` (i64, traps on overflow) `Float` `Bool` `Str` `Unit` `List[T]` `Map[K, V]` `Set[T]` `(A, B)` (2 to 8) `fn(A) -> R !io` |
| Record | `type P[T] = { x: T, y: Int }`; make `P { x: 1, y: 2 }`; update `P { ..p, y: 3 }`; read `p.x` |
| Sum | `type S = Circle(r: Float) \| Rect(w: Float, h: Float) \| Empty`; use `Circle(1.0)`, `Empty` (positional, no parens when no fields) |
| Function | `fn f[T, U, !e](a: T, g: fn(T) -> U !e) -> List[U] !e !io requires c ensures result > 0 { body }` |
| declare | `declare fn f(a: Int) -> Int !net` (signature, no body) |
| Test | `test name { ... }` (Unit body, any effects) |
| Let | `let x = e;` `let x: T = e;` `let mut x = e;` `x = e;` `let (a, b) = t;` (pattern must be irrefutable) |
| Lambda | `\|x, y\| e` `\|x: Int\| { block }`; captures by value |
| Control | `if c { a } else { b }` (expression, branches agree) `while c { }` `for x in list { }` `break;` `continue;` (innermost loop, not from a lambda) `return e;` (value required: `return ();`) |
| match | `match e { P => e, P if g => e, }` commas between arms |
| Patterns | `_` `x` `1` `"s"` `true` `Ctor(p, q)` `Ctor` `P { x: p, .. }` `(p, q)` `[]` `[p, q]` `[p, ..rest]` `[..init, p]` `[p, ..]` `[.., p]` (bare `..` binds nothing) `p \| q` (same bindings) |
| Tuples | `(1, "a")`, `t.0`, `t.1` |
| Lists | `[1, 2]`; `xs[i]` traps (`bounds`); `get(xs, i)` gives `Option`; `m[k]` traps, `map_get` is safe |
| Call | `f(a, b)` equals `a.f(b)` unless `a` has a field `f` |
| Operators (low to high) | `\|\|`, `&&`, `== != < <= > >=` (no chaining), `++` (Str or List), `+ -`, `* / %`, unary `- !`, postfix `? () .f [i]` |
| `?` | on `Result[T, E]` gives `T` or returns `Err(e)`; enclosing fn must return `Result[_, E]`. On `Option` returns `None`; fn must return `Option[_]` |

Statements `if`/`match`/block need no `;`. A block ending in `;` has type Unit. `==` is structural on everything except functions. `< <= > >=` only on Int, Float, Str. No implicit conversion: `to_float`, `to_int`. `/` truncates toward zero, `%` takes the dividend's sign; use `div_floor`, `mod_floor` for floor semantics. Division by zero traps. Names: values `snake_case`, types and constructors `Capitalized`, effects lowercase after `!`. Flat namespace, no modules, definitions in any order, prelude names cannot be redefined.

## Effects

A call is legal only if the callee's effects are a subset of the caller's declared effects. Pure means no labels. Labels: `io` (print, println, flush, read_line, read_all, args), `fs`, `net`, `rand` (random_int), `time` (now_millis), `exit` (exit). Any identifier is allowed as your own label (`!db`). List several as `!io !net`. Local mutation (`let mut`, `while`, `for`) is not an effect. A lambda's effects are inferred from its body.

Higher-order functions take one effect parameter `!e` in the same brackets as type parameters and use it on the function argument and the result: `fn map[T, U, !e](xs: List[T], f: fn(T) -> U !e) -> List[U] !e`. The caller pays for the effects of the lambda passed in.

`E0105` has `expected: ["!io"]` and a fix `patch` that is the whole signature line with the label added. Apply it, then fix the callers, up to `main`.

## Contracts and traps

- `requires c` / `ensures c`: pure Bool expressions, any number. `ensures` may use `result`. No effects inside (E0108).
- `assert(c)`, `assert_eq(a, b)` trap with kind `assert`. `todo()` traps with kind `todo`; use it as a placeholder body.
- `declare fn` typechecks and warns W0001, but executing it traps `unimplemented`. Give it a body (or `todo()`) before running.
- A trap prints one JSON object to stderr and exits with code 3: `{"kind": "requires", "fn": "total", "hash": "a3f9…", "clause": "…", "args": {"o": "…"}}`. Kinds: `requires ensures assert todo unimplemented bounds overflow div_zero foreign_null`. Fix the caller or the contract, not the trap.
- `ov test` fuzzes every function with contracts using generated inputs; a failing input is shrunk and reported.

## Prelude

All pure unless marked. Prelude names are always in scope.

**List**: `len(List[T])->Int` `get(xs,i)->Option[T]` `push(xs,x)->List[T]` (new list) `concat(xs,ys)` `slice(xs,from,to)` `reverse` `take(xs,n)` `drop(xs,n)` `tail` `head->Option[T]` `last->Option[T]` `is_empty->Bool` `map(xs,fn(T)->U !e)->List[U] !e` `filter(xs,fn(T)->Bool !e) !e` `fold(xs,init:A,fn(A,T)->A !e)->A !e` `for_each(xs,fn(T)->Unit !e)->Unit !e` `any(xs,p)` `all(xs,p)` `flat_map(xs,fn(T)->List[U] !e)` `flatten(List[List[T]])` `sum(List[Int])->Int` `sum_float(List[Float])->Float` `range(a,b)->List[Int]` (half-open) `zip(xs,ys)->List[(A,B)]` `enumerate(xs)->List[(Int,T)]` `contains(xs,x)->Bool` `index_of(xs,x)->Option[Int]` `count(xs,p)->Int !e` `filter_map(xs,fn(T)->Option[U] !e)->List[U] !e` `take_while(xs,p)` `drop_while(xs,p)` `max_by(xs,key:fn(T)->Int)->Option[T]` `min_by` (first wins ties) `count_by(xs,key:fn(T)->K)->Map[K,Int]` `sum_by(xs,fn(T)->Int !e)->Int !e` `join(List[Str],sep)->Str` `sort(xs)` `min_of(xs)->Option[T]` `max_of(xs)->Option[T]` (sort, min_of, max_of: Int, Float, Str only) `sort_by(xs,key:fn(T)->Int)` `sort_by_str(xs,key:fn(T)->Str)` (both stable) `unique(xs)` (Int, Str, Bool only).

**Str**: `str_len(s)->Int` (code points) `chars(s)->List[Str]` `split(s,sep)->List[Str]` `trim` `to_upper` `to_lower` `starts_with(s,p)` `ends_with(s,p)` `str_contains(s,needle)` `replace(s,from,to)` `substr(s,from,to)` `repeat(s,n)` `char_at(s,i)->Option[Str]` `code_point(s)->Option[Int]` `from_code_point(Int)->Str` `parse_int(s)->Option[Int]` `parse_float(s)->Option[Float]` `split_once(s,sep)->Option[(Str,Str)]` `str_index_of(s,needle)->Option[Int]` (code points) `lines(s)->List[Str]` (drops one trailing empty line) `is_digit` `is_alpha` `is_alnum` `is_space` `is_upper` `is_lower` (Str->Bool, ASCII, first code point) `split_ws(s)` (runs of whitespace, no empty pieces) `show[T](x:T)->Str` (any non-function type; Str shows quoted, `3.0` shows `3`; `println` takes Str, so `println(show(n))`).

**Map, Set** (keys Int, Str or Bool only; all immutable, updates return a new value): `map_new()` `map_get(m,k)->Option[V]` `map_set(m,k,v)` `map_remove(m,k)` `map_contains(m,k)` `map_keys(m)` `map_values(m)` `map_len(m)` `map_from_list(List[(K,V)])` `map_to_list(m)`; `set_new()` `set_add(s,x)` `set_remove(s,x)` `set_contains(s,x)` `set_len(s)` `set_to_list(s)` `set_from_list(xs)`.

**Option, Result**: `unwrap_or(o,default)` `is_some` `is_none` `is_ok` `is_err` `ok_or(o:Option[T],e:E)->Result[T,E]` `option_map(o,f)` `result_map(r,f)`.

**Math**: Int: `abs` `min` `max` `pow(base,exp)` (exp >= 0) `div_floor` `mod_floor` `gcd` `lcm` `is_prime` `digits(n)->List[Int]` (of `abs(n)`, `[0]` for 0) `from_digits(ds)` `to_base(n,base)->Str` (2..36, lowercase) `from_base(s,base)->Option[Int]`. Float: `fabs` `fmin` `fmax` `fpow` `sqrt` `floor` `ceil` `round` (all Float to Float). Convert: `to_float(Int)->Float`, `to_int(Float)->Int` (truncates).

**IO and env**: `print(s)` `println(s)` `flush()->Unit` `read_line()->Option[Str]` `read_all()->Str` `args()->List[Str]` (after `--`, no program name) all `!io`; stdout is block-buffered unless a terminal, so `flush()` before a server blocks; `exit(code)->Unit !exit`; `now_millis()->Int !time`; `random_int(lo,hi)->Int !rand`.

**Testing**: `assert(Bool)` `assert_eq(a,b)` `todo[T]()->T`.

## Toolchain

Diagnostics are JSON, one object per line, on stdout. `ov check|build|run|test` take a `.ov` file (several are one program) or, when `.ovstore/` exists, a stored name. `--json` is taken by `add view edit rm ls deps dependents ctx history diff` and `test`; not by `check` (always JSON), `build`, `run`, `fmt`, `export`. Exit codes: 0 ok; 1 errors (nothing stored or built); 2 `add`/`edit` stored the code but a dependent now has errors; 3 trap (in `ov test`, also a failed test or fuzz case). `ov edit` only replaces a bound name (else E0100: new definitions go through `ov add`). After `add`/`edit`, dependents are re-hashed and re-checked: their errors follow the output as JSON lines with the dependent's `name` and `hash` (with `--json`, in `dependents_diagnostics`); fix those next. `ov run` needs a no-parameter `main` (`--entry f` to pick another) returning `Unit` or `Result[Unit, E]`; `Err(e)` prints `{"kind":"error",...}` to stderr, exit 1. `ov run main -- a b` gives the program `args() == ["a", "b"]`. `ov test` runs `test` blocks in order, prints `ok <name>`, stops at the first failing block (exit 3), then fuzzes contracted functions.

| Command | Use |
|---|---|
| `ov check [file.ov... \| name]` | typecheck, effects, exhaustiveness; exit 1 if any error |
| `ov build <file.ov... \| name> [-o out]` | native executable |
| `ov run <file.ov... \| name> [-- args]` | build and execute (stdin, stdout, exit code forwarded) |
| `ov test [file.ov... \| name]` | test blocks, then fuzzing |
| `ov fmt <file.ov> [--write]` | canonical formatting |
| `ov add <file.ov>...` | typecheck and store definitions, bind names, print `name hash` |
| `ov view <name> [--raw]` `ov edit <name> [--file f]` `ov rm <name>` `ov history <name>` `ov diff <a> <b>` | show (`--raw`: as written, with comments) / replace / unbind / past hashes / diff |
| `ov ls [--kind k] [--sort name]` | bound names and hashes, in the order first added (`--sort name`: alphabetical) |
| `ov export [file]` | every bound definition in `ov ls` order as one `.ov` file (no comments), to stdout or `file` |
| `ov ctx <name>` | source of `name` plus signatures and types of what it calls (`--depth 1` adds bodies) |
| `ov deps <name>` / `ov dependents <name>` | transitive dependencies / callers |

Diagnostics have `code`, `severity`, `name`, `span` (`line col end_line end_col`), `message`, and when relevant `expected`, `actual`, `fixes` (`rank`, `description`, `patch`). A `patch` is often exact text to apply: the signature line for E0105, `let mut x` for E0112, `None => todo(),` arms for E0104. E0100 unknown name, E0101 type mismatch, E0107 cannot infer, E0109 prelude or duplicate name. W-codes are warnings (unused variable: rename to `_x`).

## Common mistakes

1. `1 + 2.5` or `len(xs) * 0.5`: E0101. Convert with `to_float(n)`; Float literals need a `.`.
2. Calling `println` in a fn declared without `!io`: E0105. Add `!io` to its signature and to its callers.
3. `return;`: parse error E0002. Write `return ();`.
4. `let y = if c { 1 };`: `if` without `else` is Unit. Add `else { 0 }`.
5. `xs[i]` when `i` may be out of range traps `bounds`. Use `get(xs, i)` and match `Some`/`None`.
6. `let xs = [];` alone: E0107. Annotate: `let xs: List[Int] = [];`.
7. `while true { ... return x; }` as the last statement still needs a tail expression (`0`). `assert_eq(max_of([]), None)` is E0107: annotate one side. `test`, `type`, `break`, `continue` are keywords, not names. `for (i, x) in enumerate(xs)` works.
8. `fn solve(N: Int)` or `fn Strongest_Extension(...)`: E0002. Capitalized names are types and constructors; values and functions are `snake_case` (`n`, `strongest_extension`; the fix `patch` has it).
9. Missing match arm: E0104; paste the fix `patch`. Also: assigning to a non-`mut` binding is E0112; `parse_int(s)?` in a fn returning `Int` is E0113.

<!-- token estimate: 3.5k by characters (chars / 3.7); code-heavy, so trust this over the word rule. -->
