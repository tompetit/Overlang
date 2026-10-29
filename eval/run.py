#!/usr/bin/env python3
"""Run a coding agent on benchmark tasks and grade it. One JSON line per run.

    run.py --benchmark humaneval --lang python --model sonnet --tasks 0:5 \\
           --runs 3 --agent claude --out eval/runs/pilot.jsonl

Workspaces: eval/runs/<out-stem>/<task>/<run>/{work,hidden}. The agent runs with
cwd=work; hidden tests live in the sibling hidden/. Runs already present in the
output file are skipped, so an interrupted batch can simply be restarted.
Everything stays under eval/runs/ (the agent's own scratch use of /tmp is out
of our hands, but we never write there).
"""
import argparse
import json
import os
import shutil
import sys
import time
from pathlib import Path

import adapters

HERE = Path(__file__).parent
FIELDS = ("input_tokens", "output_tokens", "cache_creation_input_tokens", "cache_read_input_tokens")


def load_tasks(benchmark, lang, spec, tasks_dir=HERE / "tasks"):
    """Tasks in index order; `spec` is a start:stop slice of the shared task list."""
    shared = json.loads((tasks_dir / "index.json").read_text())[benchmark]["shared"]
    start, _, stop = spec.partition(":")
    ids = shared[int(start or 0):int(stop) if stop else None]
    task_lang = adapters.get(lang).TASK_LANG
    return [json.loads((tasks_dir / benchmark / task_lang / f"{i}.json").read_text()) for i in ids]


def claude_cmd(model, max_turns, instruction, bare):
    # stream-json (needs --verbose) instead of json: the final "result" line has the
    # same cost/usage fields, and the stream also shows every tool call, which is the
    # only way to count failed test rounds.
    cmd = ["claude", "-p", instruction, "--output-format", "stream-json", "--verbose",
           "--model", model, "--max-turns", str(max_turns), "--dangerously-skip-permissions",
           "--setting-sources", "project,local"]  # no user plugins/hooks/settings in the agent
    if bare:  # also skips CLAUDE.md discovery, so pass ours explicitly; needs ANTHROPIC_API_KEY
        cmd += ["--bare", "--append-system-prompt-file", "CLAUDE.md"]
    return cmd


def parse_transcript(text, markers):
    """Pull the final result record and the count of failed test commands out of a
    stream-json transcript (non-JSON lines are ignored)."""
    result, pending, test_rounds, failed = None, {}, 0, 0
    for line in text.splitlines():
        try:
            ev = json.loads(line)
        except ValueError:
            continue
        if not isinstance(ev, dict):
            continue
        if ev.get("type") == "result":
            result = ev
        content = (ev.get("message") or {}).get("content")
        if not isinstance(content, list):
            continue
        for block in content:
            if not isinstance(block, dict):
                continue
            if ev["type"] == "assistant" and block.get("type") == "tool_use" and block.get("name") == "Bash":
                cmd = (block.get("input") or {}).get("command", "")
                if any(m in cmd for m in markers):
                    pending[block["id"]] = True
                    test_rounds += 1
            elif ev["type"] == "user" and block.get("type") == "tool_result" and block.get("tool_use_id") in pending:
                body = block.get("content")
                if isinstance(body, list):
                    body = " ".join(b.get("text", "") for b in body if isinstance(b, dict))
                if block.get("is_error") or str(body).startswith("Exit code"):
                    failed += 1
    usage = (result or {}).get("usage") or {}
    return {
        "num_turns": (result or {}).get("num_turns", 0),
        "total_cost_usd": (result or {}).get("total_cost_usd", 0.0),
        "duration_ms": (result or {}).get("duration_ms", 0),
        **{f: usage.get(f, 0) for f in FIELDS},
        "failed_rounds": failed,
        "test_rounds": test_rounds,
        "agent_error": (result or {}).get("result") if (result or {}).get("is_error") else
                       (None if result else "no result record"),
    }


