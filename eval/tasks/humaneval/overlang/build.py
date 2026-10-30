#!/usr/bin/env python3
"""Build and validate the Overlang HumanEval tasks from the Python ones.

    python3 -I eval/tasks/humaneval/overlang/build.py [--tasks 0:40]

For each task in eval/tasks/index.json order: take the Python task (signature,
docstring, `check` asserts), translate it mechanically (types, literals, the
doctest examples, the assertions), attach the hand-written solution from
SOLUTIONS below, and write <id>.json only if it validates by running `ov`:

  1. stub workspace: `ov check` passes on solution.ov + test_visible.ov;
  2. solution workspace: the visible test passes through ./run_tests.sh;
  3. grade(): the full hidden tests pass with the solution and fail with the stub.

Tasks whose types cannot be expressed are listed in SKIP with the reason.
Needs the ov binary (cargo build --release, or $OV_BIN).
"""
import argparse
import ast
import json
import re
import shutil
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
EVAL = HERE.parents[2]
sys.path.insert(0, str(EVAL))
import adapters  # noqa: E402
from adapters import overlang  # noqa: E402

SKIP = {}  # id -> reason

# id -> (new function name or None, solution body, extra top-level code after the function)
SOLUTIONS = {}


def sol(id_, body, extra="", rename=None, pren=None):
    """pren: {python parameter name: Overlang name} for names the language cannot take."""
    SOLUTIONS[id_] = (rename, body.strip("\n"), extra.strip("\n"), pren)


# ---------------------------------------------------------------- type / literal translation

def ty(node):
    """Python annotation AST -> Overlang type text."""
    if isinstance(node, ast.Name):
        return {"int": "Int", "float": "Float", "str": "Str", "bool": "Bool"}[node.id]
    if isinstance(node, ast.Subscript):
        head = node.value.id
        args = node.slice.elts if isinstance(node.slice, ast.Tuple) else [node.slice]
        inner = [ty(a) for a in args]
        if head == "List":
            return f"List[{inner[0]}]"
        if head == "Optional":
            return f"Option[{inner[0]}]"
        if head == "Tuple":
            return "(" + ", ".join(inner) + ")"
        if head == "Dict":
            return f"Map[{inner[0]}, {inner[1]}]"
        if head == "Set":
            return f"Set[{inner[0]}]"
    raise ValueError(f"cannot type {ast.dump(node)}")


def strlit(s):
    out = []
    for c in s:
        if c in '\\"':
            out.append("\\" + c)
        elif c == "\n":
            out.append("\\n")
        elif c == "\t":
            out.append("\\t")
        elif c == "\r":
            out.append("\\r")
        elif ord(c) < 32 or ord(c) > 126:
            out.append("\\u{%x}" % ord(c))
        else:
            out.append(c)
    return '"' + "".join(out) + '"'


def floatlit(v):
    s = repr(float(v))
    if "e" in s and "." not in s.split("e")[0]:
        m, e = s.split("e")
        s = f"{m}.0e{e}"
    return s


def lit(node, t):
    """Python literal AST + annotation AST -> Overlang literal text."""
    if isinstance(node, ast.UnaryOp) and isinstance(node.op, ast.USub) and isinstance(t, ast.Name):
        return "-" + lit(node.operand, t)
    if isinstance(t, ast.Name):
        if t.id == "float":
            return floatlit(node.value)
        if t.id == "int":
            assert isinstance(node.value, int) and not isinstance(node.value, bool), ast.dump(node)
            return str(node.value)
        if t.id == "str":
            if not isinstance(node.value, str):
                raise ValueError(f"not a str: {ast.dump(node)}")
            return strlit(node.value)
        if t.id == "bool":
            return "true" if node.value else "false"
    if isinstance(t, ast.Subscript):
        head = t.value.id
        args = t.slice.elts if isinstance(t.slice, ast.Tuple) else [t.slice]
        if head == "List":
            return "[" + ", ".join(lit(e, args[0]) for e in node.elts) + "]"
        if head == "Tuple":
            return "(" + ", ".join(lit(e, a) for e, a in zip(node.elts, args)) + ")"
        if head == "Dict":
            if not node.keys:
                return "map_new()"
            return "map_from_list([" + ", ".join(
                f"({lit(k, args[0])}, {lit(v, args[1])})" for k, v in zip(node.keys, node.values)) + "])"
        if head == "Optional":
            if isinstance(node, ast.Constant) and node.value is None:
                return "None"
            return f"Some({lit(node, args[0])})"
    raise ValueError(f"cannot translate literal {ast.dump(node)} as {ast.dump(t)}")


# ---------------------------------------------------------------- float comparison helpers

HELPERS = {
    "Float": ("tst_close", "fn tst_close(a: Float, b: Float) -> Bool {\n  fabs(a - b) < 1.0e-6\n}\n"),
    "List[Float]": ("tst_close_list",
                    "fn tst_close_list(a: List[Float], b: List[Float]) -> Bool {\n"
                    "  len(a) == len(b) && all(zip(a, b), |p| tst_close(p.0, p.1))\n}\n"),
    "(Float, Float)": ("tst_close_pair",
                       "fn tst_close_pair(a: (Float, Float), b: (Float, Float)) -> Bool {\n"
                       "  tst_close(a.0, b.0) && tst_close(a.1, b.1)\n}\n"),
}


def helper_text(ret):
    if ret not in HELPERS:
        return ""
    base = HELPERS["Float"][1]
    return base if ret == "Float" else base + "\n" + HELPERS[ret][1]


# ---------------------------------------------------------------- one task

def doc_comment(fn, name, args, ret):
    """Docstring -> `//` lines; doctest examples are rewritten as `f(x) == y` in Overlang syntax.

    Returns (lines, index of the first rewritten example or None)."""
    lines = ast.get_docstring(fn, clean=True).splitlines()
    out, i, first = [], 0, None
    while i < len(lines):
        line = lines[i]
        if line.startswith(">>> "):
            call = ast.parse(line[4:], mode="eval").body
            res, want = [], None
            # the result is the shortest run of lines that parses; anything after it is prose
            while i + 1 < len(lines) and lines[i + 1].strip() and not lines[i + 1].startswith(">>>"):
                i += 1
                res.append(lines[i])
                try:
                    want = ast.parse(" ".join(res), mode="eval").body
                    break
                except SyntaxError:
                    pass
            try:
                text = f"{name}({', '.join(lit(a, p) for a, (_, p) in zip(call.args, args))})"
                if want is not None:
                    text += " == " + lit(want, ret)
            except ValueError:  # e.g. a dict with an int key for Dict[str, str]: not typable, drop the example
                i += 1
                continue
            if first is None:
                first = len(out)
            out.append("//   " + text)
        else:
            out.append(("// " + line).rstrip())
        i += 1
    return out, first


