"""Benchmark B tests. Run: python3 -I -m unittest discover eval/tests
Everything is written under eval/runs/_test_benchb/, never /tmp. Needs the built compiler for the
Overlang parts (cargo build --release); those tests skip without it."""
import json
import shutil
import subprocess
import sys
import unittest
from pathlib import Path

EVAL = Path(__file__).resolve().parent.parent
ROOT = EVAL.parent
sys.path.insert(0, str(EVAL))
sys.path.insert(0, str(EVAL / "benchb"))

import adapters  # noqa: E402
from adapters import benchb, overlang  # noqa: E402
import validate  # noqa: E402

try:
    OV = overlang.ov_bin()
except FileNotFoundError:
    OV = None

SCRATCH = EVAL / "runs" / "_test_benchb"
TASK = "b04_blank_422"


def setUpModule():
    shutil.rmtree(SCRATCH, ignore_errors=True)
    SCRATCH.mkdir(parents=True)
    validate.OUT = SCRATCH / "validate"


def tearDownModule():
    shutil.rmtree(SCRATCH, ignore_errors=True)


class TestApps(unittest.TestCase):
    def test_python_app_unit_tests(self):
        p = subprocess.run([sys.executable, "-B", "-I", "-m", "unittest"], cwd=ROOT / "examples/todo_py",
                           capture_output=True, text=True)
        self.assertEqual(p.returncode, 0, p.stderr)
        self.assertIn("Ran 3 tests", p.stderr)

    @unittest.skipUnless(OV, "ov not built")
    def test_overlang_app_unit_tests(self):
        work = SCRATCH / "ovtest"
        work.mkdir()
        p = subprocess.run([OV, "test", str(ROOT / "examples/todo/main.ov")], cwd=work, capture_output=True, text=True)
        self.assertEqual(p.returncode, 0, p.stdout + p.stderr)
        for name in ("parse_rows_decodes_rows", "render_page_escapes_titles", "route_unknown_is_404"):
            self.assertIn(f"ok {name}", p.stdout)


class TestAdapter(unittest.TestCase):
    def test_task_inventory(self):
        ids = benchb.list_tasks()
        self.assertEqual(len(ids), 25)
        for tid in ids:
            for lang in ("python", "overlang"):
                self.assertTrue(benchb.check_script(tid, lang).is_file(), (tid, lang))
                self.assertIn("main.py" if lang == "python" else "main.ov", benchb.solution_files(tid, lang), (tid, lang))

    def test_load_tasks_slice_and_ids(self):
        self.assertEqual([t["id"] for t in benchb.load_tasks("0:2")], benchb.list_tasks()[:2])
        self.assertEqual([t["id"] for t in benchb.load_tasks(TASK)], [TASK])
        with self.assertRaises(FileNotFoundError):
            benchb.load_tasks("nope")

    def test_python_workspace(self):
        work = SCRATCH / "pyws" / "work"
        benchb.for_lang("python").workspace({"id": "b19_bug_toggle"}, work)
        self.assertEqual(sorted(p.name for p in work.iterdir()), ["CLAUDE.md", "main.py", "test_todo.py"])
        self.assertIn("update todos set done = 1 where id", (work / "main.py").read_text())  # planted bug
        self.assertIn("python3 -I -m unittest", (work / "CLAUDE.md").read_text())

    @unittest.skipUnless(OV, "ov not built")
    def test_overlang_workspace_is_store_only(self):
        work = SCRATCH / "ovws" / "work"
        ad = benchb.for_lang("overlang")
        ad.workspace({"id": "b19_bug_toggle"}, work)
        self.assertTrue((work / ".ovstore").is_dir())
        self.assertFalse(list(work.glob("*.ov")), "no .ov file in the workspace")
        self.assertIn("ov ctx", (work / "CLAUDE.md").read_text())
        out = subprocess.run([OV, "ctx", "toggle_todo"], cwd=work, capture_output=True, text=True).stdout
        self.assertIn("update todos set done = 1 where id", out)  # setup override reached the store
        self.assertIn("toggle_todo", ad.export_store(work))

    def test_read_tokens(self):
        def turn(mid, tool=None, inp=None, usage=(0, 0)):
            content = [{"type": "tool_use", "id": mid, "name": tool, "input": inp or {}}] if tool else [{"type": "text", "text": "x"}]
            return json.dumps({"type": "assistant", "message": {"id": mid, "content": content,
                               "usage": {"input_tokens": usage[0], "cache_creation_input_tokens": usage[1],
                                         "cache_read_input_tokens": 9999}}})
        lines = [
            turn("m1", "Read", {"file_path": "main.py"}, (1, 10)),
            turn("m2", "Bash", {"command": "./ov ctx route"}, (2, 200)),        # follows a read: counts 2+200
            turn("m3", "Edit", {"file_path": "main.py"}, (3, 30)),               # follows a read: counts 3+30
            turn("m4", "Bash", {"command": "python3 -I -m unittest"}, (4, 40)),  # follows an edit: no
            turn("m5", None, None, (5, 50)),                                    # follows a test run: no
            "not json",
        ]
        self.assertEqual(benchb.read_tokens("\n".join(lines)), {"read_tokens": 2 + 200 + 3 + 30 + 0, "read_calls": 2})
        self.assertEqual(benchb.read_tokens(""), {"read_tokens": 0, "read_calls": 0})


class TestChecks(unittest.TestCase):
    def test_python_check_fails_then_passes(self):
        ok, before, after = validate.validate(TASK, "python")
        self.assertFalse(before["passed"])
        self.assertIn("want status 422", before["output"])
        self.assertTrue(after["passed"], after["output"])

    @unittest.skipUnless(OV, "ov not built")
    def test_overlang_check_fails_then_passes(self):
        ok, before, after = validate.validate(TASK, "overlang")
        self.assertFalse(before["passed"])
        self.assertIn("want status 422", before["output"])
        self.assertTrue(after["passed"], after["output"])

    @unittest.skipUnless(OV, "ov not built")
    def test_overlang_trap_probe_task(self):
        """A task whose check builds a separate probe program and looks at the trap."""
        ok, before, after = validate.validate("b15_add_ensures", "overlang")
        self.assertTrue(ok, (before, after))


class TestMockEndToEnd(unittest.TestCase):
    def run_mock(self, lang, solve):
        out = SCRATCH / f"mock-{lang}-{int(solve)}.jsonl"
        cmd = [sys.executable, str(EVAL / "run.py"), "--benchmark", "benchb", "--lang", lang, "--agent", "mock",
               "--tasks", TASK, "--runs", "1", "--out", str(out)] + (["--mock-solve"] if solve else [])
        p = subprocess.run(cmd, capture_output=True, text=True)
        self.assertEqual(p.returncode, 0, p.stdout + p.stderr)
        lines = out.read_text().splitlines()
        self.assertEqual(len(lines), 1)
        return json.loads(lines[0])

    def test_python_mock(self):
        rec = self.run_mock("python", solve=True)
        self.assertEqual((rec["benchmark"], rec["lang"], rec["task"], rec["passed"]), ("benchb", "python", TASK, True))
        self.assertIn("failed_rounds", rec)
        self.assertFalse(self.run_mock("python", solve=False)["passed"])

    @unittest.skipUnless(OV, "ov not built")
    def test_overlang_mock(self):
        rec = self.run_mock("overlang", solve=True)
        self.assertEqual((rec["benchmark"], rec["lang"], rec["task"], rec["passed"]), ("benchb", "overlang", TASK, True))
        self.assertFalse(self.run_mock("overlang", solve=False)["passed"])


if __name__ == "__main__":
    unittest.main()
