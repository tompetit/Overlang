#!/usr/bin/env bash
# Shared helpers for Benchmark B checks. Source it from a task's check script:
#
#   . "$(dirname "$0")/../../lib.sh"; bb_init "$@"
#
# Arguments: <app dir> <lang>. The app dir holds main.py (python) or an exported
# main.ov (overlang). Every server runs with cwd = a fresh scratch dir, so todo.db is
# new. Scratch lives under $BB_SCRATCH (default: <app dir>/../scratch), never /tmp.
# Exit 0 = pass. Any failing assertion prints FAIL and exits 1.

BB_LIB="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BB_ROOT="$(cd "$BB_LIB/../.." && pwd)"

bb_init() {
  APP="$(cd "$1" && pwd)" || { echo "FAIL: no app dir $1" >&2; exit 2; }; LANG_="$2"
  case "$LANG_" in python|overlang) ;; *) echo "FAIL: bad language '$LANG_'" >&2; exit 2;; esac
  TASK="$(cd "$(dirname "${BASH_SOURCE[1]}")" && pwd)"
  mkdir -p "${BB_SCRATCH:-$APP/../scratch}"
  RUN="$(mktemp -d "$(cd "${BB_SCRATCH:-$APP/../scratch}" && pwd)/run.XXXXXX")"
  PID=""; PORT=""
  for p in "${OV_BIN:-}" "$BB_ROOT/target/release/ov" "$(command -v ov || true)"; do
    if [ -n "$p" ] && [ -x "$p" ]; then OV="$p"; break; fi
  done
  trap bb_cleanup EXIT
  export PYTHONDONTWRITEBYTECODE=1
  bb_build
  bb_start
}

bb_cleanup() { bb_stop; rm -rf "$RUN"; }

fail() {
  echo "FAIL: $*" >&2
  [ -f "$RUN/server.log" ] && { echo "--- server log ---" >&2; tail -n 20 "$RUN/server.log" >&2; }
  exit 1
}

bb_build() {
  [ "$LANG_" = overlang ] || return 0
  [ -n "${OV:-}" ] || fail "ov binary not found (cargo build --release, or set OV_BIN)"
  (cd "$RUN" && "$OV" build "$APP/main.ov" -o "$RUN/app") >"$RUN/build.log" 2>&1 \
    || { cat "$RUN/build.log" >&2; fail "build failed"; }
}

# bb_start [program args after the port]: start the server on a free port, wait until it answers.
bb_start() {
  PORT="$(python3 -c 'import socket;s=socket.socket();s.bind(("127.0.0.1",0));print(s.getsockname()[1])')"
  if [ "$LANG_" = python ]; then
    (cd "$RUN" && exec python3 -B "$APP/main.py" "$PORT" "$@") >>"$RUN/server.log" 2>&1 &
  else
    (cd "$RUN" && exec ./app "$PORT" "$@") >>"$RUN/server.log" 2>&1 &
  fi
  PID=$!
  URL="http://127.0.0.1:$PORT"
  for _ in $(seq 100); do
    curl -s -o /dev/null --max-time 2 "$URL/todos.txt" && return 0
    kill -0 "$PID" 2>/dev/null || fail "server exited early"
    sleep 0.1
  done
  fail "server did not come up"
}

bb_stop() { if [ -n "$PID" ]; then kill "$PID" 2>/dev/null || true; wait "$PID" 2>/dev/null || true; PID=""; fi; }
bb_restart() { bb_stop; bb_start "$@"; }