def translate(task, rename, pren=None):
    py = ast.parse(task["prompt"])
    fn = next(n for n in py.body if isinstance(n, ast.FunctionDef))
    name = rename or fn.name
    args = [(a.arg, a.annotation) for a in fn.args.args]
    ret = fn.returns
    pren = pren or {}
    sig = f"fn {name}(" + ", ".join(f"{pren.get(n, n)}: {ty(t)}" for n, t in args) + f") -> {ty(ret)}"
    doc, first = doc_comment(fn, name, args, ret)
    if first is not None:
        # separate prose from examples with a header line, unless the docstring has one
        if re.fullmatch(r"// (Examples?|For example|for examble):?", doc[first - 1] if first else ""):
            doc[first - 1] = "// Examples:"
        else:
            doc.insert(first, "// Examples:")
    prompt = "\n".join(doc) + f"\n{sig} {{\n  todo()\n}}\n"
    header = "\n".join(doc) + f"\n{sig} {{\n"

    rty = ty(ret)
    check = next(n for n in ast.parse(task["tests"]).body if isinstance(n, ast.FunctionDef) and n.name == "check")
    blocks = []
    k = 0
    for stmt in check.body:
        if not isinstance(stmt, ast.Assert):  # stray `print` etc.
            continue
        k += 1
        cmp = stmt.test
        assert isinstance(stmt, ast.Assert) and isinstance(cmp, ast.Compare) and isinstance(cmp.ops[0], ast.Eq), ast.dump(stmt)
        call, want = cmp.left, cmp.comparators[0]
        got = f"{name}({', '.join(lit(a, t) for a, (_, t) in zip(call.args, args))})"
        exp = lit(want, ret)
        body = f"assert({HELPERS[rty][0]}({got}, {exp}));" if rty in HELPERS else f"assert_eq({got}, {exp});"
        blocks.append(f"test check_{k} {{\n  {body}\n}}\n")
    helpers = helper_text(rty)
    visible = (helpers + "\n" if helpers else "") + blocks[0]
    tests = (helpers + "\n" if helpers else "") + "\n".join(blocks)
    return prompt, header, visible, tests, name


def make(task_id, tasks_dir, scratch):
    py = json.loads((tasks_dir / "python" / f"{task_id}.json").read_text())
    rename, body, extra, pren = SOLUTIONS[task_id]
    prompt, header, visible, tests, name = translate(py, rename, pren)
    solution = header + "\n".join("  " + l if l.strip() else l for l in body.splitlines()) + "\n}\n"
    if extra:
        solution += "\n" + extra + "\n"
    task = {"id": task_id, "benchmark": "humaneval", "lang": "overlang", "name": py["name"],
            "entry_point": name, "prompt": prompt, "tests": tests, "visible_test": visible, "solution": solution}
    if rename:
        task["renamed_from"] = py["entry_point"]
    if pren:
        task["renamed_params"] = pren
    problems = validate(task, scratch / task_id)
    return task, problems


def validate(task, d):
    shutil.rmtree(d, ignore_errors=True)
    work = d / "work"
    overlang.workspace(task, work)
    problems = []
    ov = overlang.ov_bin()
    rc, out, _ = adapters.run([ov, "check", "solution.ov", "test_visible.ov"], work, 60)
    if rc != 0:
        problems.append(f"stub does not check: {out[-600:]}")
    if overlang.grade(work)["passed"]:
        problems.append("stub passes the hidden tests")
    (work / "solution.ov").write_text(task["solution"])
    rc, out, _ = adapters.run(["./run_tests.sh"], work, 60)
    if rc != 0 or "ok check_1" not in out:
        problems.append(f"visible test fails with the solution: {out[-600:]}")
    g = overlang.grade(work)
    if not g["passed"]:
        problems.append(f"hidden tests fail with the solution: {g['output'][-600:]}")
    return problems


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--tasks", default="0:40")
    args = ap.parse_args()
    tasks_dir = EVAL / "tasks" / "humaneval"
    ids = json.loads((EVAL / "tasks" / "index.json").read_text())["humaneval"]["shared"]
    a, _, b = args.tasks.partition(":")
    ids = ids[int(a or 0):int(b) if b else None]
    scratch = EVAL / "runs" / "_gen"
    good, bad = [], []
    for task_id in ids:
        if task_id in SKIP:
            print(f"skip {task_id}: {SKIP[task_id]}")
            continue
        if task_id not in SOLUTIONS:
            print(f"no solution for {task_id}")
            bad.append(task_id)
            continue
        try:
            task, problems = make(task_id, tasks_dir, scratch)
        except Exception as e:  # translation failure
            task, problems = None, [f"translation: {e!r}"]
        if problems:
            print(f"FAIL {task_id}\n  " + "\n  ".join(problems))
            bad.append(task_id)
        else:
            (HERE / f"{task_id}.json").write_text(json.dumps(task, indent=1, ensure_ascii=False) + "\n")
            good.append(task_id)
            print(f"ok   {task_id}")
    shutil.rmtree(scratch, ignore_errors=True)
    print(f"{len(good)} validated, {len(bad)} failed, {sum(1 for i in ids if i in SKIP)} skipped")
    sys.exit(1 if bad else 0)


# ---------------------------------------------------------------- hand-written solutions

sol("HumanEval_0_has_close_elements", """
let n = len(numbers);
for i in range(0, n) {
  for j in range(i + 1, n) {
    if fabs(numbers[i] - numbers[j]) < threshold {
      return true;
    }
  }
}
false
""")

sol("HumanEval_1_separate_paren_groups", """
let mut out: List[Str] = [];
let mut cur = "";
let mut depth = 0;
for c in chars(paren_string) {
  if c == "(" {
    depth = depth + 1;
    cur = cur ++ c;
  } else if c == ")" {
    depth = depth - 1;
    cur = cur ++ c;
    if depth == 0 {
      out = push(out, cur);
      cur = "";
    }
  }
}
out
""")

sol("HumanEval_2_truncate_number", "number - floor(number)")

sol("HumanEval_3_below_zero", """
let mut bal = 0;
for op in operations {
  bal = bal + op;
  if bal < 0 {
    return true;
  }
}
false
""")

sol("HumanEval_4_mean_absolute_deviation", """
let n = to_float(len(numbers));
let mean = sum_float(numbers) / n;
sum_float(map(numbers, |x| fabs(x - mean))) / n
""")

sol("HumanEval_5_intersperse", """
let mut out: List[Int] = [];
for (i, x) in enumerate(numbers) {
  if i > 0 {
    out = push(out, delimeter);
  }
  out = push(out, x);
}
out
""")

