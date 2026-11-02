"""Benchmark B adapter: maintenance tasks on the todo app, in Python or Overlang. The same
adapter with app="benchc" runs Benchmark C, the same kind of tasks on the 10x larger tracker app
(eval/benchc/, examples/tracker{,_py}).

    adapters.benchb.for_lang("python" | "overlang") -> adapter with the usual interface.

Tasks live in eval/benchb/tasks/<id>/ (task.md, check.sh or check_<lang>.sh, optional
setup/<lang>/ files that replace the app's files, e.g. to plant a bug). Reference
solutions are in eval/benchb/solutions/<id>/<lang>/ (full replacement files; for
Overlang also an optional rm.txt of names to unbind).

Python workspace: main.py, test_todo.py, CLAUDE.md. The agent edits files.

Overlang workspace: .ovstore/ already populated from the app (so the agent starts from
the store), an `ov` wrapper, CLAUDE.md (language card + store workflow). There is no
main.ov: the only way to read code is `ov ctx` / `ov view`. Grading exports the store
(`ov export`) to hidden/export/main.ov and runs the check on
that. Hidden stuff (seed source, export, scratch dirs of the check) is in hidden/.

Grading runs the task's check script: `check.sh <app dir> <lang>`, exit 0 = pass.
"""
import json
import os
import re
import shutil
import subprocess
from pathlib import Path

import adapters
from adapters import overlang as ovmod

EVAL = Path(__file__).resolve().parents[1]
ROOT = EVAL.parent
BENCH = EVAL / "benchb"
TASKS = BENCH / "tasks"
SOLUTIONS = BENCH / "solutions"

APP_OVERVIEW = """The app is a small HTTP todo list backed by SQLite (file `todo.db` in the directory the
server runs in). Routes: `GET /` (HTML page), `GET /todos.txt` (one `id [x] title` line per todo),
`POST /add` (form field `title`, 400 if blank), `POST /toggle` and `POST /delete` (form field `id`),
anything else 404. Mutating routes answer with the re-rendered page. Functions: `db_init`,
`parse_rows`, `list_todos`, `add_todo`, `toggle_todo`, `delete_todo`, `id_form`, `render_todo`,
`render_page`, `render_text`, `respond_page`, `bad_request`, `route`, `port_from`, `serve`, `main`.
Do not edit anything outside the app's own files."""

PY_MD = """# Task workspace: todo app (Python)

{overview}

Files: {py_files}. Python standard library only.

Make the requested change by editing these files. Keep every existing behavior the task does
not change, and update or add unit tests where that is natural. Run the unit tests with:

    {command}

To try the app: `python3 main.py {py_run}` (listens on 127.0.0.1), then use curl. Kill any server
you start. Stop when the
change is done and the tests pass; a hidden end-to-end check grades the result.
"""

OV_MD = """# Task workspace: todo app (Overlang)

{card}

---

# Rules for this workspace

{overview}

The code lives in the store (`.ovstore/`), not in a file. There is no `main.ov`; do not look for
one and do not create one. Use the store commands through the local `./ov` wrapper (it is the
compiler binary; run it from this directory):

- `./ov ls` lists bound names. `./ov ctx <name>` prints a definition plus the signatures and types
  it needs (`--depth 1` adds bodies of callees); `./ov view <name>` prints one definition.
- `./ov edit <name> --file f.ov` (or text on stdin) replaces one existing definition; the name in
  the text must match. Dependents are re-hashed and re-checked: if one now has errors the definition
  is still stored, the errors are printed after the `name hash` lines and the exit code is 2 (fix the
  dependents with further edits).
- `./ov add f.ov` stores new definitions from a file (use it for new functions, types and tests;
  the file may hold several). `./ov rm <name>` unbinds a name.
- `./ov check` typechecks everything, `./ov test` runs the `test` blocks and fuzzes contracts,
  `./ov build main` builds `./main`, `./ov run main -- 8080` runs it (port after `--`).
- Do not pass `--json` as the card says: `ov` rejects it, diagnostics are JSON already.
{shims}

Keep every existing behavior the task does not change, and update or add `test` blocks where that
is natural. Run the tests with:

    {command}

To try the app: `./ov build main && ./main {ov_run}`, then use curl. Kill any
server you start. Stop when the change is done and the tests pass; a hidden end-to-end check
grades the result (it exports the store, builds it and drives the app over HTTP).
"""

