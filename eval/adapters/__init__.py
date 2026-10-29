"""Language adapters. Each module in this package exposes:

    LANG            language name (matches the module name)
    TASK_LANG       which task files (eval/tasks/<benchmark>/<TASK_LANG>/) to load
    SOLUTION        file name the agent must edit, inside the workspace
    TEST_MARKERS    substrings that identify a test-command invocation in a transcript
    test_command()  shell command the agent runs (also written to CLAUDE.md)
    workspace(task, dir)    write the agent-visible files into `dir`; write hidden
                            tests to hidden_dir(dir)
    grade(dir, timeout)     run the hidden tests -> {"passed": bool, "output": str}
    reference(task)         full reference solution file text, or None

The workspace layout is <run>/work (what the agent sees) and <run>/hidden
(hidden tests, outside the agent's working directory).
"""
import importlib
import os
import signal
import subprocess
from pathlib import Path

TAIL = 4000  # keep this much of command output


def get(lang):
    return importlib.import_module(f"adapters.{lang}")


def hidden_dir(workdir):
    return Path(workdir).parent / "hidden"


def run(cmd, cwd, timeout, env=None):
    """Run a command, killing its whole process group on timeout.
    Returns (returncode, combined output tail, timed_out)."""
    p = subprocess.Popen(cmd, cwd=cwd, env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                         text=True, errors="replace", start_new_session=True)
    try:
        out, _ = p.communicate(timeout=timeout)
        return p.returncode, out, False
    except subprocess.TimeoutExpired:
        os.killpg(p.pid, signal.SIGKILL)
        out, _ = p.communicate()
        return -1, out, True


def result(rc, out, timed_out):
    out = out[-TAIL:]
    if timed_out:
        out += "\n[timeout]"
    return {"passed": rc == 0 and not timed_out, "output": out}