sol("HumanEval_6_parse_nested_parens", """
map(filter(split(paren_string, " "), |g| str_len(g) > 0), |g| max_depth(g))
""", """
fn max_depth(g: Str) -> Int {
  let mut depth = 0;
  let mut best = 0;
  for c in chars(g) {
    if c == "(" {
      depth = depth + 1;
      best = max(best, depth);
    } else if c == ")" {
      depth = depth - 1;
    }
  }
  best
}
""")

sol("HumanEval_7_filter_by_substring", "filter(strings, |s| str_contains(s, substring))")

sol("HumanEval_8_sum_product", "(sum(numbers), fold(numbers, 1, |acc, x| acc * x))")

sol("HumanEval_9_rolling_max", """
let mut out: List[Int] = [];
let mut cur = 0;
for (i, x) in enumerate(numbers) {
  cur = if i == 0 { x } else { max(cur, x) };
  out = push(out, cur);
}
out
""")

sol("HumanEval_10_make_palindrome", """
let n = str_len(string);
let mut i = 0;
while !is_palindrome(substr(string, i, n)) {
  i = i + 1;
}
string ++ join(reverse(chars(substr(string, 0, i))), "")
""", """
fn is_palindrome(s: Str) -> Bool {
  chars(s) == reverse(chars(s))
}
""")

sol("HumanEval_11_string_xor", """
join(map(zip(chars(a), chars(b)), |p| if p.0 == p.1 { "0" } else { "1" }), "")
""")

sol("HumanEval_12_longest", """
let mut best: Option[Str] = None;
for s in strings {
  match best {
    None => { best = Some(s); },
    Some(b) => { if str_len(s) > str_len(b) { best = Some(s); } },
  }
}
best
""")

sol("HumanEval_13_greatest_common_divisor", """
let mut x = abs(a);
let mut y = abs(b);
while y != 0 {
  let t = x % y;
  x = y;
  y = t;
}
x
""")

sol("HumanEval_14_all_prefixes", "map(range(1, str_len(string) + 1), |i| substr(string, 0, i))")

sol("HumanEval_15_string_sequence", 'join(map(range(0, n + 1), |i| show(i)), " ")')

sol("HumanEval_16_count_distinct_characters", "len(unique(chars(to_lower(string))))")

sol("HumanEval_17_parse_music", """
map(filter(split(music_string, " "), |t| str_len(t) > 0), |t| beats(t))
""", """
fn beats(note: Str) -> Int {
  match note {
    "o" => 4,
    "o|" => 2,
    _ => 1,
  }
}
""")

sol("HumanEval_18_how_many_times", """
let n = str_len(string);
let m = str_len(substring);
let mut count = 0;
for i in range(0, n - m + 1) {
  if substr(string, i, i + m) == substring {
    count = count + 1;
  }
}
count
""")

sol("HumanEval_19_sort_numbers", """
let words = filter(split(numbers, " "), |w| str_len(w) > 0);
join(sort_by(words, |w| unwrap_or(index_of(names(), w), 0)), " ")
""", """
fn names() -> List[Str] {
  ["zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine"]
}
""")

sol("HumanEval_20_find_closest_elements", """
let xs = sort(numbers);
let mut best = (xs[0], xs[1]);
for i in range(2, len(xs)) {
  if xs[i] - xs[i - 1] < best.1 - best.0 {
    best = (xs[i - 1], xs[i]);
  }
}
best
""")

sol("HumanEval_21_rescale_to_unit", """
let lo = unwrap_or(min_of(numbers), 0.0);
let hi = unwrap_or(max_of(numbers), 0.0);
map(numbers, |x| (x - lo) / (hi - lo))
""")

sol("HumanEval_23_strlen", "str_len(string)")

sol("HumanEval_24_largest_divisor", """
let mut d = n / 2;
while d > 1 && n % d != 0 {
  d = d - 1;
}
d
""")

sol("HumanEval_25_factorize", """
let mut out: List[Int] = [];
let mut m = n;
let mut p = 2;
while p * p <= m {
  if m % p == 0 {
    out = push(out, p);
    m = m / p;
  } else {
    p = p + 1;
  }
}
if m > 1 { push(out, m) } else { out }
""")

sol("HumanEval_26_remove_duplicates", "filter(numbers, |x| len(filter(numbers, |y| y == x)) == 1)")

sol("HumanEval_27_flip_case", """
join(map(chars(string), |c| if c == to_upper(c) { to_lower(c) } else { to_upper(c) }), "")
""")

sol("HumanEval_28_concatenate", 'join(strings, "")')

sol("HumanEval_29_filter_by_prefix", "filter(strings, |s| starts_with(s, prefix))")

sol("HumanEval_30_get_positive", "filter(l, |x| x > 0)")

sol("HumanEval_31_is_prime", """
if n < 2 {
  return false;
}
let mut i = 2;
while i * i <= n {
  if n % i == 0 {
    return false;
  }
  i = i + 1;
}
true
""")

sol("HumanEval_33_sort_third", """
let vals = sort(map(filter(enumerate(l), |p| p.0 % 3 == 0), |p| p.1));
map(enumerate(l), |p| if p.0 % 3 == 0 { vals[p.0 / 3] } else { p.1 })
""")

sol("HumanEval_34_unique", "sort(unique(l))", rename="unique_sorted")

sol("HumanEval_35_max_element", "unwrap_or(max_of(l), 0)")

sol("HumanEval_36_fizz_buzz", """
let mut count = 0;
for i in range(0, n) {
  if i % 11 == 0 || i % 13 == 0 {
    count = count + len(filter(chars(show(i)), |c| c == "7"));
  }
}
count
""")

sol("HumanEval_37_sort_even", """
let vals = sort(map(filter(enumerate(l), |p| p.0 % 2 == 0), |p| p.1));
map(enumerate(l), |p| if p.0 % 2 == 0 { vals[p.0 / 2] } else { p.1 })
""")

sol("HumanEval_39_prime_fib", """
let mut a = 1;
let mut b = 2;
let mut seen = 0;
while true {
  if prime(b) {
    seen = seen + 1;
    if seen == n {
      return b;
    }
  }
  let t = a + b;
  a = b;
  b = t;
}
0
""", """
fn prime(n: Int) -> Bool {
  if n < 2 {
    return false;
  }
  let mut i = 2;
  while i * i <= n {
    if n % i == 0 {
      return false;
    }
    i = i + 1;
  }
  true
}
""")

sol("HumanEval_40_triples_sum_to_zero", """
let k = len(l);
for i in range(0, k) {
  for j in range(i + 1, k) {
    for m in range(j + 1, k) {
      if l[i] + l[j] + l[m] == 0 {
        return true;
      }
    }
  }
}
false
""")

sol("HumanEval_41_car_race_collision", "n * n")

sol("HumanEval_42_incr_list", "map(l, |x| x + 1)")