TODO_SHIMS = """- `foreign c` declarations for more of the C shims can be added with `./ov add`. Declared in the
  app: `sqlite_open/close/error/exec/exec_1s/exec_1i/query`, `http_*`, `url_decode`,
  `form_value`, `html_escape`. Also available in the shim but not declared: `ov_sqlite_exec_2s`
  (db, sql, p1: Str, p2: Str), `ov_sqlite_query_1s`, `ov_sqlite_query_1i`, `ov_http_query`,
  `ov_http_header`. Declaration form:
  `foreign c "ov_sqlite.h" link "sqlite3" fn sqlite_exec_2s(db: Int, sql: Str, p1: Str, p2: Str) -> Int !fs = "ov_sqlite_exec_2s"`."""

TRACKER_OVERVIEW = """The app is "Tracker", an HTTP issue tracker over SQLite: users, projects (key like `WEB`), issues
with per-project numbers, status open/closed, priority 1-4, optional assignee and labels, comments, and an
activity log per issue; list pages with filters, LIKE search and pagination (20 per page), a `.txt` endpoint
per list, JSON per issue, CSV per project; 400/404/422 errors with text bodies. It takes `[db_path] [port]`
on the command line. The code is in four layers, db, models, render and routes; private helpers are
prefixed `db_`, `m_`, `r_`, `rt_`. Do not edit anything outside the app's own files."""

TRACKER_SHIMS = """- `foreign c` declarations for more of the C shims can be added with `./ov add`; the db layer shows the
  declaration form and which `ov_sqlite_*` and `ov_http_*` functions exist."""

# One benchmark per app: Benchmark B is the todo app, Benchmark C the tracker (10x larger).
APPS = {
    "benchb": dict(
        tasks=BENCH / "tasks", solutions=BENCH / "solutions",
        dirs={"python": ROOT / "examples" / "todo_py", "overlang": ROOT / "examples" / "todo"},
        files={"python": ("main.py", "test_todo.py"), "overlang": ("main.ov",)},
        overview=APP_OVERVIEW, shims=TODO_SHIMS, py_files="`main.py` (the app), `test_todo.py` (unit tests)",
        py_run="8080", ov_run="8080"),
    "benchc": dict(
        tasks=EVAL / "benchc" / "tasks", solutions=EVAL / "benchc" / "solutions",
        dirs={"python": ROOT / "examples" / "tracker_py", "overlang": ROOT / "examples" / "tracker"},
        files={"python": ("db.py", "models.py", "render.py", "main.py", "test_tracker.py"),
               "overlang": ("src/01_db.ov", "src/02_models.ov", "src/03_render.ov", "src/04_routes.ov")},
        overview=TRACKER_OVERVIEW, shims=TRACKER_SHIMS,
        py_files="`db.py`, `models.py`, `render.py`, `main.py` (the app, in that layer order), `test_tracker.py`",
        py_run="tracker.db 8080", ov_run="tracker.db 8080"),
}


def list_tasks(app=None):
    app = app or APPS["benchb"]
    return sorted(p.name for p in app["tasks"].iterdir() if (p / "task.md").is_file())


def load_tasks(spec, app=None):
    """Tasks in sorted id order; `spec` is a start:stop slice."""
    app = app or APPS["benchb"]
    ids = list_tasks(app)
    if re.search(r"[A-Za-z]", spec):  # comma-separated task ids
        picked = spec.split(",")
        missing = [i for i in picked if i not in ids]
        if missing:
            raise FileNotFoundError(f"unknown benchb task(s): {missing}")
    else:
        start, _, stop = spec.partition(":")
        picked = ids[int(start or 0):int(stop) if stop else None]
    out = []
    for i in picked:
        out.append({"id": i, "benchmark": "benchb", "statement": (app["tasks"] / i / "task.md").read_text().strip()})
    return out


def check_script(task_id, lang, app=None):
    d = (app or APPS["benchb"])["tasks"] / task_id
    for name in ("check.sh", f"check_{lang}.sh"):
        if (d / name).is_file():
            return d / name
    raise FileNotFoundError(f"no check script for task {task_id} ({lang})")


