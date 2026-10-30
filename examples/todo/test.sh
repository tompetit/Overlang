#!/usr/bin/env bash
# End-to-end test: build, run against a fresh database, exercise routes with curl, restart, check persistence.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
ov="${OV:-$here/../../target/release/ov}"
mkdir -p "$here/.tmp"
work="$(mktemp -d "$here/.tmp/run.XXXXXX")"
pid=""
stop() { if [ -n "$pid" ]; then kill "$pid" 2>/dev/null || true; wait "$pid" 2>/dev/null || true; pid=""; fi; }
trap 'stop; rm -rf "$work"' EXIT
fail() { echo "FAIL: $*" >&2; echo "--- server log ---" >&2; cat "$work/server.log" >&2 || true; exit 1; }

"$ov" build "$here/main.ov" -o "$work/todo"

# Pick a free port (best effort) and start the server with cwd = work, so todo.db is fresh.
port=$(( 20000 + RANDOM % 20000 ))
start() {
  (cd "$work" && exec ./todo "$port") >>"$work/server.log" 2>&1 &
  pid=$!
  for _ in $(seq 100); do
    curl -s -o /dev/null "http://127.0.0.1:$port/todos.txt" && return 0
    kill -0 "$pid" 2>/dev/null || fail "server exited early"
    sleep 0.1
  done
  fail "server did not come up"
}
url="http://127.0.0.1:$port"
list() { curl -sf "$url/todos.txt"; }
post() { curl -sf -o /dev/null -d "$2" "$url$1"; }
expect_has() { list | grep -q -- "$1" || fail "expected '$1' in list: $(list)"; }
expect_lacks() { ! list | grep -q -- "$1" || fail "unexpected '$1' in list: $(list)"; }

start
post /add "title=buy+milk"
post /add "title=write+%26+ship%21"
expect_has "buy milk"
expect_has "write & ship!"
post /toggle "id=1"
list | grep -q '^1 \[x\] buy milk$' || fail "toggle did not mark todo 1 done"
curl -sf "$url/" | grep -q 'write &amp; ship!' || fail "html page missing escaped title"
post /delete "id=1"
expect_lacks "buy milk"
expect_has "write & ship!"
[ "$(curl -s -o /dev/null -w '%{http_code}' "$url/nope")" = 404 ] || fail "expected 404"

stop
start
expect_has "write & ship!"
expect_lacks "buy milk"
echo "PASS"