sol("HumanEval_43_pairs_sum_to_zero", """
let n = len(l);
for i in range(0, n) {
  for j in range(i + 1, n) {
    if l[i] + l[j] == 0 {
      return true;
    }
  }
}
false
""")

sol("HumanEval_44_change_base", """
let mut out = "";
let mut m = x;
while m > 0 {
  out = show(m % base) ++ out;
  m = m / base;
}
out
""")

sol("HumanEval_45_triangle_area", "to_float(a * h) / 2.0")

sol("HumanEval_46_fib4", """
if n < 2 {
  return 0;
}
let mut a = 0;
let mut b = 0;
let mut c = 2;
let mut d = 0;
for _i in range(3, n) {
  let t = a + b + c + d;
  a = b;
  b = c;
  c = d;
  d = t;
}
if n == 2 { c } else { d }
""")

sol("HumanEval_47_median", """
let s = sort(l);
let n = len(s);
if n % 2 == 1 {
  to_float(s[n / 2])
} else {
  to_float(s[n / 2 - 1] + s[n / 2]) / 2.0
}
""")

sol("HumanEval_48_is_palindrome", "chars(text) == reverse(chars(text))")

sol("HumanEval_49_modp", """
let mut r = 1;
for _i in range(0, n) {
  r = r * 2 % p;
}
r
""")

sol("HumanEval_51_remove_vowels", """
join(filter(chars(text), |c| !contains(["a", "e", "i", "o", "u"], to_lower(c))), "")
""")

sol("HumanEval_52_below_threshold", "all(l, |x| x < t)")

sol("HumanEval_53_add", "x + y")

sol("HumanEval_54_same_chars", "sort(unique(chars(s0))) == sort(unique(chars(s1)))")

sol("HumanEval_55_fib", """
let mut a = 0;
let mut b = 1;
for _i in range(0, n) {
  let t = a + b;
  a = b;
  b = t;
}
a
""")

sol("HumanEval_56_correct_bracketing", """
let mut depth = 0;
for c in chars(brackets) {
  depth = if c == "<" { depth + 1 } else { depth - 1 };
  if depth < 0 {
    return false;
  }
}
depth == 0
""")

sol("HumanEval_57_monotonic", "l == sort(l) || l == reverse(sort(l))")

sol("HumanEval_58_common", "sort(unique(filter(l1, |x| contains(l2, x))))")

sol("HumanEval_59_largest_prime_factor", """
let mut m = n;
let mut p = 2;
let mut best = 1;
while m > 1 {
  if m % p == 0 {
    best = p;
    m = m / p;
  } else {
    p = p + 1;
  }
}
best
""")

sol("HumanEval_60_sum_to_n", "n * (n + 1) / 2")

sol("HumanEval_61_correct_bracketing", """
let mut depth = 0;
for c in chars(brackets) {
  depth = if c == "(" { depth + 1 } else { depth - 1 };
  if depth < 0 {
    return false;
  }
}
depth == 0
""")

sol("HumanEval_62_derivative", "map(tail(enumerate(xs)), |p| p.0 * p.1)")

sol("HumanEval_63_fibfib", """
let mut a = 0;
let mut b = 0;
let mut c = 1;
for _i in range(0, n) {
  let t = a + b + c;
  a = b;
  b = c;
  c = t;
}
a
""")

sol("HumanEval_64_vowels_count", """
let t = to_lower(s);
let base = len(filter(chars(t), |c| contains(["a", "e", "i", "o", "u"], c)));
if ends_with(t, "y") { base + 1 } else { base }
""")

sol("HumanEval_65_circular_shift", """
let d = show(x);
let n = str_len(d);
if shift > n {
  join(reverse(chars(d)), "")
} else {
  substr(d, n - shift, n) ++ substr(d, 0, n - shift)
}
""")

sol("HumanEval_66_digitSum", """
sum(map(filter(chars(s), |c| upper_letter(c)), |c| unwrap_or(code_point(c), 0)))
""", """
fn upper_letter(c: Str) -> Bool {
  to_upper(c) == c && to_lower(c) != c
}
""")

sol("HumanEval_67_fruit_distribution", """
n - sum(map(split(s, " "), |w| unwrap_or(parse_int(w), 0)))
""")

sol("HumanEval_68_pluck", """
let mut best: List[Int] = [];
for (i, x) in enumerate(arr) {
  if x % 2 == 0 && (is_empty(best) || x < best[0]) {
    best = [x, i];
  }
}
best
""")

sol("HumanEval_69_search", """
let mut best = -1;
for x in unique(lst) {
  if x > best && len(filter(lst, |y| y == x)) >= x {
    best = x;
  }
}
best
""")

sol("HumanEval_70_strange_sort_list", """
let s = sort(lst);
let mut out: List[Int] = [];
let mut i = 0;
let mut j = len(s) - 1;
while i <= j {
  out = push(out, s[i]);
  i = i + 1;
  if i <= j {
    out = push(out, s[j]);
    j = j - 1;
  }
}
out
""")

sol("HumanEval_71_triangle_area", """
if a + b <= c || a + c <= b || b + c <= a {
  return -1.0;
}
let s = to_float(a + b + c) / 2.0;
let area = sqrt(s * (s - to_float(a)) * (s - to_float(b)) * (s - to_float(c)));
round(area * 100.0) / 100.0
""")

sol("HumanEval_72_will_it_fly", "q == reverse(q) && sum(q) <= w")

sol("HumanEval_73_smallest_change", """
let n = len(arr);
len(filter(range(0, n / 2), |i| arr[i] != arr[n - 1 - i]))
""")

sol("HumanEval_74_total_match", """
let a = sum(map(lst1, |s| str_len(s)));
let b = sum(map(lst2, |s| str_len(s)));
if a <= b { lst1 } else { lst2 }
""")

sol("HumanEval_75_is_multiply_prime", """
let mut m = a;
let mut p = 2;
let mut count = 0;
while m > 1 {
  if m % p == 0 {
    count = count + 1;
    m = m / p;
  } else {
    p = p + 1;
  }
}
count == 3
""")

sol("HumanEval_76_is_simple_power", """
if n == 1 {
  return x == 1;
}
let mut p = 1;
while p < x {
  p = p * n;
}
p == x
""")

sol("HumanEval_77_iscube", """
let m = abs(a);
let mut i = 0;
while i * i * i < m {
  i = i + 1;
}
i * i * i == m
""")

sol("HumanEval_78_hex_key", """
len(filter(chars(num), |c| contains(["2", "3", "5", "7", "B", "D"], c)))
""")

sol("HumanEval_79_decimal_to_binary", """
let mut bits = "";
let mut m = decimal;
while m > 0 {
  bits = show(m % 2) ++ bits;
  m = m / 2;
}
"db" ++ (if bits == "" { "0" } else { bits }) ++ "db"
""")