# HTTP helpers. get/post print the body; code prints the status; ctype prints the content type.
get()  { curl -s --max-time 10 "$URL$1"; }
post() { curl -s --max-time 10 -d "$2" "$URL$1"; }
code() { if [ $# -ge 2 ]; then curl -s --max-time 10 -o /dev/null -w '%{http_code}' -d "$2" "$URL$1"; else curl -s --max-time 10 -o /dev/null -w '%{http_code}' "$URL$1"; fi; }
ctype() { curl -s --max-time 10 -o /dev/null -w '%{content_type}' "$URL$1"; }
add() { post /add "title=$(python3 -c 'import sys,urllib.parse;print(urllib.parse.quote_plus(sys.argv[1]))' "$1")${2:+&$2}" >/dev/null; }
list() { get /todos.txt; }

expect_code() { local want="$1"; shift; local got; got="$(code "$@")"; [ "$got" = "$want" ] || fail "$*: want status $want, got $got"; }
expect_eq() { [ "$1" = "$2" ] || fail "${3:-values differ}: want [$2] got [$1]"; }
expect_has() { grep -qF -- "$2" <<<"$1" || fail "${3:-missing}: want [$2] in [$1]"; }
expect_lacks() { ! grep -qF -- "$2" <<<"$1" || fail "${3:-unexpected}: found [$2] in [$1]"; }
expect_list() { local got; got="$(list)"; [ "$got" = "$1" ] || fail "todos.txt: want [$1] got [$got]"; }

# The old behavior every check re-asserts: add, toggle, delete, 404 and persistence of /todos.txt
# in the original format. Tasks that change that format do not call this.
bb_baseline() {
  add "buy milk"; add "write & ship!"
  expect_list $'1 [ ] buy milk\n2 [ ] write & ship!'
  post /toggle id=1 >/dev/null
  expect_list $'1 [x] buy milk\n2 [ ] write & ship!'
  expect_has "$(get /)" 'write &amp; ship!' "html page escapes"
  post /delete id=1 >/dev/null
  expect_list '2 [ ] write & ship!'
  expect_code 404 /nope
  expect_code 400 /toggle id=0
  expect_code "${BB_BLANK_CODE:-400}" /add title=
  bb_restart
  expect_list '2 [ ] write & ship!'
}

# The app's own unit tests must still pass.
bb_unit() {
  if [ "$LANG_" = python ]; then
    (cd "$APP" && python3 -B -I -m unittest) >"$RUN/unit.log" 2>&1 || { cat "$RUN/unit.log" >&2; fail "unit tests (python -m unittest) fail"; }
  else
    (cd "$RUN" && "$OV" test "$APP/main.ov") >"$RUN/unit.log" 2>&1 || { cat "$RUN/unit.log" >&2; fail "unit tests (ov test) fail"; }
  fi
}

# bb_probe: run the task's hidden unit-level probe against the app: probe.ov (as extra `test`
# blocks loaded with main.ov) or probe.py (plain script; exit 0 = pass).
bb_probe() {
  if [ "$LANG_" = python ]; then
    (cd "$RUN" && BB_APP="$APP" python3 -B -I "$TASK/probe.py") >"$RUN/probe.log" 2>&1 \
      || { cat "$RUN/probe.log" >&2; fail "probe.py failed"; }
  else
    (cd "$RUN" && "$OV" test "$APP/main.ov" "$TASK/probe.ov") >"$RUN/probe.log" 2>&1 \
      || { cat "$RUN/probe.log" >&2; fail "probe.ov failed"; }
    local n
    for n in $(sed -n 's/^test \([A-Za-z0-9_]*\).*/\1/p' "$TASK/probe.ov"); do
      grep -q "^ok $n\$" "$RUN/probe.log" || { cat "$RUN/probe.log" >&2; fail "probe test $n did not report ok"; }
    done
  fi
}

# bb_sql <sql>: run a statement against the live server's database file (a todo.db in $RUN),
# print rows separated by | .
bb_sql() { python3 -c '
import sqlite3, sys
c = sqlite3.connect(sys.argv[1] + "/" + sys.argv[3])
for r in c.execute(sys.argv[2]).fetchall(): print("|".join(str(x) for x in r))
c.commit()' "$RUN" "$1" "${2:-todo.db}"; }
