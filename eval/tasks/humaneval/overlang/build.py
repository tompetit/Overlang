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


def sol(id_, body, extra="", rename=None):
    SOLUTIONS[id_] = (rename, body.strip("\n"), extra.strip("\n"))


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
    if isinstance(node, ast.UnaryOp) and isinstance(node.op, ast.USub):
        inner = lit(node.operand, t)
        return "-" + inner
    if isinstance(t, ast.Name):
        if t.id == "float":
            return floatlit(node.value)
        if t.id == "int":
            assert isinstance(node.value, int) and not isinstance(node.value, bool), ast.dump(node)
            return str(node.value)
        if t.id == "str":
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
    """Docstring -> `//` lines; doctest examples are rewritten as `f(x) == y` in Overlang syntax."""
    lines = ast.get_docstring(fn, clean=True).splitlines()
    out, i = [], 0
    while i < len(lines):
        line = lines[i]
        if line.startswith(">>> "):
            call = ast.parse(line[4:], mode="eval").body
            res = []
            while i + 1 < len(lines) and lines[i + 1].strip() and not lines[i + 1].startswith(">>>"):
                i += 1
                res.append(lines[i])
            text = f"{name}({', '.join(lit(a, p) for a, (_, p) in zip(call.args, args))})"
            if res:
                text += " == " + lit(ast.parse(" ".join(res), mode="eval").body, ret)
            out.append("//   " + text)
        else:
            out.append(("// " + line).rstrip())
        i += 1
    return out


def translate(task, rename):
    py = ast.parse(task["prompt"])
    fn = next(n for n in py.body if isinstance(n, ast.FunctionDef))
    name = rename or fn.name
    args = [(a.arg, a.annotation) for a in fn.args.args]
    ret = fn.returns
    sig = f"fn {name}(" + ", ".join(f"{n}: {ty(t)}" for n, t in args) + f") -> {ty(ret)}"
    doc = doc_comment(fn, name, args, ret)
    if any(">>>" in l for l in ast.get_docstring(fn, clean=True).splitlines()):
        # separate prose from examples with a header line
        k = next(i for i, l in enumerate(doc) if l.startswith("//   "))
        doc.insert(k, "// Examples:")
    prompt = "\n".join(doc) + f"\n{sig} {{\n  todo()\n}}\n"
    header = "\n".join(doc) + f"\n{sig} {{\n"

    rty = ty(ret)
    check = next(n for n in ast.parse(task["tests"]).body if isinstance(n, ast.FunctionDef) and n.name == "check")
    blocks = []
    for k, stmt in enumerate(check.body, 1):
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
    rename, body, extra = SOLUTIONS[task_id]
    prompt, header, visible, tests, name = translate(py, rename)
    solution = header + "\n".join("  " + l if l.strip() else l for l in body.splitlines()) + "\n}\n"
    if extra:
        solution += "\n" + extra + "\n"
    task = {"id": task_id, "benchmark": "humaneval", "lang": "overlang", "name": py["name"],
            "entry_point": name, "prompt": prompt, "tests": tests, "visible_test": visible, "solution": solution}
    if rename:
        task["renamed_from"] = py["entry_point"]
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


if __name__ == "__main__":
    main()