sol("HumanEval_80_is_happy", """
let cs = chars(s);
let n = len(cs);
if n < 3 {
  return false;
}
for i in range(0, n - 2) {
  if cs[i] == cs[i + 1] || cs[i] == cs[i + 2] || cs[i + 1] == cs[i + 2] {
    return false;
  }
}
true
""")

sol("HumanEval_81_numerical_letter_grade", "map(grades, |g| letter(g))", """
fn letter(g: Float) -> Str {
  if g == 4.0 { "A+" }
  else if g > 3.7 { "A" }
  else if g > 3.3 { "A-" }
  else if g > 3.0 { "B+" }
  else if g > 2.7 { "B" }
  else if g > 2.3 { "B-" }
  else if g > 2.0 { "C+" }
  else if g > 1.7 { "C" }
  else if g > 1.3 { "C-" }
  else if g > 1.0 { "D+" }
  else if g > 0.7 { "D" }
  else if g > 0.0 { "D-" }
  else { "E" }
}
""")

sol("HumanEval_82_prime_length", """
let n = str_len(string);
if n < 2 {
  return false;
}
let mut i = 2;
while i * i <= n {
  if n % i == 0 {
    return false;
  }
  i = i + 1;
}
true
""")

sol("HumanEval_83_starts_one_ends", "if n == 1 { 1 } else { 18 * pow(10, n - 2) }")

sol("HumanEval_84_solve", """
let mut m = sum(map(chars(show(n)), |c| unwrap_or(parse_int(c), 0)));
if m == 0 {
  return "0";
}
let mut bits = "";
while m > 0 {
  bits = show(m % 2) ++ bits;
  m = m / 2;
}
bits
""", pren={"N": "n"})

sol("HumanEval_85_add", """
sum(map(filter(enumerate(lst), |p| p.0 % 2 == 1 && p.1 % 2 == 0), |p| p.1))
""")

sol("HumanEval_86_anti_shuffle", """
join(map(split(s, " "), |w| join(sort(chars(w)), "")), " ")
""")

sol("HumanEval_87_get_row", """
let mut out: List[(Int, Int)] = [];
for (i, row) in enumerate(lst) {
  for j in reverse(range(0, len(row))) {
    if row[j] == x {
      out = push(out, (i, j));
    }
  }
}
out
""")

sol("HumanEval_88_sort_array", """
if is_empty(array) {
  return [];
}
let s = sort(array);
if (array[0] + array[len(array) - 1]) % 2 == 1 { s } else { reverse(s) }
""")

SKIP["HumanEval_92_any_int"] = (
    "tests tell 3 from 3.0 (any_int(3, 4, 7) is True, any_int(3.0, 4, 7) is False): "
    "the contract is about the runtime type of a number, not its value; no static Overlang type expresses it")

sol("HumanEval_89_encrypt", """
let abc = "abcdefghijklmnopqrstuvwxyz";
join(map(chars(s), |c| match index_of(chars(abc), c) {
  Some(i) => unwrap_or(char_at(abc, (i + 4) % 26), c),
  None => c,
}), "")
""")

sol("HumanEval_90_next_smallest", "get(sort(unique(lst)), 1)")

sol("HumanEval_91_is_bored", """
let sentences = split(replace(replace(s, "?", "."), "!", "."), ".");
len(filter(sentences, |t| starts_with(ltrim(t), "I ")))
""", """
fn ltrim(t: Str) -> Str {
  if starts_with(t, " ") { ltrim(substr(t, 1, str_len(t))) } else { t }
}
""", pren={"S": "s"})

sol("HumanEval_93_encode", """
join(map(chars(message), |c| {
  let w = if c == to_upper(c) { to_lower(c) } else { to_upper(c) };
  if contains(["a", "e", "i", "o", "u", "A", "E", "I", "O", "U"], w) {
    from_code_point(unwrap_or(code_point(w), 0) + 2)
  } else {
    w
  }
}), "")
""")

sol("HumanEval_94_skjkasdkd", """
let primes = filter(lst, |x| is_prime(x));
let best = unwrap_or(max_of(primes), 0);
sum(map(chars(show(best)), |c| unwrap_or(parse_int(c), 0)))
""", """
fn is_prime(n: Int) -> Bool {
  if n < 2 {
    return false;
  }
  let mut i = 2;
  while i * i <= n {
    if n % i == 0 {
      return false;
    }
    i = i + 1;
  }
  true
}
""")

sol("HumanEval_95_check_dict_case", """
let keys = map_keys(dict);
!is_empty(keys) && (all(keys, |k| k == to_lower(k) && k != to_upper(k)) || all(keys, |k| k == to_upper(k) && k != to_lower(k)))
""")

sol("HumanEval_96_count_up_to", "filter(range(2, n), |x| is_prime(x))", """
fn is_prime(n: Int) -> Bool {
  if n < 2 {
    return false;
  }
  let mut i = 2;
  while i * i <= n {
    if n % i == 0 {
      return false;
    }
    i = i + 1;
  }
  true
}
""")

sol("HumanEval_97_multiply", "(abs(a) % 10) * (abs(b) % 10)")

sol("HumanEval_98_count_upper", """
len(filter(enumerate(chars(s)), |p| p.0 % 2 == 0 && contains(["A", "E", "I", "O", "U"], p.1)))
""")

sol("HumanEval_99_closest_integer", """
to_int(round(unwrap_or(parse_float(value), 0.0)))
""")

sol("HumanEval_100_make_a_pile", "map(range(0, n), |i| n + 2 * i)")

sol("HumanEval_101_words_string", """
filter(split(replace(s, ",", " "), " "), |w| str_len(w) > 0)
""")

sol("HumanEval_102_choose_num", """
if x > y {
  -1
} else if y % 2 == 0 {
  y
} else if y - 1 >= x {
  y - 1
} else {
  -1
}
""")

sol("HumanEval_104_unique_digits", """
sort(filter(x, |n| all(chars(show(n)), |c| contains(["1", "3", "5", "7", "9"], c))))
""")

sol("HumanEval_105_by_length", """
let names = ["One", "Two", "Three", "Four", "Five", "Six", "Seven", "Eight", "Nine"];
map(reverse(sort(filter(arr, |x| x >= 1 && x <= 9))), |x| names[x - 1])
""")

sol("HumanEval_106_f", """
map(range(1, n + 1), |i| if i % 2 == 0 { fold(range(1, i + 1), 1, |a, k| a * k) } else { i * (i + 1) / 2 })
""")

sol("HumanEval_107_even_odd_palindrome", """
let mut even = 0;
let mut odd = 0;
for i in range(1, n + 1) {
  let t = chars(show(i));
  if t == reverse(t) {
    if i % 2 == 0 {
      even = even + 1;
    } else {
      odd = odd + 1;
    }
  }
}
(even, odd)
""")