def run_claude(adapter, work, args):
    instruction = (f"Read CLAUDE.md, then implement the function stub in {adapter.SOLUTION}. "
                   f"Run `{adapter.test_command()}` until it passes, then stop.")
    env = dict(os.environ, CLAUDE_CODE_DISABLE_AUTO_MEMORY="1")
    start = time.time()
    _, out, timed_out = adapters.run(claude_cmd(args.model, args.max_turns, instruction, args.bare),
                                     work, args.agent_timeout, env)
    (work.parent / "transcript.jsonl").write_text(out)
    rec = parse_transcript(out, adapter.TEST_MARKERS)
    if timed_out:
        rec.update(agent_error="timeout", timeout=True,
                   duration_ms=rec["duration_ms"] or int((time.time() - start) * 1000))
    return rec


def run_mock(adapter, task, work, args):
    """No model: leave the stub (fails), or with --mock-solve write the reference."""
    start = time.time()
    ref = adapter.reference(task) if args.mock_solve else None
    if ref:
        (work / adapter.SOLUTION).write_text(ref)
    return {"num_turns": 0, "total_cost_usd": 0.0, "duration_ms": int((time.time() - start) * 1000),
            **{f: 0 for f in FIELDS}, "failed_rounds": 0, "test_rounds": 0, "agent_error": None}


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--benchmark", required=True, choices=["humaneval", "mbpp"])
    ap.add_argument("--lang", required=True, choices=["python", "rust", "overlang"])
    ap.add_argument("--model", default="sonnet")
    ap.add_argument("--tasks", default="0:5", help="start:stop slice of the shared task list (default 0:5)")
    ap.add_argument("--runs", type=int, default=3, help="runs per task")
    ap.add_argument("--agent", choices=["claude", "mock"], default="claude")
    ap.add_argument("--mock-solve", action="store_true", help="mock agent writes the dataset's reference solution if any")
    ap.add_argument("--max-turns", type=int, default=30)
    ap.add_argument("--agent-timeout", type=int, default=900, help="seconds per agent run")
    ap.add_argument("--grade-timeout", type=int, default=120, help="seconds per grading")
    ap.add_argument("--bare", action="store_true", help="claude --bare: no CLAUDE.md discovery from parent dirs (needs ANTHROPIC_API_KEY)")
    ap.add_argument("--out", type=Path, required=True, help="output .jsonl (workspaces go in a sibling dir of the same stem)")
    ap.add_argument("--tasks-dir", type=Path, default=HERE / "tasks")
    args = ap.parse_args()

    adapter = adapters.get(args.lang)
    runs_dir = args.out.parent / args.out.stem
    args.out.parent.mkdir(parents=True, exist_ok=True)
    done = set()
    if args.out.exists():
        for line in args.out.read_text().splitlines():
            r = json.loads(line)
            done.add((r["benchmark"], r["lang"], r["task"], r["run"], r["model"]))

    try:
        tasks = load_tasks(args.benchmark, args.lang, args.tasks, args.tasks_dir)
        for task in tasks:
            for run in range(args.runs):
                if (args.benchmark, args.lang, task["id"], run, args.model) in done:
                    print(f"skip {task['id']} run {run} (already in {args.out})")
                    continue
                run_dir = runs_dir / task["id"] / str(run)
                shutil.rmtree(run_dir, ignore_errors=True)
                work = run_dir / "work"
                adapter.workspace(task, work)
                rec = (run_claude(adapter, work, args) if args.agent == "claude"
                       else run_mock(adapter, task, work, args))
                graded = adapter.grade(work, args.grade_timeout)
                rec = {"benchmark": args.benchmark, "lang": args.lang, "task": task["id"], "run": run,
                       "model": args.model, "agent": args.agent, "passed": graded["passed"], **rec,
                       "grade_output": graded["output"][-500:]}
                with args.out.open("a") as f:
                    f.write(json.dumps(rec) + "\n")
                print(f"{task['id']} run {run}: {'PASS' if rec['passed'] else 'FAIL'}"
                      f"{' [' + rec['agent_error'][:60] + ']' if rec['agent_error'] else ''}")
    except NotImplementedError as e:
        sys.exit(f"{args.lang} adapter: {e}")


if __name__ == "__main__":
    main()