def app_source(task_id, lang, app=None):
    """{file name: text} for the app as the task starts: base files plus the task's setup overrides.
    Keys are base names (the tracker's Overlang files live in src/), in layer order."""
    app = app or APPS["benchb"]
    files = {Path(n).name: (app["dirs"][lang] / n).read_text() for n in app["files"][lang]}
    setup = app["tasks"] / task_id / "setup" / lang
    if setup.is_dir():
        for p in setup.iterdir():
            files[p.name] = p.read_text()
    return files


def solution_files(task_id, lang, app=None):
    d = (app or APPS["benchb"])["solutions"] / task_id / lang
    return {p.name: p.read_text() for p in d.iterdir() if p.name != "rm.txt"} if d.is_dir() else {}


class Adapter:
    TASK_LANG = None
    TEST_MARKERS = None
    SOLUTION = None

    def __init__(self, lang, app="benchb"):
        self.app = APPS[app]
        # "overlang-ctx": same workspace, but `ov view/export/history/diff` are disabled, so the
        # only way to read code is `ov ctx` (tests whether the computed context is sufficient)
        self.ctx_only = lang.endswith("-ctx")
        lang = lang.removesuffix("-ctx")
        self.lang = self.LANG = lang
        self.TASK_LANG = lang
        self.SOLUTION = "main.py" if lang == "python" else "the store"
        self.TEST_MARKERS = ("unittest",) if lang == "python" else ("ov test", "ov check")

    # ---- interface used by run.py ----
    def test_command(self):
        return "python3 -I -m unittest" if self.lang == "python" else "./ov test"

    def instruction(self, task):
        return (f"Read CLAUDE.md, then make this change to the todo app:\n\n{task['statement']}\n\n"
                f"Run `{self.test_command()}` before you stop.")

    def workspace(self, task, dir):
        work, hidden = Path(dir), adapters.hidden_dir(dir)
        work.mkdir(parents=True, exist_ok=True)
        hidden.mkdir(parents=True, exist_ok=True)
        files = app_source(task["id"], self.lang, self.app)
        a = self.app
        if self.lang == "python":
            for n, text in files.items():
                (work / n).write_text(text)
            (work / "CLAUDE.md").write_text(PY_MD.format(overview=a["overview"], command=self.test_command(),
                                                         py_files=a["py_files"], py_run=a["py_run"]))
            return
        seed = hidden / "seed"
        seed.mkdir(exist_ok=True)
        for n, text in files.items():  # in layer order: each add checks against what is stored
            (seed / n).write_text(text)
        guard = ('case "$1" in view|export|history|diff) echo \'{"error":"disabled in this workspace: '
                 'read code with ./ov ctx <name> [--depth 1]"}\' >&2; exit 1;; esac\n' if self.ctx_only else "")
        (work / "ov").write_text(f'#!/bin/sh\n{guard}exec "{ovmod.ov_bin()}" "$@"\n')
        (work / "ov").chmod(0o755)
        for n in files:
            self._ov(work, "add", str((seed / n).resolve()))  # ov runs with cwd=work
        shutil.rmtree(seed)  # the store must be the only copy of the code the agent can reach
        md = OV_MD.format(card=ovmod.card(), overview=a["overview"], command=self.test_command(),
                          shims=a["shims"], ov_run=a["ov_run"])
        if self.ctx_only:
            md = md.replace("; `./ov view <name>` prints one definition.",
                            ". In this workspace `./ov view`, `./ov export`, `./ov history` and `./ov diff` are "
                            "disabled: `./ov ctx` is the only way to read code, so ask it for exactly the "
                            "names you need.")
        (work / "CLAUDE.md").write_text(md)

    def grade(self, dir, timeout=120):
        work, hidden = Path(dir).resolve(), adapters.hidden_dir(dir).resolve()
        tid = work.parent.parent.name
        app = work
        if self.lang == "overlang":
            app = hidden / "export"
            app.mkdir(exist_ok=True)
            try:
                (app / "main.ov").write_text(self.export_store(work))
            except RuntimeError as e:
                return {"passed": False, "output": f"store export failed: {e}"}
        scratch = hidden / "scratch"
        env = dict(os.environ, BB_SCRATCH=str(scratch), OV_BIN=ovmod.ov_bin())
        rc, out, timed_out = adapters.run(["bash", str(check_script(tid, self.lang, self.app)), str(app), self.lang],
                                          hidden, timeout, env)
        return adapters.result(rc, out, timed_out)

    # ---- mock agent ----
    def apply_reference(self, task, dir):
        """Put the reference solution into the workspace (what a perfect agent would leave)."""
        work = Path(dir)
        files = solution_files(task["id"], self.lang, self.app)
        if not files:
            raise FileNotFoundError(f"no reference solution for {task['id']} ({self.lang})")
        if self.lang == "python":
            for n, text in files.items():
                (work / n).write_text(text)
            return
        stage = adapters.hidden_dir(work) / "solution"
        stage.mkdir(exist_ok=True)
        rm = self.app["solutions"] / task["id"] / "overlang" / "rm.txt"
        for n in sorted(files):  # layer order by file name
            (stage / n).write_text(files[n])
            self._ov(work, "add", str((stage / n).resolve()))  # ov runs with cwd=work
        for name in rm.read_text().split() if rm.is_file() else []:
            self._ov(work, "rm", name)

    # ---- overlang helpers ----
    def _ov(self, work, *args):
        """Run `ov` in the workspace and return stdout. Exit 2 from add/edit means "stored, but a
        rebound dependent has errors" (a valid state mid-edit); its diagnostics are on stdout, so
        they are part of the returned text."""
        p = subprocess.run([ovmod.ov_bin(), *args], cwd=work, capture_output=True, text=True)
        if p.returncode not in ((0, 2) if args[0] in ("add", "edit") else (0,)):
            raise RuntimeError(f"ov {' '.join(args)} failed: {p.stdout[-800:]} {p.stderr[-800:]}")
        return p.stdout

    def export_store(self, work):
        """All bound definitions as one source file, in `ov ls` order."""
        return self._ov(work, "export")