sol("HumanEval_108_count_nums", """
len(filter(arr, |n| signed_digit_sum(n) > 0))
""", """
fn signed_digit_sum(n: Int) -> Int {
  let ds = map(chars(show(abs(n))), |c| unwrap_or(parse_int(c), 0));
  sum(ds) - (if n < 0 { 2 * ds[0] } else { 0 })
}
""")

sol("HumanEval_109_move_one_ball", """
let n = len(arr);
len(filter(range(0, n), |i| arr[i] > arr[(i + 1) % n])) <= 1
""")

sol("HumanEval_110_exchange", """
if len(filter(lst1, |x| x % 2 != 0)) <= len(filter(lst2, |x| x % 2 == 0)) { "YES" } else { "NO" }
""")

sol("HumanEval_111_histogram", """
let words = filter(split(s, " "), |w| str_len(w) > 0);
let none: Map[Str, Int] = map_new();
let counts = fold(words, none, |m, w| map_set(m, w, unwrap_or(map_get(m, w), 0) + 1));
let best = unwrap_or(max_of(map_values(counts)), 0);
map_from_list(filter(map_to_list(counts), |p| p.1 == best))
""", pren={"test": "s"})

sol("HumanEval_112_reverse_delete", """
let t = filter(chars(s), |ch| !contains(chars(c), ch));
(join(t, ""), t == reverse(t))
""")

sol("HumanEval_113_odd_count", """
map(lst, |t| {
  let d = show(len(filter(chars(t), |ch| contains(["1", "3", "5", "7", "9"], ch))));
  "the number of odd elements " ++ d ++ "n the str" ++ d ++ "ng " ++ d ++ " of the " ++ d ++ "nput."
})
""")

sol("HumanEval_114_minSubArraySum", """
let mut best = nums[0];
let mut cur = nums[0];
for x in tail(nums) {
  cur = min(x, cur + x);
  best = min(best, cur);
}
best
""")

sol("HumanEval_115_max_fill", """
sum(map(grid, |row| (sum(row) + capacity - 1) / capacity))
""")

sol("HumanEval_116_sort_array", "sort_by(sort(arr), |x| popcount(abs(x)))", """
fn popcount(n: Int) -> Int {
  if n == 0 { 0 } else { n % 2 + popcount(n / 2) }
}
""")

sol("HumanEval_117_select_words", """
filter(filter(split(s, " "), |w| str_len(w) > 0), |w| len(filter(chars(to_lower(w)), |c| !contains(["a", "e", "i", "o", "u"], c))) == n)
""")

sol("HumanEval_118_get_closest_vowel", """
let cs = chars(word);
let v = ["a", "e", "i", "o", "u", "A", "E", "I", "O", "U"];
let mut i = len(cs) - 2;
while i >= 1 {
  if contains(v, cs[i]) && !contains(v, cs[i + 1]) && !contains(v, cs[i - 1]) {
    return cs[i];
  }
  i = i - 1;
}
""
""")

sol("HumanEval_119_match_parens", """
if balanced(lst[0] ++ lst[1]) || balanced(lst[1] ++ lst[0]) { "Yes" } else { "No" }
""", """
fn balanced(s: Str) -> Bool {
  let mut depth = 0;
  for c in chars(s) {
    depth = if c == "(" { depth + 1 } else { depth - 1 };
    if depth < 0 {
      return false;
    }
  }
  depth == 0
}
""")

sol("HumanEval_120_maximum", "drop(sort(arr), len(arr) - k)")

sol("HumanEval_121_solution", """
sum(map(filter(enumerate(lst), |p| p.0 % 2 == 0 && p.1 % 2 != 0), |p| p.1))
""")

sol("HumanEval_122_add_elements", """
sum(filter(take(arr, k), |x| str_len(show(x)) <= 2))
""")

sol("HumanEval_123_get_odd_collatz", """
let mut out: List[Int] = [];
let mut m = n;
while m != 1 {
  if m % 2 == 1 {
    out = push(out, m);
  }
  m = if m % 2 == 0 { m / 2 } else { 3 * m + 1 };
}
sort(push(out, 1))
""")

sol("HumanEval_124_valid_date", """
let parts = split(date, "-");
if len(parts) != 3 {
  return false;
}
match (parse_int(parts[0]), parse_int(parts[1]), parse_int(parts[2])) {
  (Some(m), Some(d), Some(_y)) => {
    if m < 1 || m > 12 || d < 1 {
      false
    } else if contains([1, 3, 5, 7, 8, 10, 12], m) {
      d <= 31
    } else if contains([4, 6, 9, 11], m) {
      d <= 30
    } else {
      d <= 29
    }
  },
  _ => false,
}
""")

sol("HumanEval_126_is_sorted", """
for x in unique(lst) {
  if len(filter(lst, |y| y == x)) > 2 {
    return false;
  }
}
for i in range(1, len(lst)) {
  if lst[i - 1] > lst[i] {
    return false;
  }
}
true
""")

sol("HumanEval_127_intersection", """
let length = min(interval1.1, interval2.1) - max(interval1.0, interval2.0);
if length > 1 && is_prime(length) { "YES" } else { "NO" }
""", """
fn is_prime(n: Int) -> Bool {
  if n < 2 {
    return false;
  }
  let mut i = 2;
  while i * i <= n {
    if n % i == 0 {
      return false;
    }
    i = i + 1;
  }
  true
}
""")

sol("HumanEval_128_prod_signs", """
if is_empty(arr) {
  return None;
}
let negs = len(filter(arr, |x| x < 0));
let sign = if contains(arr, 0) { 0 } else if negs % 2 == 1 { -1 } else { 1 };
Some(sign * sum(map(arr, |x| abs(x))))
""")

sol("HumanEval_129_minPath", """
let n = len(grid);
let mut low = n * n + 1;
for i in range(0, n) {
  for j in range(0, n) {
    if grid[i][j] == 1 {
      if i > 0 { low = min(low, grid[i - 1][j]); }
      if i < n - 1 { low = min(low, grid[i + 1][j]); }
      if j > 0 { low = min(low, grid[i][j - 1]); }
      if j < n - 1 { low = min(low, grid[i][j + 1]); }
    }
  }
}
map(range(0, k), |i| if i % 2 == 0 { 1 } else { low })
""")

sol("HumanEval_130_tri", """
let mut t: List[Int] = [1, 3];
for i in range(2, n + 1) {
  let v = if i % 2 == 0 { i / 2 + 1 } else { t[i - 1] + t[i - 2] + (i + 3) / 2 };
  t = push(t, v);
}
take(t, n + 1)
""")

sol("HumanEval_131_digits", """
let odd = filter(map(chars(show(n)), |c| unwrap_or(parse_int(c), 0)), |d| d % 2 == 1);
if is_empty(odd) { 0 } else { fold(odd, 1, |a, d| a * d) }
""")

