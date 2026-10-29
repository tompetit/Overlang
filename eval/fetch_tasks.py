#!/usr/bin/env python3
"""Download HumanEval and MBPP tasks (Python and Rust) into eval/tasks/.

Rust: MultiPL-E rows from the Hugging Face datasets-server rows API
(configs humaneval-rs and mbpp-rs).

Python: the MultiPL-E dataset has no humaneval-py / mbpp-py config on the
rows API, so the Python variants are the MultiPL-E *original* .py files from
the nuprl/MultiPL-E GitHub repo (same task names as the Rust rows, so the two
languages line up one to one). Each original holds a typed stub with docstring,
the canonical solution (HumanEval only; MBPP has `pass`), and `check(candidate)`.

Output: tasks/<benchmark>/<lang>/<task_id>.json and tasks/index.json
(task ids present in every language, in dataset order). Existing files are
skipped, raw pages are cached in tasks/_cache/.
"""
import argparse
import json
import re
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

HF_ROWS = "https://datasets-server.huggingface.co/rows"
GH_RAW = "https://raw.githubusercontent.com/nuprl/MultiPL-E/main/datasets"
PY_DIRS = {"humaneval": "originals-with-cleaned-doctests", "mbpp": "mbpp-typed"}
PAGE = 100
SOL_MARK = "### Canonical solution below ###"
TEST_MARK = "### Unit tests below ###"


def http_get(url, retries=5):
    err = None
    for attempt in range(retries):
        try:
            with urllib.request.urlopen(url, timeout=60) as r:
                return r.read().decode("utf-8")
        except urllib.error.HTTPError as e:
            if e.code == 404:
                raise
            err = e
        except Exception as e:  # network blips, 429s
            err = e
        time.sleep(2 ** attempt)
    raise err


def task_order(name):
    return int(name.split("_")[1])


def fetch_rows(benchmark, cache, limit=None):
    """All rows of the Rust config, paged by 100 (up to `limit` rows)."""
    config = f"{benchmark}-rs"
    rows, offset, total = [], 0, None
    while total is None or offset < total:
        n = PAGE if limit is None else min(PAGE, limit - offset)
        if n <= 0:
            break
        page = cache / f"{config}-{offset}-{n}.json"
        if page.exists():
            data = json.loads(page.read_text())
        else:
            q = urllib.parse.urlencode({"dataset": "nuprl/MultiPL-E", "config": config,
                                        "split": "test", "offset": offset, "length": n})
            data = json.loads(http_get(f"{HF_ROWS}?{q}"))
            page.write_text(json.dumps(data))
        total = data["num_rows_total"]
        rows += [r["row"] for r in data["rows"]]
        if not data["rows"]:
            break
        offset += n
    return rows


def rust_task(benchmark, row):
    m = re.search(r"let candidate = (\w+);", row["tests"])
    return {"id": row["name"], "benchmark": benchmark, "lang": "rust", "name": row["name"],
            "entry_point": m.group(1) if m else None,
            "prompt": row["prompt"], "tests": row["tests"], "raw": row}


def python_task(benchmark, name, text, url):
    if SOL_MARK not in text or TEST_MARK not in text:
        return None
    prompt, rest = text.split(SOL_MARK, 1)
    prompt = prompt.rstrip() + "\n"  # drop the indent that preceded the marker
    solution, tests = rest.split(TEST_MARK, 1)
    entry = re.findall(r"check\((\w+)\)", tests)
    if not entry:
        return None
    # MBPP originals have a bare `pass` instead of a reference solution.
    solution = None if solution.strip() == "pass" else prompt + solution
    return {"id": name, "benchmark": benchmark, "lang": "python", "name": name,
            "entry_point": entry[-1], "prompt": prompt, "tests": tests.lstrip("\n"),
            "solution": solution, "raw": {"source_url": url, "text": text}}


def write(path, obj):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(obj, indent=1) + "\n")


def write_index(out):
    index = {}
    for bench in PY_DIRS:
        ids = {lang: {p.stem for p in (out / bench / lang).glob("*.json")}
               for lang in ("python", "rust")}
        shared = sorted(ids["python"] & ids["rust"], key=task_order)
        if any(ids.values()):
            index[bench] = {"shared": shared, "python": len(ids["python"]), "rust": len(ids["rust"])}
    write(out / "index.json", index)
    return index


def fetch_python(bench, rows, out):
    def get_py(row):
        name = row["name"]
        path = out / bench / "python" / f"{name}.json"
        if path.exists():
            return 0
        url = f"{GH_RAW}/{PY_DIRS[bench]}/{name}.py"
        try:
            task = python_task(bench, name, http_get(url), url)
        except urllib.error.HTTPError:
            task = None
        if task is None:
            print(f"warning: no usable Python original for {name}", file=sys.stderr)
            return 0
        write(path, task)
        return 1
    with ThreadPoolExecutor(8) as pool:
        return sum(pool.map(get_py, rows))


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--out", type=Path, default=Path(__file__).parent / "tasks")
    ap.add_argument("--benchmarks", nargs="+", default=list(PY_DIRS), choices=list(PY_DIRS))
    ap.add_argument("--langs", nargs="+", default=["python", "rust"], choices=["python", "rust"])
    ap.add_argument("--limit", type=int, help="only the first N tasks of each benchmark")
    args = ap.parse_args()

    cache = args.out / "_cache"
    cache.mkdir(parents=True, exist_ok=True)
    for bench in args.benchmarks:
        rows = fetch_rows(bench, cache, args.limit)  # also supplies the task names for Python
        new = 0
        if "rust" in args.langs:
            for row in rows:
                path = args.out / bench / "rust" / f"{row['name']}.json"
                if not path.exists():
                    write(path, rust_task(bench, row))
                    new += 1
        if "python" in args.langs:
            new += fetch_python(bench, rows, args.out)
        print(f"{bench}: {len(rows)} rows, {new} new task files")
    index = write_index(args.out)
    for bench, v in index.items():
        print(f"{bench}: python={v['python']} rust={v['rust']} shared={len(v['shared'])}")


if __name__ == "__main__":
    main()