def for_lang(lang, app="benchb"):
    return Adapter(lang, app)


# ---- transcript metrics ----
READ_CMD = re.compile(r"(^|[\s;&|(])(cat|head|tail|less|grep|rg|ls|find|wc|nl|awk|sed\s+-n|\./ov\s+(ctx|view|ls|deps|dependents|history|diff)|ov\s+(ctx|view|ls|deps|dependents|history|diff))\b")
READ_TOOLS = {"Read", "Grep", "Glob", "NotebookRead"}


def is_read_call(name, inp):
    if name in READ_TOOLS:
        return True
    return name == "Bash" and bool(READ_CMD.search((inp or {}).get("command", "")))


def read_tokens(text):
    """Sum of fresh input tokens (input_tokens + cache_creation_input_tokens) of every assistant
    turn that directly follows a turn containing a read-type tool call, from a stream-json
    transcript. Fresh tokens are what the read result adds to the context; cache reads are
    excluded because they re-count the whole conversation on every turn. A turn is one API
    message (events sharing message.id are merged). Also returns the number of read calls."""
    turns, seen = [], {}
    for line in text.splitlines():
        try:
            ev = json.loads(line)
        except ValueError:
            continue
        if not isinstance(ev, dict) or ev.get("type") != "assistant":
            continue
        msg = ev.get("message") or {}
        mid = msg.get("id") or f"anon{len(turns)}"
        if mid not in seen:
            seen[mid] = {"usage": msg.get("usage") or {}, "read": False}
            turns.append(seen[mid])
        for b in msg.get("content") or []:
            if isinstance(b, dict) and b.get("type") == "tool_use" and is_read_call(b.get("name"), b.get("input")):
                seen[mid]["read"] = True
                seen[mid]["calls"] = seen[mid].get("calls", 0) + 1
    total = 0
    for prev, cur in zip(turns, turns[1:]):
        if prev["read"]:
            u = cur["usage"]
            total += u.get("input_tokens", 0) + u.get("cache_creation_input_tokens", 0)
    return {"read_tokens": total, "read_calls": sum(t.get("calls", 0) for t in turns)}