sol("HumanEval_132_is_nested", """
let mut opens: List[Int] = [];
let mut closes: List[Int] = [];
for (i, c) in enumerate(chars(string)) {
  if c == "[" {
    opens = push(opens, i);
  } else {
    closes = push(closes, i);
  }
}
let closes = reverse(closes);
let mut count = 0;
let mut j = 0;
for idx in opens {
  if j < len(closes) && idx < closes[j] {
    count = count + 1;
    j = j + 1;
  }
}
count >= 2
""")

sol("HumanEval_133_sum_squares", """
sum(map(lst, |x| { let c = to_int(ceil(x)); c * c }))
""")

sol("HumanEval_134_check_if_last_char_is_a_letter", """
match last(split(txt, " ")) {
  Some(w) => {
    let cp = unwrap_or(code_point(to_lower(w)), 0);
    str_len(w) == 1 && cp >= 97 && cp <= 122
  },
  None => false,
}
""")

sol("HumanEval_135_can_arrange", """
let mut best = -1;
for i in range(1, len(arr)) {
  if arr[i] < arr[i - 1] {
    best = i;
  }
}
best
""")

sol("HumanEval_136_largest_smallest_integers", """
(max_of(filter(lst, |x| x < 0)), min_of(filter(lst, |x| x > 0)))
""")

sol("HumanEval_138_is_equal_to_sum_even", "n >= 8 && n % 2 == 0")

sol("HumanEval_139_special_factorial", """
let mut f = 1;
let mut r = 1;
for i in range(1, n + 1) {
  f = f * i;
  r = r * f;
}
r
""")

sol("HumanEval_140_fix_spaces", """
let mut out = "";
let mut run = 0;
for c in chars(text) {
  if c == " " {
    run = run + 1;
  } else {
    out = out ++ blanks(run) ++ c;
    run = 0;
  }
}
out ++ blanks(run)
""", """
fn blanks(k: Int) -> Str {
  if k > 2 { "-" } else { repeat("_", k) }
}
""")

sol("HumanEval_141_file_name_check", """
let parts = split(file_name, ".");
if len(parts) != 2 || !contains(["txt", "exe", "dll"], parts[1]) {
  return "No";
}
let cs = chars(parts[0]);
let digits = len(filter(cs, |c| c >= "0" && c <= "9"));
if is_empty(cs) || to_upper(cs[0]) == to_lower(cs[0]) || digits > 3 { "No" } else { "Yes" }
""")

sol("HumanEval_142_sum_squares", """
sum(map(enumerate(lst), |p| if p.0 % 3 == 0 { p.1 * p.1 } else if p.0 % 4 == 0 { p.1 * p.1 * p.1 } else { p.1 }))
""")

sol("HumanEval_143_words_in_sentence", """
join(filter(split(sentence, " "), |w| is_prime(str_len(w))), " ")
""", """
fn is_prime(n: Int) -> Bool {
  if n < 2 {
    return false;
  }
  let mut i = 2;
  while i * i <= n {
    if n % i == 0 {
      return false;
    }
    i = i + 1;
  }
  true
}
""")

sol("HumanEval_144_simplify", """
let a = frac(x);
let b = frac(n);
(a.0 * b.0) % (a.1 * b.1) == 0
""", """
fn frac(t: Str) -> (Int, Int) {
  let p = split(t, "/");
  (unwrap_or(parse_int(p[0]), 0), unwrap_or(parse_int(p[1]), 1))
}
""")

sol("HumanEval_145_order_by_points", "sort_by(nums, |n| signed_digit_sum(n))", """
fn signed_digit_sum(n: Int) -> Int {
  let ds = map(chars(show(abs(n))), |c| unwrap_or(parse_int(c), 0));
  sum(ds) - (if n < 0 { 2 * ds[0] } else { 0 })
}
""")

sol("HumanEval_146_specialFilter", """
len(filter(nums, |n| n > 10 && odd_char(substr(show(n), 0, 1)) && odd_char(substr(show(n), str_len(show(n)) - 1, str_len(show(n))))))
""", """
fn odd_char(c: Str) -> Bool {
  contains(["1", "3", "5", "7", "9"], c)
}
""")

sol("HumanEval_147_get_max_triples", """
let a = map(range(1, n + 1), |i| i * i - i + 1);
let mut count = 0;
for i in range(0, n) {
  for j in range(i + 1, n) {
    for k in range(j + 1, n) {
      if (a[i] + a[j] + a[k]) % 3 == 0 {
        count = count + 1;
      }
    }
  }
}
count
""")

sol("HumanEval_149_sorted_list_sum", """
sort_by(sort(filter(lst, |s| str_len(s) % 2 == 0)), |s| str_len(s))
""")

sol("HumanEval_150_x_or_y", """
if n < 2 {
  return y;
}
let mut i = 2;
while i * i <= n {
  if n % i == 0 {
    return y;
  }
  i = i + 1;
}
x
""")

sol("HumanEval_151_double_the_difference", """
fold(lst, 0, |acc, x| {
  if x > 0.0 && x == floor(x) && to_int(x) % 2 == 1 { acc + to_int(x) * to_int(x) } else { acc }
})
""")

sol("HumanEval_152_compare", "map(zip(game, guess), |p| abs(p.0 - p.1))")

sol("HumanEval_153_Strongest_Extension", """
let mut best = extensions[0];
let mut best_strength = strength(best);
for e in extensions {
  if strength(e) > best_strength {
    best = e;
    best_strength = strength(e);
  }
}
class_name ++ "." ++ best
""", """
fn strength(e: Str) -> Int {
  fold(chars(e), 0, |acc, c| if to_upper(c) == to_lower(c) { acc } else if c == to_upper(c) { acc + 1 } else { acc - 1 })
}
""", rename="strongest_extension")

sol("HumanEval_154_cycpattern_check", """
let n = str_len(b);
any(range(0, n + 1), |i| str_contains(a, substr(b, i, n) ++ substr(b, 0, i)))
""")

sol("HumanEval_155_even_odd_count", """
let ds = map(chars(show(abs(num))), |c| unwrap_or(parse_int(c), 0));
(len(filter(ds, |d| d % 2 == 0)), len(filter(ds, |d| d % 2 == 1)))
""")

sol("HumanEval_156_int_to_mini_roman", """
let values = [1000, 900, 500, 400, 100, 90, 50, 40, 10, 9, 5, 4, 1];
let symbols = ["m", "cm", "d", "cd", "c", "xc", "l", "xl", "x", "ix", "v", "iv", "i"];
let mut n = number;
let mut out = "";
for (i, v) in enumerate(values) {
  while n >= v {
    out = out ++ symbols[i];
    n = n - v;
  }
}
out
""")

