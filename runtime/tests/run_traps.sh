#!/bin/sh
# Run trap_main for each case; require exit 3, stdout "before" flushed, and one stderr line containing the expected text.
exe=$1
fail=0
n=0
check() {
  name=$1; want=$2
  n=$((n + 1))
  out=$($exe "$name" 2>/tmp/ov_trap_err.$$); code=$?
  err=$(cat /tmp/ov_trap_err.$$); rm -f /tmp/ov_trap_err.$$
  if [ "$code" != 3 ] || [ "$out" != before ] || [ "$(printf '%s\n' "$err" | wc -l)" != 1 ] || ! printf '%s' "$err" | grep -qF -- "$want"; then
    echo "FAIL $name: code=$code out=$out err=$err (wanted: $want)"; fail=1
  fi
}
check add   '{"kind": "overflow", "fn": "f", "hash": "abc", "detail": "integer overflow"}'
check sub   '"kind": "overflow"'
check mul   '"kind": "overflow"'
check neg   '"kind": "overflow"'
check divmin '"kind": "overflow"'
check div0  '"kind": "div_zero", "fn": "f"'
check mod0  '"kind": "div_zero"'
check bounds '"kind": "bounds", "fn": "f", "hash": "abc", "detail": "index 2 out of range for length 0"}'
check plain '"detail": null}'
check esc   '"fn": "f\"x", "hash": "abc", "detail": {"a": 1}}'
check requires '{"kind": "requires", "fn": "f", "hash": "abc", "clause": "x > 0", "args": {"x": "-1"}}'
check ensures  '{"kind": "ensures", "fn": "f", "hash": "abc", "clause": "result >= 0", "args": {}}'
[ $fail = 0 ] && echo "$n trap cases ok"
exit $fail
