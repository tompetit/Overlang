#!/usr/bin/env python3
"""Validate Benchmark B tasks: every check must FAIL on the unmodified app and PASS on the
reference solution, for each language.

    python3 eval/benchb/validate.py                      # all tasks, both languages
    python3 eval/benchb/validate.py --tasks b01_priority,b04_blank_422 --lang python

Workspaces go to eval/runs/_benchb_validate/<task>/<lang>/. Exit 1 if any task misbehaves.
"""
import argparse
import shutil
import sys
from pathlib import Path

EVAL = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(EVAL))

import adapters  # noqa: E402
from adapters import benchb  # noqa: E402

OUT = EVAL / "runs" / "_benchb_validate"


def last_fail(output):
    lines = [l for l in output.splitlines() if l.startswith("FAIL")]
    return (lines[-1] if lines else (output.strip().splitlines() or [""])[-1])[:110]


def validate(task_id, lang, verbose=False):
    ad = benchb.for_lang(lang)
    task = {"id": task_id}
    work = OUT / task_id / lang / "work"
    shutil.rmtree(work.parent, ignore_errors=True)
    ad.workspace(task, work)
    before = ad.grade(work, 120)
    ad.apply_reference(task, work)
    after = ad.grade(work, 120)
    ok = (not before["passed"]) and after["passed"]
    if verbose or not ok:
        print(f"--- {task_id} {lang}\nunmodified: {before['output'][-600:]}\nsolved: {after['output'][-600:]}")
    return ok, before, after


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--tasks", default="all")
    ap.add_argument("--lang", default="python,overlang")
    ap.add_argument("-v", "--verbose", action="store_true")
    args = ap.parse_args()
    ids = benchb.list_tasks() if args.tasks == "all" else args.tasks.split(",")
    bad = 0
    for tid in ids:
        for lang in args.lang.split(","):
            ok, before, after = validate(tid, lang, args.verbose)
            bad += not ok
            print(f"{'ok  ' if ok else 'BAD '} {tid:28} {lang:9} unmodified: "
                  f"{'FAIL' if not before['passed'] else 'PASS?!'} ({last_fail(before['output'])})  "
                  f"solved: {'PASS' if after['passed'] else 'FAIL?!'}", flush=True)
    sys.exit(1 if bad else 0)


if __name__ == "__main__":
    main()