sol("HumanEval_157_right_angle_triangle", """
a * a + b * b == c * c || a * a + c * c == b * b || b * b + c * c == a * a
""")

sol("HumanEval_158_find_max", """
let mut best = words[0];
for w in words {
  let u = len(unique(chars(w)));
  let bu = len(unique(chars(best)));
  if u > bu || (u == bu && w < best) {
    best = w;
  }
}
best
""")

sol("HumanEval_159_eat", """
if need <= remaining { [number + need, remaining - need] } else { [number + remaining, 0] }
""")

sol("HumanEval_160_do_algebra", """
let (first, e0) = term(operand, operator, 0);
let mut total = first;
let mut e = e0;
while e < len(operator) {
  let (v, e2) = term(operand, operator, e + 1);
  total = if operator[e] == "+" { total + v } else { total - v };
  e = e2;
}
total
""", """
// value of the `**` chain starting at operand i (right associative), and the index of the operator after it
fn chain(vals: List[Int], ops: List[Str], i: Int) -> (Int, Int) {
  let mut e = i;
  while e < len(ops) && ops[e] == "**" {
    e = e + 1;
  }
  let mut v = vals[e];
  for k in reverse(range(i, e)) {
    v = pow(vals[k], v);
  }
  (v, e)
}

// value of the `*` and `//` chain starting at operand i
fn term(vals: List[Int], ops: List[Str], i: Int) -> (Int, Int) {
  let (first, e0) = chain(vals, ops, i);
  let mut v = first;
  let mut e = e0;
  while e < len(ops) && (ops[e] == "*" || ops[e] == "//") {
    let (r, e2) = chain(vals, ops, e + 1);
    v = if ops[e] == "*" { v * r } else { div_floor(v, r) };
    e = e2;
  }
  (v, e)
}
""")

sol("HumanEval_161_solve", """
if all(chars(s), |c| to_upper(c) == to_lower(c)) {
  join(reverse(chars(s)), "")
} else {
  join(map(chars(s), |c| if c == to_upper(c) { to_lower(c) } else { to_upper(c) }), "")
}
""")

sol("HumanEval_163_generate_integers", """
filter(range(max(2, min(a, b)), min(8, max(a, b)) + 1), |i| i % 2 == 0)
""")

import math as _math

_K = ", ".join(str(int(abs(_math.sin(i + 1)) * 2**32)) for i in range(64))
_S = ", ".join(str(x) for x in ([7, 12, 17, 22] * 4 + [5, 9, 14, 20] * 4 + [4, 11, 16, 23] * 4 + [6, 10, 15, 21] * 4))

sol("HumanEval_162_string_to_md5", """
if text == "" {
  return None;
}
Some(md5_hex(pad(utf8(text))))
""", """
// No bit operators, hex literals or sin in the prelude: 32-bit words are Ints in [0, 2^32), bit operations loop over the bits.
fn bitop(a: Int, b: Int, kind: Int) -> Int {
  let mut x = a;
  let mut y = b;
  let mut p = 1;
  let mut r = 0;
  for _i in range(0, 32) {
    let u = x % 2;
    let v = y % 2;
    let bit = if kind == 0 { u * v } else if kind == 1 { max(u, v) } else { (u + v) % 2 };
    r = r + bit * p;
    x = x / 2;
    y = y / 2;
    p = p * 2;
  }
  r
}

fn band(a: Int, b: Int) -> Int { bitop(a, b, 0) }

fn bor(a: Int, b: Int) -> Int { bitop(a, b, 1) }

fn bxor(a: Int, b: Int) -> Int { bitop(a, b, 2) }

fn bnot(a: Int) -> Int { 4294967295 - a }

fn rotl(x: Int, s: Int) -> Int {
  let hi = pow(2, 32 - s);
  (x % hi) * pow(2, s) + x / hi
}

fn utf8(text: Str) -> List[Int] {
  flat_map(chars(text), |c| {
    let cp = unwrap_or(code_point(c), 0);
    if cp < 128 {
      [cp]
    } else if cp < 2048 {
      [192 + cp / 64, 128 + cp % 64]
    } else if cp < 65536 {
      [224 + cp / 4096, 128 + (cp / 64) % 64, 128 + cp % 64]
    } else {
      [240 + cp / 262144, 128 + (cp / 4096) % 64, 128 + (cp / 64) % 64, 128 + cp % 64]
    }
  })
}

fn pad(bytes: List[Int]) -> List[Int] {
  let n = len(bytes);
  let zeros = (119 - n % 64) % 64;
  let bits = n * 8;
  let mut out = push(bytes, 128);
  for _i in range(0, zeros) {
    out = push(out, 0);
  }
  for i in range(0, 8) {
    out = push(out, (bits / pow(256, i)) % 256);
  }
  out
}

fn md5_k() -> List[Int] {
  [__K__]
}

fn md5_s() -> List[Int] {
  [__S__]
}

fn word(block: List[Int], j: Int) -> Int {
  block[4 * j] + block[4 * j + 1] * 256 + block[4 * j + 2] * 65536 + block[4 * j + 3] * 16777216
}

fn md5_hex(bytes: List[Int]) -> Str {
  let k = md5_k();
  let s = md5_s();
  let mut a0 = 1732584193;
  let mut b0 = 4023233417;
  let mut c0 = 2562383102;
  let mut d0 = 271733878;
  for blk in range(0, len(bytes) / 64) {
    let block = slice(bytes, blk * 64, blk * 64 + 64);
    let mut a = a0;
    let mut b = b0;
    let mut c = c0;
    let mut d = d0;
    for i in range(0, 64) {
      let mut f = 0;
      let mut g = 0;
      if i < 16 {
        f = bor(band(b, c), band(bnot(b), d));
        g = i;
      } else if i < 32 {
        f = bor(band(d, b), band(bnot(d), c));
        g = (5 * i + 1) % 16;
      } else if i < 48 {
        f = bxor(bxor(b, c), d);
        g = (3 * i + 5) % 16;
      } else {
        f = bxor(c, bor(b, bnot(d)));
        g = (7 * i) % 16;
      }
      let t = (f + a + k[i] + word(block, g)) % 4294967296;
      a = d;
      d = c;
      c = b;
      b = (b + rotl(t, s[i])) % 4294967296;
    }
    a0 = (a0 + a) % 4294967296;
    b0 = (b0 + b) % 4294967296;
    c0 = (c0 + c) % 4294967296;
    d0 = (d0 + d) % 4294967296;
  }
  let digits = chars("0123456789abcdef");
  join(map(flat_map([a0, b0, c0, d0], |w| map(range(0, 4), |i| (w / pow(256, i)) % 256)),
    |v| digits[v / 16] ++ digits[v % 16]), "")
}
""".replace("__K__", _K).replace("__S__", _S))


if __name__ == "__main__":
    main()
