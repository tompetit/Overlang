"""Overlang adapter: STUB. The compiler (`ov`) is not ready.

What it will do, with the same interface as python.py and rust.py:

  workspace(task, dir)  write solution.ov containing the task's function stub
                        (declare/doc-comment form from the language card, body
                        a stub the checker accepts), a visible fixture-style test
                        file with one assertion (same format as tests/programs/*.ov
                        plus a .expect file), and a CLAUDE.md with the language
                        card and the rules: edit solution.ov only, run
                        `ov check solution.ov` then `ov test`, stop when green.
                        Tasks come from the Python task files (TASK_LANG), with
                        argument values translated to Overlang literals.
  test_command()        "ov test test_visible.ov" (and `ov check solution.ov`).
  grade(dir)            append the hidden assertions as a fixture-style program
                        that imports solution.ov, `ov run` it, pass iff exit
                        status is 0 and stdout matches the .expect file.
  reference(task)       None until a hand-written corpus exists.
"""

LANG = "overlang"
TASK_LANG = "python"
SOLUTION = "solution.ov"
TEST_MARKERS = ("ov check", "ov test")


def test_command():
    raise NotImplementedError("compiler not ready")


def workspace(task, dir):
    raise NotImplementedError("compiler not ready")


def grade(dir, timeout=120):
    raise NotImplementedError("compiler not ready")


def reference(task):
    raise NotImplementedError("compiler not ready")
