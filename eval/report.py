#!/usr/bin/env python3
"""Summarise run .jsonl files.

    report.py eval/runs/a.jsonl [more.jsonl ...]
    report.py eval/runs/*.jsonl --compare python overlang

Default: one table per (benchmark, lang, model) with n, pass rate and the
median/mean of each token field, cost and duration, plus median turns and
failed_rounds. Runs whose agent hit an API/infra error (agent_error set, but
not a timeout) are excluded unless --include-errors.

--compare A B: for each (benchmark, model) present in both languages, the ratio
B/A of every median (and the pass-rate difference in points). A ratio of 1.3
means B used 30% more.
"""
import argparse
import json
import statistics
from collections import defaultdict

METRICS = ["input_tokens", "output_tokens", "cache_creation_input_tokens", "cache_read_input_tokens",
           "total_tokens", "cache_hit_rate", "total_cost_usd", "duration_ms", "num_turns", "failed_rounds",
           "read_tokens"]


def load(paths, include_errors=False):
    rows, excluded = [], 0
    for path in paths:
        with open(path) as f:
            for line in f:
                if line.strip():
                    r = json.loads(line)
                    if r.get("agent_error") and not r.get("timeout") and not include_errors:
                        excluded += 1
                        continue
                    rows.append(r)
    return rows, excluded


def derived(r):
    inp = r["input_tokens"] + r["cache_creation_input_tokens"] + r["cache_read_input_tokens"]
    return dict(r, total_tokens=inp + r["output_tokens"],
                cache_hit_rate=r["cache_read_input_tokens"] / inp if inp else 0.0)


def summarise(rows):
    """{(benchmark, lang, model): {"n", "pass_rate", metric: (median, mean)}}"""
    groups = defaultdict(list)
    for r in rows:
        groups[(r["benchmark"], r["lang"], r["model"])].append(derived(r))
    out = {}
    for key, rs in groups.items():
        s = {"n": len(rs), "pass_rate": sum(bool(r["passed"]) for r in rs) / len(rs)}
        for m in METRICS:
            vals = [r.get(m, 0) for r in rs]
            s[m] = (statistics.median(vals), statistics.fmean(vals))
        out[key] = s
    return out


def fmt(x):
    return f"{x:,.0f}" if abs(x) >= 100 else f"{x:.3g}"


def print_tables(summary):
    for (bench, lang, model), s in sorted(summary.items()):
        print(f"\n{bench} / {lang} / {model}: n={s['n']}  pass rate={s['pass_rate']:.1%}")
        print(f"  {'metric':<30}{'median':>14}{'mean':>14}")
        for m in METRICS:
            print(f"  {m:<30}{fmt(s[m][0]):>14}{fmt(s[m][1]):>14}")


def print_compare(summary, a, b):
    pairs = sorted({(bench, model) for (bench, lang, model) in summary if lang in (a, b)})
    for bench, model in pairs:
        sa, sb = summary.get((bench, a, model)), summary.get((bench, b, model))
        if not sa or not sb:
            continue
        print(f"\n{bench} / {model}: {b} vs {a}  (n={sb['n']} vs {sa['n']}, "
              f"pass rate {sb['pass_rate']:.1%} vs {sa['pass_rate']:.1%}, "
              f"diff {100 * (sb['pass_rate'] - sa['pass_rate']):+.1f} points)")
        print(f"  {'metric (median)':<30}{a:>14}{b:>14}{'ratio':>10}")
        for m in METRICS:
            ma, mb = sa[m][0], sb[m][0]
            ratio = f"{mb / ma:.2f}" if ma else "n/a"
            print(f"  {m:<30}{fmt(ma):>14}{fmt(mb):>14}{ratio:>10}")


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("files", nargs="+")
    ap.add_argument("--compare", nargs=2, metavar=("BASE_LANG", "OTHER_LANG"))
    ap.add_argument("--include-errors", action="store_true", help="keep runs with API/infra errors")
    args = ap.parse_args()
    rows, excluded = load(args.files, args.include_errors)
    summary = summarise(rows)
    if args.compare:
        print_compare(summary, *args.compare)
    else:
        print_tables(summary)
    if excluded:
        print(f"\n({excluded} runs excluded for agent errors; use --include-errors to keep them)")


if __name__ == "__main__":
    main()
