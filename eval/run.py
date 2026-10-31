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
    if benchmark == "benchb":  # maintenance tasks: eval/benchb/tasks/<id>/, sorted by id
        return adapters.get("benchb").load_tasks(spec)
    shared = json.loads((tasks_dir / "index.json").read_text())[benchmark]["shared"]
    start, _, stop = spec.partition(":")
    ids = shared[int(start or 0):int(stop) if stop else None]
    task_lang = adapters.get(lang).TASK_LANG
    files = [(i, tasks_dir / benchmark / task_lang / f"{i}.json") for i in ids]
    for i, f in files:
        if not f.exists():
            print(f"skip {i}: no {task_lang} task file", file=sys.stderr)
    return [json.loads(f.read_text()) for _, f in files if f.exists()]


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
    if result is None:
        result = aggregate_usage(text)
    usage = result.get("usage") or {}
    return {
        "num_turns": result.get("num_turns", 0),
        "total_cost_usd": result.get("total_cost_usd", 0.0),
        "duration_ms": result.get("duration_ms", 0),
        **{f: usage.get(f, 0) for f in FIELDS},
        "failed_rounds": failed,
        "test_rounds": test_rounds,
        "agent_error": result.get("result") if result.get("is_error") else
                       (None if result.get("usage") else "no result record"),
    }


def aggregate_usage(text):
    """A session transcript (subagent .jsonl) has no final result record: sum the usage
    of each distinct assistant API message instead. Cost is unknown there (0)."""
    seen, times = {}, []
    for line in text.splitlines():
        try:
            ev = json.loads(line)
        except ValueError:
            continue
        if not isinstance(ev, dict):
            continue
        if ev.get("timestamp"):
            times.append(ev["timestamp"])
        if ev.get("type") != "assistant":
            continue
        msg = ev.get("message") or {}
        if msg.get("usage"):
            seen.setdefault(msg.get("id") or f"anon{len(seen)}", msg["usage"])
    if not seen:
        return {}
    usage = {f: sum(u.get(f, 0) for u in seen.values()) for f in FIELDS}
    ms = 0
    if len(times) > 1:
        from datetime import datetime
        t = [datetime.fromisoformat(x.replace("Z", "+00:00")) for x in (min(times), max(times))]
        ms = int((t[1] - t[0]).total_seconds() * 1000)
    return {"usage": usage, "num_turns": len(seen), "duration_ms": ms, "total_cost_usd": 0.0}


def instruction_for(adapter, task):
    if hasattr(adapter, "instruction"):  # benchb: the task statement goes in the prompt
        return adapter.instruction(task)
    return (f"Read CLAUDE.md, then implement the function stub in {adapter.SOLUTION}. "
            f"Run `{adapter.test_command()}` until it passes, then stop.")


def run_subagent(adapter, task, work, args, key):
    """Agent runs are driven from an interactive Claude Code session via its Agent tool
    (used when `claude -p` cannot authenticate). First call: write the workspace and
    prompt.txt, return None (pending). Later calls: find the finished subagent transcript
    whose description is `key` under --transcripts and grade it."""
    prompt = work.parent / "prompt.txt"
    if not prompt.exists():
        prompt.write_text(f"You are solving one coding task. The workspace is {work.resolve()}. "
                          f"First `cd` there; run every command in that directory and only read or "
                          f"edit files inside it. Do not read, use or modify anything outside it "
                          f"(not the parent repository, not /tmp). Reply with one line when done.\n\n"
                          + instruction_for(adapter, task))
        return None
    for jl in sorted(Path(args.transcripts).rglob("agent-*.jsonl")):
        meta = jl.with_name(jl.name[:-len(".jsonl")] + ".meta.json")
        m = json.loads(meta.read_text()) if meta.exists() else None
        # workflow agents' meta has no `finished`; they are judged after the workflow ends
        if m is not None and not (m.get("description") == key and (m.get("finished") or "workflowPhase" in m)):
            continue
        with jl.open() as f:  # workflow agents have no meta file: match the prompt's workspace path
            first = f.readline()
        if m is None and str(work.resolve()) not in first:
            continue
        if True:
            out = jl.read_text()
            (work.parent / "transcript.jsonl").write_text(out)
            rec = parse_transcript(out, adapter.TEST_MARKERS)
            if args.benchmark == "benchb":
                rec.update(adapters.get("benchb").read_tokens(out))
            return rec
    return None


