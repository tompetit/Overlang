"""Rust adapter (plain rustc, no cargo project).

Agent sees: solution.rs (doc comments + signature + `todo!()` body, no main),
test_visible.rs (include!s solution.rs and runs the first assertion), CLAUDE.md.
Hidden: grading concatenates solution.rs with the dataset's `fn main()` of
assertions into hidden/main.rs, builds it with rustc and runs it.
"""
import re
from pathlib import Path

import adapters

LANG = "rust"
TASK_LANG = "rust"
SOLUTION = "solution.rs"
VISIBLE = "test_visible.rs"
TEST_MARKERS = ("test_visible",)
RUSTC = ["rustc", "--edition", "2021", "-A", "warnings"]

CLAUDE_MD = """# Task

Implement the function stub in `{solution}` (replace the `todo!()` body). Edit
that file only; do not touch any other file. Do not add a `main` function and
do not add inner attributes (`#![...]`). Use the Rust standard library only.

Run the tests with:

    {command}

Stop as soon as they pass. The visible test is a single smoke check; the grader
runs more cases, so read the doc comment and handle the general case.
"""


def test_command():
    return f"{' '.join(RUSTC)} -o test_visible {VISIBLE} && ./test_visible"


def split_tests(tests):
    """(main-function text, first assertion text or None). The dataset's tests
    start with a `}` that closes the function body of the prompt; drop it."""
    main = tests.lstrip()
    if main.startswith("}"):
        main = main[1:].lstrip("\n")
    m = re.search(r"^[ \t]*(assert(?:_eq|_ne)?!\()", main, re.M)
    if not m:
        return main, None
    # scan to the matching close paren, skipping "string literals"
    depth, i, n = 0, m.end() - 1, len(main)
    while i < n:
        c = main[i]
        if c == '"':
            i += 1
            while i < n and main[i] != '"':
                i += 2 if main[i] == "\\" else 1
        elif c == "(":
            depth += 1
        elif c == ")":
            depth -= 1
            if depth == 0:
                return main, main[m.start():i + 1].strip() + ";"
        i += 1
    return main, None


def workspace(task, dir):
    work, hidden = Path(dir), adapters.hidden_dir(dir)
    work.mkdir(parents=True, exist_ok=True)
    hidden.mkdir(parents=True, exist_ok=True)
    (work / SOLUTION).write_text(task["prompt"].rstrip("\n") + "\n    todo!()\n}\n")
    main, assertion = split_tests(task["tests"])
    body = f"    let candidate = {task['entry_point']};\n    {assertion or ''}\n"
    (work / VISIBLE).write_text(f'include!("{SOLUTION}");\n\nfn main() {{\n{body}    println!("visible test passed");\n}}\n')
    (work / "CLAUDE.md").write_text(CLAUDE_MD.format(solution=SOLUTION, command=test_command()))
    (hidden / "tests.rs").write_text(main)


def grade(dir, timeout=120):
    work, hidden = Path(dir), adapters.hidden_dir(dir)
    (hidden / "main.rs").write_text((work / SOLUTION).read_text() + "\n" + (hidden / "tests.rs").read_text())
    rc, out, timed_out = adapters.run(RUSTC + ["-o", "hidden_bin", "main.rs"], hidden, timeout)
    if rc != 0 or timed_out:
        return adapters.result(rc, out, timed_out)
    return adapters.result(*adapters.run(["./hidden_bin"], hidden, timeout))


def reference(task):
    return None  # the dataset ships no Rust reference solutions
