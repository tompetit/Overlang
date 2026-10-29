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
| Control | `if c { a } else { b }` (expression, branches agree) `while c { }` `for x in list { }` `return e;` (value required: `return ();`) |
| match | `match e { P => e, P if g => e, }` commas between arms |
| Patterns | `_` `x` `1` `"s"` `true` `Ctor(p, q)` `Ctor` `P { x: p, .. }` `(p, q)` `[]` `[p, q]` `[p, ..rest]` `[..init, p]` `p \| q` (same bindings) |
| Tuples | `(1, "a")`, `t.0`, `t.1` |
| Lists | `[1, 2]`; `xs[i]` traps (`bounds`); `get(xs, i)` gives `Option`; `m[k]` traps, `map_get` is safe |
| Call | `f(a, b)` equals `a.f(b)` unless `a` has a field `f` |
| Operators (low to high) | `\|\|`, `&&`, `== != < <= > >=` (no chaining), `++` (Str or List), `+ -`, `* / %`, unary `- !`, postfix `? () .f [i]` |
| `?` | on `Result[T, E]` gives `T` or returns `Err(e)`; enclosing fn must return `Result[_, E]`. On `Option` returns `None`; fn must return `Option[_]` |

Statements `if`/`match`/block need no `;`. A block ending in `;` has type Unit. `==` is structural on everything except functions. `< <= > >=` only on Int, Float, Str. No implicit conversion: `to_float`, `to_int`. `/` truncates toward zero, `%` takes the dividend's sign; use `div_floor`, `mod_floor` for floor semantics. Division by zero traps. Names: values `snake_case`, types and constructors `Capitalized`, effects lowercase after `!`. Flat namespace, no modules, definitions in any order, prelude names cannot be redefined.

## Effects

A call is legal only if the callee's effects are a subset of the caller's declared effects. Pure means no labels. Labels: `io` (print, println, read_line, read_all, args), `fs`, `net`, `rand` (random_int), `time` (now_millis), `exit` (exit). Any identifier is allowed as your own label (`!db`). List several as `!io !net`. Local mutation (`let mut`, `while`, `for`) is not an effect. A lambda's effects are inferred from its body.

Higher-order functions take one effect parameter `!e` in the same brackets as type parameters and use it on the function argument and the result: `fn map[T, U, !e](xs: List[T], f: fn(T) -> U !e) -> List[U] !e`. The caller pays for the effects of the lambda passed in.

Error you will see: `E0105` with `expected: ["!io"]` and a fix whose `patch` is the full signature line with the label added, e.g. `fn greet(name: Str) -> Unit !io`. Apply it, then re-check callers (they need the label too, up to `main`).

## Contracts and traps

- `requires c` / `ensures c`: pure Bool expressions, any number. `ensures` may use `result`. No effects inside (E0108).
- `assert(c)`, `assert_eq(a, b)` trap with kind `assert`. `todo()` traps with kind `todo`; use it as a placeholder body.
- `declare fn` typechecks and warns W0001, but executing it traps `unimplemented`. Give it a body (or `todo()`) before running.
- A trap prints one JSON object to stderr and exits with code 3: `{"kind": "requires", "fn": "total", "hash": "a3f9…", "clause": "…", "args": {"o": "…"}}`. Kinds: `requires ensures assert todo unimplemented bounds overflow div_zero foreign_null`. Fix the caller or the contract, not the trap.
- `ov test` fuzzes every function with contracts using generated inputs; a failing input is shrunk and reported.

## Prelude

All pure unless marked. Prelude names are always in scope.

**List**: `len(List[T])->Int` `get(xs,i)->Option[T]` `push(xs,x)->List[T]` (new list) `concat(xs,ys)` `slice(xs,from,to)` `reverse` `take(xs,n)` `drop(xs,n)` `tail` `head->Option[T]` `last->Option[T]` `is_empty->Bool` `map(xs,fn(T)->U !e)->List[U] !e` `filter(xs,fn(T)->Bool !e) !e` `fold(xs,init:A,fn(A,T)->A !e)->A !e` `for_each(xs,fn(T)->Unit !e)->Unit !e` `any(xs,p)` `all(xs,p)` `flat_map(xs,fn(T)->List[U] !e)` `flatten(List[List[T]])` `sum(List[Int])->Int` `sum_float(List[Float])->Float` `range(a,b)->List[Int]` (half-open) `zip(xs,ys)->List[(A,B)]` `enumerate(xs)->List[(Int,T)]` `contains(xs,x)->Bool` `index_of(xs,x)->Option[Int]` `join(List[Str],sep)->Str` `sort(xs)` `min_of(xs)->Option[T]` `max_of(xs)->Option[T]` (sort, min_of, max_of: Int, Float, Str only) `sort_by(xs,key:fn(T)->Int)` `unique(xs)` (Int, Str, Bool only).