def run_claude(adapter, task, work, args):
    instruction = instruction_for(adapter, task)
    env = dict(os.environ, CLAUDE_CODE_DISABLE_AUTO_MEMORY="1")
    start = time.time()
    _, out, timed_out = adapters.run(claude_cmd(args.model, args.max_turns, instruction, args.bare),
                                     work, args.agent_timeout, env)
    (work.parent / "transcript.jsonl").write_text(out)
    rec = parse_transcript(out, adapter.TEST_MARKERS)
    if args.benchmark == "benchb":
        rec.update(adapters.get("benchb").read_tokens(out))
    if timed_out:
        rec.update(agent_error="timeout", timeout=True,
                   duration_ms=rec["duration_ms"] or int((time.time() - start) * 1000))
    return rec


def run_mock(adapter, task, work, args):
    """No model: leave the stub (fails), or with --mock-solve write the reference."""
    start = time.time()
    if args.mock_solve and hasattr(adapter, "apply_reference"):  # benchb: reference solution files
        adapter.apply_reference(task, work)
    else:
        ref = adapter.reference(task) if args.mock_solve else None
        if ref:
            (work / adapter.SOLUTION).write_text(ref)
    return {"num_turns": 0, "total_cost_usd": 0.0, "duration_ms": int((time.time() - start) * 1000),
            **{f: 0 for f in FIELDS}, "failed_rounds": 0, "test_rounds": 0, "agent_error": None}


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--benchmark", required=True, choices=["humaneval", "mbpp", "benchb"])
    ap.add_argument("--lang", required=True, choices=["python", "rust", "overlang"])
    ap.add_argument("--model", default="sonnet")
    ap.add_argument("--tasks", default="0:5", help="start:stop slice of the shared task list (default 0:5)")
    ap.add_argument("--runs", type=int, default=3, help="runs per task")
    ap.add_argument("--agent", choices=["claude", "mock", "subagent"], default="claude")
    ap.add_argument("--transcripts", type=Path, help="subagent transcript dir (<project>/<session>/subagents)")
    ap.add_argument("--mock-solve", action="store_true", help="mock agent writes the dataset's reference solution if any")
    ap.add_argument("--max-turns", type=int, default=30)
    ap.add_argument("--agent-timeout", type=int, default=900, help="seconds per agent run")
    ap.add_argument("--grade-timeout", type=int, default=120, help="seconds per grading")
    ap.add_argument("--bare", action="store_true", help="claude --bare: no CLAUDE.md discovery from parent dirs (needs ANTHROPIC_API_KEY)")
    ap.add_argument("--out", type=Path, required=True, help="output .jsonl (workspaces go in a sibling dir of the same stem)")
    ap.add_argument("--tasks-dir", type=Path, default=HERE / "tasks")
    args = ap.parse_args()

    if args.benchmark == "benchb":
        if args.lang not in ("python", "overlang"):
            sys.exit("benchb supports --lang python and overlang")
        adapter = adapters.get("benchb").for_lang(args.lang)
    else:
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
                work = run_dir / "work"
                key = f"ev {args.out.stem} {task['id']} {run}"  # unique per output file
                if args.agent == "subagent":
                    if not work.exists():
                        adapter.workspace(task, work)
                    rec = run_subagent(adapter, task, work, args, key)
                    if rec is None:
                        print(f"PENDING {key}")
                        continue
                else:
                    shutil.rmtree(run_dir, ignore_errors=True)
                    adapter.workspace(task, work)
                    rec = (run_claude(adapter, task, work, args) if args.agent == "claude"
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
