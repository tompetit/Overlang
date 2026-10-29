"""Python adapter.

Agent sees: solution.py (prompt + `pass`), test_visible.py (first assertion of
the dataset's check()), CLAUDE.md.
Hidden: hidden/hidden_test.py, run with `python3 -I`.
"""
import ast
from pathlib import Path

import adapters

LANG = "python"
TASK_LANG = "python"
SOLUTION = "solution.py"
VISIBLE = "test_visible.py"
TEST_MARKERS = ("test_visible",)

CLAUDE_MD = """# Task

Implement the function stub in `{solution}`. Edit that file only; do not touch
any other file. Use the Python standard library only.

Run the tests with:

    {command}

Stop as soon as they pass. The visible test is a single smoke check; the grader
runs more cases, so read the docstring and handle the general case.
"""


def test_command():
    return f"python3 {VISIBLE}"


def first_assert(tests):
    """Source of the first assert statement inside check(), or None."""
    for node in ast.parse(tests).body:
        if isinstance(node, ast.FunctionDef) and node.name == "check":
            for stmt in node.body:
                if isinstance(stmt, ast.Assert):
                    return ast.get_source_segment(tests, stmt)
    return None


def workspace(task, dir):
    work, hidden = Path(dir), adapters.hidden_dir(dir)
    work.mkdir(parents=True, exist_ok=True)
    hidden.mkdir(parents=True, exist_ok=True)
    name = task["entry_point"]
    (work / SOLUTION).write_text(task["prompt"].rstrip("\n") + "\n    pass\n")
    assertion = first_assert(task["tests"])
    body = f"    {assertion}\n" if assertion else "    pass\n"
    (work / VISIBLE).write_text(
        f"from solution import {name}\n\n\ndef check(candidate):\n{body}\n\n"
        f"check({name})\nprint('visible test passed')\n")
    (work / "CLAUDE.md").write_text(CLAUDE_MD.format(solution=SOLUTION, command=test_command()))
    (hidden / "hidden_test.py").write_text(
        f"import sys\nsys.path.insert(0, {str(work.resolve())!r})\n"
        f"from solution import {name}\n\n{task['tests']}\n\n"
        f"check({name})\nprint('hidden tests passed')\n")


def grade(dir, timeout=120):
    hidden = adapters.hidden_dir(dir)
    return adapters.result(*adapters.run(["python3", "-I", "hidden_test.py"], hidden, timeout))


def reference(task):
    return task.get("solution")
