"""Harness tests. Run: python3 -I -m unittest discover eval/tests
Everything is written under eval/runs/_test/, never /tmp."""
import contextlib
import io
import json
import shutil
import sys
import unittest
from pathlib import Path
from unittest import mock

EVAL = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(EVAL))

import adapters  # noqa: E402
import fetch_tasks  # noqa: E402
import report  # noqa: E402
import run  # noqa: E402
from adapters import overlang, python as py, rust  # noqa: E402

try:
    overlang.ov_bin()
    HAVE_OV = True
except FileNotFoundError:
    HAVE_OV = False

SCRATCH = EVAL / "runs" / "_test"

PY_ORIG = '''from typing import List


def add(a: int, b: int) -> int:
    """ Add two numbers.
    >>> add(1, 2)
    3
    """
    ### Canonical solution below ###
    return a + b

### Unit tests below ###
def check(candidate):
    assert candidate(1, 2) == 3
    assert candidate(
        0, 0) == 0

def test_check():
    check(add)
'''

RS_ROW = {
    "name": "HumanEval_0_add", "language": "rs",
    "prompt": "/// Add two numbers.\n/// >>> add(1, 2)\n/// 3\nfn add(a: isize, b: isize) -> isize {\n",
    "tests": "}\n\nfn main() {\n    let candidate = add;\n    assert_eq!(candidate(1, 2), 3);\n"
             "    assert_eq!(candidate(0, 0), 0);\n}\n",
    "stop_tokens": ["\n}"],
}


OV_TASK = {
    "id": "HumanEval_0_add", "benchmark": "humaneval", "lang": "overlang", "entry_point": "add",
    "prompt": "// Add two numbers.\nfn add(a: Int, b: Int) -> Int {\n  todo()\n}\n",
    "visible_test": "test check_1 {\n  assert_eq(add(1, 2), 3);\n}\n",
    "tests": "test check_1 {\n  assert_eq(add(1, 2), 3);\n}\n\ntest check_2 {\n  assert_eq(add(0, 0), 0);\n}\n",
    "solution": "// Add two numbers.\nfn add(a: Int, b: Int) -> Int {\n  a + b\n}\n",
}


def fake_http_get(calls):
    def get(url, retries=5):
        calls.append(url)
        if "datasets-server" in url:
            return json.dumps({"num_rows_total": 1, "rows": [{"row_idx": 0, "row": RS_ROW}]})
        return PY_ORIG
    return get


