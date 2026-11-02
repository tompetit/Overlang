#!/usr/bin/env bash
# Shared helpers for Benchmark C checks (the Tracker app, examples/tracker/SPEC.md). Source it from
# a task's check script:
#
#   . "$(dirname "$0")/../../lib.sh"; bb_init "$@"
#
# Arguments: <app dir> <lang>. The app dir holds main.py and its modules (python), or an exported
# main.ov or src/*.ov (overlang). The server runs as `<app> tracker.db <port>` with cwd = a fresh
# scratch dir, so tracker.db is new. Scratch lives under $BB_SCRATCH (default: <app dir>/../scratch),
# never /tmp. Exit 0 = pass. Any failing assertion prints FAIL and exits 1.

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

# The Overlang sources: an exported main.ov, else src/*.ov in name order (01_db .. 04_routes).
bb_sources() {
  if [ -f "$APP/main.ov" ]; then echo "$APP/main.ov"; else ls "$APP"/src/*.ov | sort; fi
}

bb_build() {
  [ "$LANG_" = overlang ] || return 0
  [ -n "${OV:-}" ] || fail "ov binary not found (cargo build --release, or set OV_BIN)"
  # shellcheck disable=SC2046
  (cd "$RUN" && "$OV" build $(bb_sources) -o "$RUN/app") >"$RUN/build.log" 2>&1 \
    || { cat "$RUN/build.log" >&2; fail "build failed"; }
}

# bb_start: start the server on a free port with db tracker.db, wait until /health answers.
bb_start() {
  PORT="$(python3 -c 'import socket;s=socket.socket();s.bind(("127.0.0.1",0));print(s.getsockname()[1])')"
  if [ "$LANG_" = python ]; then
    (cd "$RUN" && exec python3 -B "$APP/main.py" tracker.db "$PORT") >>"$RUN/server.log" 2>&1 &
  else
    (cd "$RUN" && exec ./app tracker.db "$PORT") >>"$RUN/server.log" 2>&1 &
  fi
  PID=$!
  URL="http://127.0.0.1:$PORT"
  for _ in $(seq 100); do
    curl -s -o /dev/null --max-time 2 "$URL/health" && return 0
    kill -0 "$PID" 2>/dev/null || fail "server exited early"
    sleep 0.1
  done
  fail "server did not come up"
}

bb_stop() { if [ -n "$PID" ]; then kill "$PID" 2>/dev/null || true; wait "$PID" 2>/dev/null || true; PID=""; fi; }
bb_restart() { bb_stop; bb_start; }

# HTTP helpers. get/post print the body; code prints the status; ctype prints the content type.
# getf PATH FILE saves the exact body (use it when trailing newlines or \r matter).
get()  { curl -s --max-time 10 "$URL$1"; }
getf() { curl -s --max-time 10 -o "$2" "$URL$1"; }
post() { curl -s --max-time 10 -d "$2" "$URL$1"; }
code() { if [ $# -ge 2 ]; then curl -s --max-time 10 -o /dev/null -w '%{http_code}' -d "$2" "$URL$1"; else curl -s --max-time 10 -o /dev/null -w '%{http_code}' "$URL$1"; fi; }
ctype() { curl -s --max-time 10 -o /dev/null -w '%{content_type}' "$URL$1"; }
# enc S: S form-encoded (spaces as +).
enc() { python3 -c 'import sys,urllib.parse;print(urllib.parse.quote_plus(sys.argv[1]))' "$1"; }

expect_code() { local want="$1"; shift; local got; got="$(code "$@")"; [ "$got" = "$want" ] || fail "$*: want status $want, got $got"; }
expect_eq() { [ "$1" = "$2" ] || fail "${3:-values differ}: want [$2] got [$1]"; }
expect_has() { grep -qF -- "$2" <<<"$1" || fail "${3:-missing}: want [$2] in [$1]"; }
expect_lacks() { ! grep -qF -- "$2" <<<"$1" || fail "${3:-unexpected}: found [$2] in [$1]"; }
# expect_err CODE BODY PATH [DATA]: status and exact body (minus the final newline); DATA means POST.
expect_err() { local want_code="$1" want_body="$2"; shift 2; expect_code "$want_code" "$@"
  local got; if [ $# -ge 2 ]; then got="$(post "$1" "$2")"; else got="$(get "$1")"; fi
  expect_eq "$got" "$want_body" "$* body"; }
# expect_file FILE PRINTF_FORMAT: FILE's bytes equal printf's output (exact, \r and final \n included).
expect_file() { local want; want="$(mktemp "$RUN/want.XXXXXX")"; printf "$2" >"$want"
  cmp -s "$1" "$want" || fail "$1: want [$(od -c "$want" | head -c 2000)] got [$(od -c "$1" | head -c 2000)]"; }

# The old behavior every check re-asserts (spec sections 2.2 to 2.4), on project BASE: users
# alice (1) and bob (2), labels bug and ui, issues BASE-1 (closed, labels bug,ui, a comment by bob)
# and BASE-2 (assigned to bob). It does not touch edit, label add/remove routes, JSON or CSV, and
# it does not restart; each check restarts once itself.
bb_baseline() {
  expect_eq "$(get /health)" "ok" "health"
  post /users name=alice >/dev/null; post /users name=bob >/dev/null
  expect_eq "$(get /users.txt)" $'1 alice\n2 bob' "users.txt"
  expect_err 422 "user exists" /users name=alice
  expect_err 422 "name required" /users name=
  expect_err 422 "bad name" /users "name=a+b"
  post /projects 'key=BASE&name=Base+line' >/dev/null
  expect_eq "$(get /projects.txt)" "BASE Base line" "projects.txt"
  expect_err 422 "bad key" /projects 'key=base&name=x'
  expect_has "$(get /)" '<a href="/p/BASE">BASE</a>' "home page"
  post /p/BASE/labels name=bug >/dev/null; post /p/BASE/labels name=ui >/dev/null
  expect_eq "$(get /p/BASE/labels.txt)" $'bug\nui' "labels.txt"
  post /p/BASE/issues 'title=Login+fails&body=Steps%3A+**click**&priority=1&assignee=alice&labels=bug,ui' >/dev/null
  post /p/BASE/issues 'title=Typo+on+home&body=50%25+of+the+text' >/dev/null
  expect_eq "$(get /p/BASE/issues.txt)" $'BASE-1 open P1 alice [bug,ui] Login fails\nBASE-2 open P3 - [] Typo on home' "issues.txt"
  expect_has "$(get /p/BASE/1)" '<strong>click</strong>' "issue page markdown"
  post /p/BASE/1/status status=closed >/dev/null
  post /p/BASE/2/assign assignee=bob >/dev/null
  post /p/BASE/1/comments 'author=bob&body=Fixed+in+%3Cb%3Ev2%3C%2Fb%3E' >/dev/null
  expect_eq "$(get /p/BASE/1/activity.txt)" $'created\nstatus closed\ncommented bob' "activity.txt"
  expect_eq "$(get /p/BASE/1/comments.txt)" 'bob: Fixed in <b>v2</b>' "comments.txt"
  expect_has "$(get /p/BASE/1)" 'Fixed in &lt;b&gt;v2&lt;/b&gt;' "comment escaped"
  expect_eq "$(get '/p/BASE/issues.txt?status=open')" 'BASE-2 open P3 bob [] Typo on home' "status filter"
  expect_eq "$(get '/p/BASE/issues.txt?q=50%25')" 'BASE-2 open P3 bob [] Typo on home' "q filter"
  expect_eq "$(get /p/BASE/1.txt)" 'BASE-1 closed P1 alice [bug,ui] Login fails' "issue txt"
  expect_err 404 "no such project: NOPE" /p/NOPE
  expect_err 404 "no such issue: BASE-99" /p/BASE/99
  expect_err 400 "bad page" '/p/BASE?page=abc'
  expect_err 400 "bad status" '/p/BASE?status=done'
  expect_err 422 "title required" /p/BASE/issues title=
  expect_err 422 "bad status" /p/BASE/1/status status=done
  expect_err 422 "no such label: nope" /p/BASE/issues 'title=x&labels=nope'
  expect_err 404 "not found" /nope
  expect_code 404 /p/BASE/abc
}

# The app's own unit tests must still pass.
bb_unit() {
  if [ "$LANG_" = python ]; then
    (cd "$APP" && python3 -B -I -m unittest) >"$RUN/unit.log" 2>&1 || { cat "$RUN/unit.log" >&2; fail "unit tests (python -m unittest) fail"; }
  else
    # shellcheck disable=SC2046
    (cd "$RUN" && "$OV" test $(bb_sources)) >"$RUN/unit.log" 2>&1 || { cat "$RUN/unit.log" >&2; fail "unit tests (ov test) fail"; }
  fi
}

# bb_sql <sql>: run a statement against the live server's database file (tracker.db in $RUN),
# print rows separated by | .
bb_sql() { python3 -c '
import sqlite3, sys
c = sqlite3.connect(sys.argv[1] + "/" + sys.argv[3])
for r in c.execute(sys.argv[2]).fetchall(): print("|".join(str(x) for x in r))
c.commit()' "$RUN" "$1" "${2:-tracker.db}"; }
