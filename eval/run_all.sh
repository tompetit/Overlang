#!/usr/bin/env bash
# Runs the whole evaluation described in docs/PLAN.md section 6.
# Prerequisites: `claude -p` must authenticate (run `claude login`, or export
# ANTHROPIC_API_KEY), and `cargo build --release` must have produced target/release/ov.
# Resumable: finished runs in each .jsonl are skipped on rerun.
#
# Rough cost at Sonnet rates: Benchmark A 900 trajectories, Benchmark B 150
# longer ones; budget about $1,000 to $1,500 in total. Pass a smaller --tasks
# range or --runs to try it first, e.g. ./eval/run_all.sh 0:5 1
set -euo pipefail
cd "$(dirname "$0")/.."
RANGE="${1:-0:100}"
RUNS="${2:-3}"
MODEL="${MODEL:-sonnet}"
STAMP="$(date +%Y%m%d)"
export OV_BIN="$PWD/target/release/ov"
[ -x "$OV_BIN" ] || { echo "build the compiler first: cargo build --release" >&2; exit 1; }
if ! claude -p "ok" --model haiku --output-format json 2>/dev/null | grep -q '"is_error":false'; then
  echo "claude -p is not authenticated; run: claude login" >&2; exit 1
fi
mkdir -p ./eval/runs
for lang in python rust overlang; do
  python3 ./eval/run.py --benchmark humaneval --lang "$lang" --model "$MODEL" \
    --tasks "$RANGE" --runs "$RUNS" --agent claude --out "./eval/runs/A-$STAMP-$lang.jsonl"
done
for lang in python overlang; do
  python3 ./eval/run.py --benchmark benchb --lang "$lang" --model "$MODEL" \
    --tasks 0:25 --runs "$RUNS" --agent claude --out "./eval/runs/B-$STAMP-$lang.jsonl"
done
python3 ./eval/report.py ./eval/runs/A-$STAMP-*.jsonl
python3 ./eval/report.py --compare python overlang ./eval/runs/A-$STAMP-python.jsonl ./eval/runs/A-$STAMP-overlang.jsonl
python3 ./eval/report.py ./eval/runs/B-$STAMP-*.jsonl
python3 ./eval/report.py --compare python overlang ./eval/runs/B-$STAMP-python.jsonl ./eval/runs/B-$STAMP-overlang.jsonl