**Str**: `str_len(s)->Int` (code points) `chars(s)->List[Str]` `split(s,sep)->List[Str]` `trim` `to_upper` `to_lower` `starts_with(s,p)` `ends_with(s,p)` `str_contains(s,needle)` `replace(s,from,to)` `substr(s,from,to)` `repeat(s,n)` `char_at(s,i)->Option[Str]` `code_point(s)->Option[Int]` `from_code_point(Int)->Str` `parse_int(s)->Option[Int]` `parse_float(s)->Option[Float]` `show[T](x:T)->Str` (any non-function type; Str shows quoted, `3.0` shows `3`; `println` takes Str, so `println(show(n))`).

**Map, Set** (keys Int, Str or Bool only; all immutable, updates return a new value): `map_new()` `map_get(m,k)->Option[V]` `map_set(m,k,v)` `map_remove(m,k)` `map_contains(m,k)` `map_keys(m)` `map_values(m)` `map_len(m)` `map_from_list(List[(K,V)])` `map_to_list(m)`; `set_new()` `set_add(s,x)` `set_remove(s,x)` `set_contains(s,x)` `set_len(s)` `set_to_list(s)` `set_from_list(xs)`.

**Option, Result**: `unwrap_or(o,default)` `is_some` `is_none` `is_ok` `is_err` `ok_or(o:Option[T],e:E)->Result[T,E]` `option_map(o,f)` `result_map(r,f)`.

**Math**: Int: `abs` `min` `max` `pow(base,exp)` (exp >= 0) `div_floor` `mod_floor`. Float: `fabs` `fmin` `fmax` `fpow` `sqrt` `floor` `ceil` `round` (all Float to Float). Convert: `to_float(Int)->Float`, `to_int(Float)->Int` (truncates).

**IO and env**: `print(s)` `println(s)` `read_line()->Option[Str]` `read_all()->Str` `args()->List[Str]` all `!io`; `exit(code)->Unit !exit`; `now_millis()->Int !time`; `random_int(lo,hi)->Int !rand`.

**Testing**: `assert(Bool)` `assert_eq(a,b)` `todo[T]()->T`.

## Toolchain

Always pass `--json`. `ov run <name>` needs a no-parameter fn returning `Unit` or `Result[Unit, E]`; `Err(e)` prints `{"kind":"error",...}` to stderr, exit 1.

| Command | Use |
|---|---|
| `ov check [name]` | typecheck, effects, exhaustiveness; JSON diagnostics, exit 0 if no errors |
| `ov run <name>` | build and execute (stdin, stdout, exit code forwarded) |
| `ov test [name]` | run `test` blocks, then fuzz contracted functions |
| `ov fmt <file.ov>` | canonical formatting |
| `ov add <file.ov>...` | typecheck and store definitions, bind names, print `name hash` |
| `ov view <name>` | print one definition in canonical form |
| `ov edit <name> [--file f]` | replace one definition from file or stdin (its name must match) |
| `ov ctx <name>` | source of `name`, plus signatures and needed types of what it calls; use instead of reading files (`--depth 1` adds bodies) |
| `ov deps <name>` / `ov dependents <name>` | transitive dependencies / callers |

Diagnostics are one JSON object per line: `code`, `severity`, `name`, `span` (`line col end_line end_col`), `message`, and when relevant `expected`, `actual`, `fixes` (`rank`, `description`, `patch`). A `patch` is often exact text to apply: the signature line for E0105, `let mut x` for E0112, `None => todo(),` arms for E0104. Other codes: E0100 unknown name (nearest names), E0101 type mismatch, E0102 argument count, E0103 unknown field or constructor, E0106 unknown type, E0107 cannot infer, E0110 refutable `let`, E0113 `?` in a fn with the wrong return type. W-codes are warnings (unused variable: rename to `_x`).

## Common mistakes

1. `1 + 2.5` or `len(xs) * 0.5`: E0101. Convert with `to_float(n)`; Float literals need a `.`.
2. Calling `println` in a fn declared without `!io`: E0105. Add `!io` to its signature and to its callers.
3. `return;`: parse error E0002. Write `return ();`.
4. `let y = if c { 1 };`: `if` without `else` is Unit. Add `else { 0 }`.
5. `let x = 1 x`: parse error E0002. Every `let` and assignment ends with `;`.
6. `f == g` on functions: E0101. Compare data, not functions.
7. `xs[i]` when `i` may be out of range traps `bounds`. Use `get(xs, i)` and match `Some`/`None`.
8. `xs.0`: E0103; `.0` is for tuples only. Use `head(xs)` or `xs[0]`.
9. `let xs = [];` alone: E0107. Annotate: `let xs: List[Int] = [];`.
10. Missing match arm: E0104; paste the fix `patch`. Also: assigning to a non-`mut` binding is E0112; `declare fn` bodies do not exist, so running one traps `unimplemented`; `parse_int(s)?` inside a fn returning `Int` is E0113 (return `Option` or `match`).

<!-- token estimate: 1,740 words x 1.3 = ~2,260 by the word rule; by characters (11.4k / 3.7) ~3,050. Code-heavy, so trust the larger figure. -->
