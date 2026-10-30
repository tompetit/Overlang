"""Overlang adapter.

Tasks come from tasks/<benchmark>/overlang/<id>.json (hand-translated, see
tasks/humaneval/overlang/NOTES.md). Overlang has no modules, so "the solution"
and "the tests" are separate files that `ov` loads together as one program.

Agent sees: solution.ov (doc comment + signature + `todo()` body), test_visible.ov
(one `test` block, plus a test-only helper if the task needs one), ov (wrapper
for the compiler binary), run_tests.sh, CLAUDE.md (language card + rules).
Hidden: hidden/tests.ov, the full test blocks. Grading runs
`ov test solution.ov tests.ov` in hidden/ (build products land there, not in work/)
and passes iff the exit code is 0 and every test block printed `ok <name>`.
"""
import os
import re
import shutil
from pathlib import Path

import adapters

LANG = "overlang"
TASK_LANG = "overlang"
SOLUTION = "solution.ov"
VISIBLE = "test_visible.ov"
TEST_MARKERS = ("run_tests.sh",)
ROOT = Path(__file__).resolve().parents[2]  # repo root

CLAUDE_MD = """# Task

{card}

---

# Rules for this workspace

Implement the function stub in `{solution}` (replace the `todo()` body; you may
add helper functions). Edit that file only; do not touch any other file. Do not
add a `main` function or `test` blocks to it. Overlang has no modules and no
imports: the tests are loaded together with `{solution}` as one program.

Run the tests with:

    {command}

To typecheck without running: `./ov check {solution} {visible}`. In this workspace
`ov` is the local `./ov` wrapper and works on files: `check`, `run`, `test` and `fmt`
take file names. There is no store in this workspace, so the store commands in the
Toolchain table (`add`, `view`, `edit`, `ctx`, `deps`, `export`, ...) have nothing to work on.

Stop as soon as the tests pass. The visible test is a single smoke check; the
grader runs more cases, so read the doc comment and handle the general case.
"""

RUN_TESTS = """#!/bin/sh
# Build and run solution.ov together with the visible test. Output goes to .ovtest/.
mkdir -p .ovtest && cd .ovtest && exec ../ov test ../{solution} ../{visible}
"""


def ov_bin():
    """The compiler binary: $OV_BIN, else target/release/ov of this checkout, else PATH."""
    for p in (os.environ.get("OV_BIN"), ROOT / "target/release/ov", shutil.which("ov")):
        if p and Path(p).is_file():
            return str(Path(p).resolve())
    raise FileNotFoundError("ov binary not found: cargo build --release, or set OV_BIN")


def test_command():
    return "./run_tests.sh"


def card():
    text = (ROOT / "docs/CARD.md").read_text()
    return re.sub(r"<!--.*?-->", "", text, flags=re.S).strip()


def workspace(task, dir):
    work, hidden = Path(dir), adapters.hidden_dir(dir)
    work.mkdir(parents=True, exist_ok=True)
    hidden.mkdir(parents=True, exist_ok=True)
    (work / SOLUTION).write_text(task["prompt"].rstrip("\n") + "\n")
    (work / VISIBLE).write_text(task["visible_test"].rstrip("\n") + "\n")
    (work / "ov").write_text(f'#!/bin/sh\nexec "{ov_bin()}" "$@"\n')
    (work / "run_tests.sh").write_text(RUN_TESTS.format(solution=SOLUTION, visible=VISIBLE))
    for name in ("ov", "run_tests.sh"):
        (work / name).chmod(0o755)
    (work / "CLAUDE.md").write_text(CLAUDE_MD.format(
        card=card(), solution=SOLUTION, visible=VISIBLE, command=test_command()))
    (hidden / "tests.ov").write_text(task["tests"].rstrip("\n") + "\n")


def grade(dir, timeout=120):
    work, hidden = Path(dir).resolve(), adapters.hidden_dir(dir).resolve()
    rc, out, timed_out = adapters.run([ov_bin(), "test", str(work / SOLUTION), "tests.ov"], hidden, timeout)
    names = re.findall(r"^test (\w+)", (hidden / "tests.ov").read_text(), re.M)
    ok = set(re.findall(r"^ok (\w+)$", out, re.M))
    if rc == 0 and not timed_out and not ok.issuperset(names):
        rc = 1  # exit 0 but a block did not report ok (e.g. the test file was empty)
    return adapters.result(rc, out, timed_out)


def reference(task):
    return task.get("solution")