class Base(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        shutil.rmtree(SCRATCH, ignore_errors=True)
        SCRATCH.mkdir(parents=True)
        cls.tasks = SCRATCH / "tasks"
        cls.calls = []
        with mock.patch.object(fetch_tasks, "http_get", fake_http_get(cls.calls)), \
                mock.patch.object(sys, "argv", ["fetch_tasks.py", "--out", str(cls.tasks), "--benchmarks", "humaneval"]), \
                contextlib.redirect_stdout(io.StringIO()):
            fetch_tasks.main()
        cls.py_task = json.loads((cls.tasks / "humaneval/python/HumanEval_0_add.json").read_text())
        cls.rs_task = json.loads((cls.tasks / "humaneval/rust/HumanEval_0_add.json").read_text())
        (cls.tasks / "humaneval/overlang").mkdir()
        (cls.tasks / "humaneval/overlang/HumanEval_0_add.json").write_text(json.dumps(OV_TASK))

    @classmethod
    def tearDownClass(cls):
        shutil.rmtree(SCRATCH, ignore_errors=True)


class TestFetch(Base):
    def test_task_files_and_index(self):
        t = self.py_task
        self.assertEqual((t["id"], t["lang"], t["entry_point"]), ("HumanEval_0_add", "python", "add"))
        self.assertTrue(t["prompt"].rstrip().endswith('"""'))
        self.assertIn("def check(candidate)", t["tests"])
        self.assertIn("return a + b", t["solution"])
        self.assertEqual(self.rs_task["entry_point"], "add")
        self.assertEqual(self.rs_task["raw"], RS_ROW)
        index = json.loads((self.tasks / "index.json").read_text())
        self.assertEqual(index["humaneval"]["shared"], ["HumanEval_0_add"])

    def test_second_fetch_skips_existing(self):
        calls = []
        with mock.patch.object(fetch_tasks, "http_get", fake_http_get(calls)), \
                mock.patch.object(sys, "argv", ["f", "--out", str(self.tasks), "--benchmarks", "humaneval"]), \
                contextlib.redirect_stdout(io.StringIO()):
            fetch_tasks.main()
        self.assertEqual(calls, [])  # rows page cached, task files present


class TestAdapters(Base):
    def test_python_workspace(self):
        d = SCRATCH / "pyws" / "work"
        py.workspace(self.py_task, d)
        self.assertEqual(sorted(p.name for p in d.iterdir()), ["CLAUDE.md", "solution.py", "test_visible.py"])
        self.assertIn("def add", (d / "solution.py").read_text())
        self.assertNotIn("return a + b", (d / "solution.py").read_text())
        visible = (d / "test_visible.py").read_text()
        self.assertIn("assert candidate(1, 2) == 3", visible)
        self.assertNotIn("0, 0", visible)  # only the first assertion is visible
        self.assertIn("python3 test_visible.py", (d / "CLAUDE.md").read_text())
        self.assertTrue((adapters.hidden_dir(d) / "hidden_test.py").exists())

    def test_python_grade(self):
        d = SCRATCH / "pygrade" / "work"
        py.workspace(self.py_task, d)
        self.assertFalse(py.grade(d)["passed"])  # stub
        (d / "solution.py").write_text(py.reference(self.py_task))
        self.assertTrue(py.grade(d)["passed"])
        (d / "solution.py").write_text(self.py_task["prompt"] + "    return a - b\n")
        self.assertFalse(py.grade(d)["passed"])
        (d / "solution.py").write_text(self.py_task["prompt"] + "    while True: pass\n")
        self.assertTrue(py.grade(d, timeout=2)["output"].endswith("[timeout]"))

    def test_rust_workspace(self):
        d = SCRATCH / "rsws" / "work"
        rust.workspace(self.rs_task, d)
        self.assertEqual(sorted(p.name for p in d.iterdir()), ["CLAUDE.md", "solution.rs", "test_visible.rs"])
        self.assertIn("todo!()", (d / "solution.rs").read_text())
        visible = (d / "test_visible.rs").read_text()
        self.assertIn("assert_eq!(candidate(1, 2), 3);", visible)
        self.assertNotIn("(0, 0)", visible)
        hidden = (adapters.hidden_dir(d) / "tests.rs").read_text()
        self.assertTrue(hidden.startswith("fn main()"))  # leading `}` stripped

    def test_rust_grade(self):
        d = SCRATCH / "rsgrade" / "work"
        rust.workspace(self.rs_task, d)
        self.assertFalse(rust.grade(d)["passed"])  # todo!() panics
        prompt = self.rs_task["prompt"]
        (d / "solution.rs").write_text(prompt + "    a + b\n}\n")
        self.assertTrue(rust.grade(d)["passed"], rust.grade(d)["output"])
        (d / "solution.rs").write_text(prompt + "    a - b\n}\n")
        self.assertFalse(rust.grade(d)["passed"])
        (d / "solution.rs").write_text(prompt + "    a +\n}\n")  # does not compile
        self.assertFalse(rust.grade(d)["passed"])

    def test_rust_visible_command_compiles_and_runs(self):
        d = SCRATCH / "rsvis" / "work"
        rust.workspace(self.rs_task, d)
        (d / "solution.rs").write_text(self.rs_task["prompt"] + "    a + b\n}\n")
        rc, out, _ = adapters.run(["sh", "-c", rust.test_command()], d, 60)
        self.assertEqual(rc, 0, out)
        self.assertIn("visible test passed", out)

    @unittest.skipUnless(HAVE_OV, "ov binary not built")
    def test_overlang_workspace(self):
        d = SCRATCH / "ovws" / "work"
        overlang.workspace(OV_TASK, d)
        self.assertEqual(sorted(p.name for p in d.iterdir()),
                         ["CLAUDE.md", "ov", "run_tests.sh", "solution.ov", "test_visible.ov"])
        self.assertIn("todo()", (d / "solution.ov").read_text())
        visible = (d / "test_visible.ov").read_text()
        self.assertIn("add(1, 2)", visible)
        self.assertNotIn("add(0, 0)", visible)  # only the first assertion is visible
        md = (d / "CLAUDE.md").read_text()
        self.assertIn("# Overlang card", md)
        self.assertIn("./run_tests.sh", md)
        self.assertNotIn("<!--", md)
        self.assertEqual(overlang.test_command(), "./run_tests.sh")
        self.assertIn("add(0, 0)", (adapters.hidden_dir(d) / "tests.ov").read_text())

    @unittest.skipUnless(HAVE_OV, "ov binary not built")
    def test_overlang_grade(self):
        d = SCRATCH / "ovgrade" / "work"
        overlang.workspace(OV_TASK, d)
        self.assertFalse(overlang.grade(d)["passed"])  # todo() traps
        (d / "solution.ov").write_text(overlang.reference(OV_TASK))
        g = overlang.grade(d)
        self.assertTrue(g["passed"], g["output"])
        self.assertIn("ok check_2", g["output"])
        (d / "solution.ov").write_text(OV_TASK["prompt"].replace("todo()", "a - b"))
        self.assertFalse(overlang.grade(d)["passed"])  # wrong answer
        (d / "solution.ov").write_text(OV_TASK["prompt"].replace("todo()", "a +"))
        self.assertFalse(overlang.grade(d)["passed"])  # does not compile
        (d / "solution.ov").write_text(OV_TASK["prompt"].replace("todo()", "while true { }\n  0"))
        self.assertTrue(overlang.grade(d, timeout=10)["output"].endswith("[timeout]"))

    @unittest.skipUnless(HAVE_OV, "ov binary not built")
    def test_overlang_visible_command(self):
        d = SCRATCH / "ovvis" / "work"
        overlang.workspace(OV_TASK, d)
        rc, out, _ = adapters.run(["sh", "-c", overlang.test_command()], d, 60)
        self.assertEqual(rc, 3, out)  # stub traps with kind todo
        self.assertIn('"kind": "todo"', out)
        (d / "solution.ov").write_text(overlang.reference(OV_TASK))
        rc, out, _ = adapters.run(["sh", "-c", overlang.test_command()], d, 60)
        self.assertEqual(rc, 0, out)
        self.assertIn("ok check_1", out)
        self.assertNotIn(".ovbuild", [p.name for p in d.iterdir()])  # build products stay in .ovtest/

    def test_overlang_task_files(self):
        for path in sorted((EVAL / "tasks/humaneval/overlang").glob("HumanEval_*.json")):
            t = json.loads(path.read_text())
            for key in ("id", "lang", "prompt", "tests", "visible_test", "solution", "entry_point"):
                self.assertIn(key, t, path.name)
            self.assertEqual((t["id"], t["lang"]), (path.stem, "overlang"))
            self.assertIn("todo()", t["prompt"], path.name)
            self.assertNotIn("todo()", t["solution"], path.name)
            self.assertEqual(t["visible_test"].count("test check_"), 1, path.name)


STREAM = "\n".join(json.dumps(e) for e in [
    {"type": "system", "subtype": "init"},
    {"type": "assistant", "message": {"content": [
        {"type": "tool_use", "id": "t1", "name": "Bash", "input": {"command": "python3 test_visible.py"}}]}},
    {"type": "user", "message": {"content": [
        {"type": "tool_result", "tool_use_id": "t1", "is_error": True, "content": "Exit code 1\nAssertionError"}]}},
    {"type": "assistant", "message": {"content": [
        {"type": "tool_use", "id": "t2", "name": "Bash", "input": {"command": "ls"}}]}},
    {"type": "user", "message": {"content": [
        {"type": "tool_result", "tool_use_id": "t2", "is_error": True, "content": "Exit code 2"}]}},
    {"type": "assistant", "message": {"content": [
        {"type": "tool_use", "id": "t3", "name": "Bash", "input": {"command": "cd . && python3 test_visible.py"}}]}},
    {"type": "user", "message": {"content": [
        {"type": "tool_result", "tool_use_id": "t3", "content": "visible test passed"}]}},
    {"type": "result", "is_error": False, "num_turns": 4, "duration_ms": 9000, "total_cost_usd": 0.05,
     "usage": {"input_tokens": 10, "output_tokens": 200, "cache_creation_input_tokens": 3000,
               "cache_read_input_tokens": 8000}},
]) + "\nnot json at all\n"


class TestRun(Base):
    def test_parse_transcript(self):
        r = run.parse_transcript(STREAM, py.TEST_MARKERS)
        self.assertEqual((r["failed_rounds"], r["test_rounds"], r["num_turns"]), (1, 2, 4))
        self.assertEqual((r["input_tokens"], r["output_tokens"]), (10, 200))
        self.assertEqual((r["cache_creation_input_tokens"], r["cache_read_input_tokens"]), (3000, 8000))
        self.assertIsNone(r["agent_error"])

    def test_parse_auth_failure(self):
        line = json.dumps({"type": "result", "is_error": True, "num_turns": 1, "total_cost_usd": 0,
                           "usage": {}, "result": "Failed to authenticate: OAuth session expired"})
        self.assertIn("Failed to authenticate", run.parse_transcript(line, ("x",))["agent_error"])

    def run_cli(self, lang, name, *extra):
        out = SCRATCH / f"{name}.jsonl"
        argv = ["run.py", "--benchmark", "humaneval", "--lang", lang, "--agent", "mock", "--runs", "2",
                "--tasks", "0:1", "--model", "m", "--out", str(out), "--tasks-dir", str(self.tasks), *extra]
        with mock.patch.object(sys, "argv", argv), contextlib.redirect_stdout(io.StringIO()):
            run.main()
        return [json.loads(line) for line in out.read_text().splitlines()]

    def test_mock_end_to_end_python(self):
        lines = self.run_cli("python", "mock_py", "--mock-solve")
        self.assertEqual(len(lines), 2)
        r = lines[0]
        for key in ("benchmark", "lang", "task", "run", "model", "passed", "num_turns", "total_cost_usd",
                    "input_tokens", "output_tokens", "cache_creation_input_tokens", "cache_read_input_tokens",
                    "duration_ms", "failed_rounds"):
            self.assertIn(key, r)
        self.assertTrue(all(x["passed"] for x in lines))
        self.assertTrue((SCRATCH / "mock_py/HumanEval_0_add/1/work/solution.py").exists())
        # rerun resumes: nothing appended
        self.assertEqual(len(self.run_cli("python", "mock_py", "--mock-solve")), 2)

    def test_mock_stub_fails_rust(self):
        lines = self.run_cli("rust", "mock_rs")
        self.assertEqual([x["passed"] for x in lines], [False, False])

    @unittest.skipUnless(HAVE_OV, "ov binary not built")
    def test_mock_end_to_end_overlang(self):
        lines = self.run_cli("overlang", "mock_ov", "--mock-solve")
        self.assertEqual([x["passed"] for x in lines], [True, True])
        self.assertEqual((lines[0]["lang"], lines[0]["task"]), ("overlang", "HumanEval_0_add"))
        self.assertTrue((SCRATCH / "mock_ov/HumanEval_0_add/1/work/solution.ov").exists())
        self.assertEqual(len(self.run_cli("overlang", "mock_ov", "--mock-solve")), 2)  # resumes
        stubs = self.run_cli("overlang", "mock_ov_stub")
        self.assertEqual([x["passed"] for x in stubs], [False, False])


def rec(lang, passed, tok, cost, turns, failed, model="m"):
    return {"benchmark": "humaneval", "lang": lang, "model": model, "passed": passed,
            "input_tokens": tok, "output_tokens": tok * 2, "cache_creation_input_tokens": 0,
            "cache_read_input_tokens": tok * 7, "total_cost_usd": cost, "duration_ms": 1000,
            "num_turns": turns, "failed_rounds": failed, "agent_error": None}


class TestReport(Base):
    def fixture(self):
        path = SCRATCH / "fixture.jsonl"
        rows = [rec("python", True, 100, 0.1, 3, 0), rec("python", True, 200, 0.2, 5, 1),
                rec("python", False, 300, 0.3, 7, 2),
                rec("rust", True, 200, 0.2, 4, 1), rec("rust", False, 400, 0.4, 8, 3),
                rec("rust", False, 600, 0.6, 12, 5)]
        bad = rec("rust", False, 0, 0, 1, 0)
        bad["agent_error"] = "Failed to authenticate"
        path.write_text("\n".join(json.dumps(r) for r in rows + [bad]) + "\n")
        return path

    def test_aggregate(self):
        rows, excluded = report.load([self.fixture()])
        self.assertEqual(excluded, 1)
        s = report.summarise(rows)[("humaneval", "python", "m")]
        self.assertEqual(s["n"], 3)
        self.assertAlmostEqual(s["pass_rate"], 2 / 3)
        self.assertEqual(s["input_tokens"], (200, 200))
        self.assertEqual(s["total_tokens"], (200 * 10, 200 * 10))
        self.assertAlmostEqual(s["cache_hit_rate"][0], 0.875)
        self.assertEqual(s["num_turns"][0], 5)
        self.assertEqual(s["failed_rounds"][0], 1)
        self.assertAlmostEqual(s["total_cost_usd"][1], 0.2)

    def test_compare_ratio(self):
        out = io.StringIO()
        with mock.patch.object(sys, "argv", ["report.py", str(self.fixture()), "--compare", "python", "rust"]), \
                contextlib.redirect_stdout(out):
            report.main()
        text = out.getvalue()
        line = next(x for x in text.splitlines() if x.strip().startswith("input_tokens"))
        self.assertTrue(line.split()[-1] == "2.00", line)  # median 400 vs 200
        self.assertIn("-33.3 points", text)
        self.assertIn("1 runs excluded", text)


if __name__ == "__main__":
    unittest.main()
