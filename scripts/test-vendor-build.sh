#!/usr/bin/env bash
# Hermetic test for scripts/vendor-build.sh — the per-worktree queue reader,
# the memory gate on `run`, and copy pruning.
#
# Hermetic: fixture target trees in a temp dir plus python3 flock holders.
# No cargo, no network, no repo writes, and it never touches
# ~/.cache/dsb-vendor-targets (every case passes --root into the temp dir).
#
# Why the tool exists (measured 2026-09-26): three sessions shared the warm
# vendored target as CARGO_TARGET_DIR; one held the cargo advisory flock on
# target/debug/.cargo-lock while compiling, two waited with zero children for
# 6-13 minutes and printing nothing. That shared build target is gone (one
# worktree, one target — scripts/test-vendor-cargo.sh pins that), so `status`
# reads every worktree's target: one vendored build at a time is a whole-host
# rule and no single lock file answers it any more.
#
# Paths covered:
#   0. header     — a plain run emits no shell errors
#   1. free       — no lock holder: exit 0
#   2. no target  — nothing built there yet: exit 0
#   3. holder     — a process with a compiler child: exit 1, inferred holder
#   4. json       — the machine-readable contract (single target)
#   5. waiter     — a process with no descendants, past the start grace: waiter
#   6. released   — lock gone: exit 0 again
#   7. worktrees  — the default reads every worktree's target and names them
#   8. prune      — deletes only idle copies; the base target survives
#   9. run        — pass-through on a free queue, memory gate on a busy one
#
# Usage: ./scripts/test-vendor-build.sh
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SCRIPT="$ROOT/scripts/vendor-build.sh"

PASS=0
FAIL=0
TMP="$(cd "$(mktemp -d)" && pwd -P)"
BASE="$TMP/base-target"
CLONES="$TMP/clones"
FIXTURE_PIDS=()

cleanup() {
  for pid in ${FIXTURE_PIDS[@]+"${FIXTURE_PIDS[@]}"}; do
    kill "$pid" 2>/dev/null || true
  done
  wait 2>/dev/null || true
  CHILDREN="$(grep -h -o 'child [0-9][0-9]*' "$TMP"/fx-*.out 2>/dev/null || true)"
  for child in $(printf '%s\n' "$CHILDREN" | awk '{print $2}'); do
    kill "$child" 2>/dev/null || true
  done
  rm -rf "$TMP"
}
trap cleanup EXIT

ok()  { printf '  ok   %s\n' "$*"; PASS=$((PASS + 1)); }
bad() { printf '  FAIL %s\n' "$*" >&2; FAIL=$((FAIL + 1)); }
head_() { printf '\n== %s ==\n' "$*"; }

for c in python3 lsof ps git; do
  command -v "$c" >/dev/null 2>&1 || { echo "error: $c not found on PATH" >&2; exit 1; }
done
[[ -x "$SCRIPT" ]] || { echo "error: $SCRIPT not executable" >&2; exit 1; }

# Run a command, capture stdout/stderr separately into OUT/ERR, RC into RC.
run_capture() {
  RC=0
  OUT="$("$@" 2>"$TMP/stderr.txt")" || RC=$?
  ERR="$(cat "$TMP/stderr.txt")"
}

wait_holding() { # <logfile>
  local i
  for i in $(seq 1 100); do
    if grep -q holding "$1" 2>/dev/null; then return 0; fi
    sleep 0.1
  done
  return 1
}

# --- fixtures --------------------------------------------------------------
mkdir -p "$BASE/debug/deps" "$BASE/debug/incremental"
: > "$BASE/debug/.cargo-lock"
printf 'artifact\n' > "$BASE/debug/deps/libfixture.rlib"
printf 'artifact\n' > "$BASE/debug/incremental/chunk"

# A sleep symlink named rustc: a child that ps reports as `…/fakebin/rustc`,
# which is what the compiler-descendant rule keys on.
FAKEBIN="$TMP/fakebin"
mkdir -p "$FAKEBIN"
ln -sf "$(command -v sleep)" "$FAKEBIN/rustc"

# A lock holder. mode "compiler": holds the lock with a live fake-rustc child
# (its pid goes to the fixture log so cleanup can end it). mode "plain": no
# child. mode "release-after": releases the lock after N seconds and exits.
cat > "$TMP/fx-holder.py" <<'PY'
import fcntl, os, subprocess, sys, time
lock, mode, fakebin = sys.argv[1], sys.argv[2], sys.argv[3]
seconds = float(sys.argv[4]) if len(sys.argv) > 4 else 240.0
f = open(lock, "rb")
fcntl.flock(f, fcntl.LOCK_EX)
child = None
if mode == "compiler":
    child = subprocess.Popen([os.path.join(fakebin, "rustc"), "120"])
print("holding", os.getpid(), "child", child.pid if child else "-", flush=True)
time.sleep(seconds)
if mode == "release-after":
    fcntl.flock(f, fcntl.LOCK_UN)
    f.close()
PY

start_fixture() { # <lockfile> <mode> [seconds] -> FIXTURE_PID
  # cwd stays inside the fixture tree so the worktree mapping has nothing to map.
  (cd "$(dirname "$1")/../.." && exec python3 "$TMP/fx-holder.py" "$1" "$2" "$FAKEBIN" "${3:-240}") \
    > "$TMP/fx-$2.out" 2>&1 &
  FIXTURE_PID=$!
  FIXTURE_PIDS+=("$FIXTURE_PID")
  wait_holding "$TMP/fx-$2.out" || { echo "fixture $2 never reported holding" >&2; exit 1; }
}

# --- 0. the dispatcher header runs clean -----------------------------------
# A stray editor anchor in the script header used to make the shell print
# "command not found" to stderr before the real output; nothing parsed stderr,
# so it slipped through. Pin a clean stderr on a plain run.
head_ "0. the dispatcher header emits no shell errors"
run_capture "$SCRIPT" status --target "$TMP/never-built"
case "$ERR" in
  *"command not found"*) bad "the dispatcher emits shell errors: $ERR" ;;
  *"syntax error"*) bad "the dispatcher has a shell syntax error: $ERR" ;;
  *) ok "stderr is clean on a plain run" ;;
esac

# --- 1. free ---------------------------------------------------------------
head_ "1. no holder: free, exit 0"
run_capture "$SCRIPT" status --target "$BASE"
[[ "$RC" -eq 0 ]] && ok "exit 0" || bad "exit $RC (wanted 0)"
case "$OUT" in *"lock debug/.cargo-lock: free"*) ok "lock reported free" ;;
  *) bad "free lock not reported: $OUT" ;; esac
case "$OUT" in *"FREE → exit 0"*) ok "free verdict printed" ;;
  *) bad "no free verdict: $OUT" ;; esac

# --- 2. target absent ------------------------------------------------------
head_ "2. no target directory: free, exit 0"
run_capture "$SCRIPT" status --target "$TMP/never-built"
[[ "$RC" -eq 0 ]] && ok "exit 0" || bad "exit $RC (wanted 0)"
case "$OUT" in *"does not exist"*) ok "absent target named" ;;
  *) bad "absent target not named: $OUT" ;; esac

# --- 3. holder with a compiler child ---------------------------------------
head_ "3. holder with a compiler child: busy, exit 1, inferred holder"
start_fixture "$BASE/debug/.cargo-lock" compiler
HOLDER="$FIXTURE_PID"
run_capture "$SCRIPT" status --target "$BASE"
[[ "$RC" -eq 1 ]] && ok "exit 1 (busy)" || bad "exit $RC (wanted 1)"
case "$OUT" in *"BUSY (file lock held)"*) ok "lock reported busy via the file-lock probe" ;;
  *) bad "busy lock not reported: $OUT" ;; esac
case "$OUT" in *"pid $HOLDER"*) ok "holder pid $HOLDER reported" ;;
  *) bad "holder pid not reported: $OUT" ;; esac
case "$OUT" in *"holder (inferred)"*) ok "inferred holder label" ;;
  *) bad "holder label missing: $OUT" ;; esac
case "$OUT" in *"compiler descendant"*) ok "compiler descendant in the basis" ;;
  *) bad "compiler descendant missing: $OUT" ;; esac
case "$OUT" in *"(inferred) —"*) ok "the inference is marked, not presented as fact" ;;
  *) bad "inference mark missing: $OUT" ;; esac

# --- 4. json contract ------------------------------------------------------
head_ "4. status --json: the machine contract"
run_capture "$SCRIPT" status --json --target "$BASE"
[[ "$RC" -eq 1 ]] && ok "exit 1 (busy)" || bad "exit $RC (wanted 1)"
printf '%s' "$OUT" > "$TMP/status.json"
python3 - "$TMP/status.json" "$HOLDER" <<'PY' && ok "json fields hold" || bad "json contract broken"
import json, sys
d = json.load(open(sys.argv[1]))
pid = int(sys.argv[2])
assert d["target"].endswith("base-target"), d["target"]
assert d["status"] == "busy", d["status"]
assert d["exit_code"] == 1, d["exit_code"]
locks = [l for l in d["locks"] if l["busy"]]
assert locks, "no busy lock"
assert locks[0]["profile"] == "debug" and locks[0]["probe"] == "locked", locks[0]
procs = [p for p in locks[0]["processes"] if p["pid"] == pid]
assert procs, f"pid {pid} not reported"
p = procs[0]
assert p["label"] == "holder", p["label"]
assert "compiler descendant" in p["basis"], p["basis"]
assert p["worktree"] is None, f"synthetic tree mapped to {p['worktree']}"
assert any("rustc" in (x.get("comm") or "") for x in p["descendants"]), p["descendants"]
PY

# --- 5. waiter -------------------------------------------------------------
head_ "5. no descendants past the start grace: waiter"
kill "$FIXTURE_PID" 2>/dev/null || true
wait "$FIXTURE_PID" 2>/dev/null || true
start_fixture "$BASE/debug/.cargo-lock" plain
WAITER="$FIXTURE_PID"
sleep 3.2   # STARTING_GRACE_SECONDS is 3; before that the label is "unknown"
run_capture "$SCRIPT" status --target "$BASE"
[[ "$RC" -eq 1 ]] && ok "exit 1 (busy)" || bad "exit $RC (wanted 1)"
case "$OUT" in *"pid $WAITER"*) ok "waiter pid $WAITER reported" ;;
  *) bad "waiter pid not reported: $OUT" ;; esac
case "$OUT" in *"waiter (inferred) — no descendants"*) ok "waiter labelled from the empty process tree" ;;
  *) bad "waiter label missing: $OUT" ;; esac

# --- 6. released -----------------------------------------------------------
head_ "6. lock released: free again, exit 0"
kill "$FIXTURE_PID" 2>/dev/null || true
wait "$FIXTURE_PID" 2>/dev/null || true
run_capture "$SCRIPT" status --target "$BASE"
[[ "$RC" -eq 0 ]] && ok "exit 0" || bad "exit $RC (wanted 0)"
case "$OUT" in *"lock debug/.cargo-lock: free"*) ok "lock free after release" ;;
  *) bad "lock not free after release: $OUT" ;; esac

# --- 7. every worktree's target, by default --------------------------------
# The default reads <worktree>/third_party/grok-build/target for each worktree
# of the repository, so a busy build is visible without naming a target.
head_ "7. status reads every worktree's target by default"
REPO="$TMP/repo"
WT2="$TMP/repo-wt"
WT3="$TMP/repo-wt3"
V=third_party/grok-build
mkdir -p "$REPO"
printf '%s/target/\n' "$V" > "$REPO/.gitignore"
git -C "$REPO" init -q -b main
git -C "$REPO" -c user.name=t -c user.email=t@t add -A
git -C "$REPO" -c user.name=t -c user.email=t@t commit -q -m fixture
git -C "$REPO" worktree add -q "$WT2" -b wt2
git -C "$REPO" worktree add -q "$WT3" -b wt3
mkdir -p "$REPO/$V/target/debug" "$WT2/$V/target/debug"
: > "$REPO/$V/target/debug/.cargo-lock"
: > "$WT2/$V/target/debug/.cargo-lock"
start_fixture "$WT2/$V/target/debug/.cargo-lock" compiler
WT_HOLDER="$FIXTURE_PID"
run_capture "$SCRIPT" status --repo "$REPO"
[[ "$RC" -eq 1 ]] && ok "exit 1 (one worktree is busy)" || bad "exit $RC (wanted 1)"
for path in "$REPO" "$WT2" "$WT3"; do
  case "$OUT" in *"worktree $path"*) ok "worktree $path listed" ;;
    *) bad "worktree $path missing from the report: $OUT" ;; esac
done
case "$OUT" in *"$WT2/$V/target — BUSY"*) ok "the busy worktree's target is marked BUSY" ;;
  *) bad "busy worktree not marked: $OUT" ;; esac
case "$OUT" in *"$REPO/$V/target — free"*) ok "the free worktree's target is marked free" ;;
  *) bad "free worktree not marked: $OUT" ;; esac
case "$OUT" in *"$WT3/$V/target — not built yet"*) ok "a worktree with no target is named" ;;
  *) bad "absent worktree target not named: $OUT" ;; esac
case "$OUT" in *"pid $WT_HOLDER"*) ok "the holder is reported under that worktree" ;;
  *) bad "holder pid missing: $OUT" ;; esac
run_capture "$SCRIPT" status --json --repo "$REPO"
printf '%s' "$OUT" > "$TMP/machine.json"
python3 - "$TMP/machine.json" "$WT_HOLDER" "$WT2" <<'PY' && ok "machine-wide json fields hold" || bad "machine-wide json broken"
import json, sys
d = json.load(open(sys.argv[1]))
pid, wt2 = int(sys.argv[2]), sys.argv[3]
assert d["status"] == "busy" and d["exit_code"] == 1, d
assert d["worktrees"] == 3, d["worktrees"]
by_wt = {t["worktree"]: t for t in d["targets"]}
assert by_wt[wt2]["state"] == "busy", by_wt[wt2]
busy = [t for t in d["targets"] if t["state"] == "busy"]
assert len(busy) == 1, busy
procs = [p for l in busy[0]["locks"] for p in l["processes"] if p["pid"] == pid]
assert procs, f"pid {pid} not reported"
assert procs[0]["label"] == "holder", procs[0]
assert sum(1 for t in d["targets"] if t["state"] == "absent") == 1, d["targets"]
PY
kill "$FIXTURE_PID" 2>/dev/null || true
wait "$FIXTURE_PID" 2>/dev/null || true
run_capture "$SCRIPT" status --repo "$REPO"
[[ "$RC" -eq 0 ]] && ok "exit 0 once the holder is gone" || bad "exit $RC (wanted 0)"
case "$OUT" in *"FREE → exit 0"*) ok "free verdict printed" ;;
  *) bad "no free verdict: $OUT" ;; esac

# --- 8. prune --------------------------------------------------------------
head_ "8. prune deletes only idle copies and never a base target"
OLD_TS="$(python3 -c 'import time; print(time.strftime("%Y%m%d%H%M", time.localtime(time.time()-5*86400)))')"
mkdir -p "$CLONES/stale/nested"
printf 'x\n' > "$CLONES/stale/nested/file"
touch -t "$OLD_TS" "$CLONES/stale/nested/file" "$CLONES/stale/nested" "$CLONES/stale"
mkdir -p "$CLONES/fresh/debug/deps"
printf 'artifact\n' > "$CLONES/fresh/debug/deps/libfixture.rlib"

# A base target that sits inside the copy root: prune must skip it even
# though it is idle past the cutoff.
mkdir -p "$CLONES/base-target/debug"
: > "$CLONES/base-target/debug/.cargo-lock"
printf 'y\n' > "$CLONES/base-target/debug/artifact"
touch -t "$OLD_TS" "$CLONES/base-target/debug/artifact" "$CLONES/base-target/debug" "$CLONES/base-target"

run_capture "$SCRIPT" prune --days 3 --target "$CLONES/base-target" --root "$CLONES"
[[ "$RC" -eq 0 ]] && ok "exit 0" || bad "exit $RC: $ERR"
case "$OUT" in *"pruned stale"*) ok "the idle copy was pruned" ;;
  *) bad "stale copy not pruned: $OUT" ;; esac
[[ ! -d "$CLONES/stale" ]] && ok "the idle copy is gone" || bad "stale copy still present"
[[ -d "$CLONES/fresh" ]] && ok "a recently used copy is kept" || bad "fresh copy was deleted"
[[ -d "$CLONES/base-target" ]] && ok "the base target survived" || bad "prune deleted the base target"
case "$OUT" in *"base target inside root"*) ok "the base target inside the root is named as skipped" ;;
  *) bad "no skip note for the in-root base target: $OUT" ;; esac
case "$OUT" in *"freed ~"*) ok "freed space is reported" ;;
  *) bad "no freed estimate: $OUT" ;; esac

# The copy namespace is the only thing prune reads or writes.
run_capture "$SCRIPT" prune --days 3 --target "$BASE" --root "$TMP/never-created"
[[ "$RC" -eq 0 ]] && ok "a missing namespace is a no-op, exit 0" || bad "exit $RC: $ERR"

# --- 8b. a copy that cannot be removed fails loudly ------------------------
# An immutable file (macOS `chflags uchg`) makes rmtree stop partway. The
# copy must appear in the report as failed — never silently counted as
# pruned or dropped from the totals.
if [[ "$(uname -s)" == "Darwin" ]] && command -v chflags >/dev/null 2>&1; then
  head_ "8b. an undeletable copy is reported and fails the exit code"
  LOCKED="$CLONES/locked"
  mkdir -p "$LOCKED/inner"
  printf 'x\n' > "$LOCKED/inner/held"
  touch -t "$OLD_TS" "$LOCKED/inner/held" "$LOCKED/inner" "$LOCKED"
  chflags uchg "$LOCKED/inner/held"
  run_capture "$SCRIPT" prune --days 3 --target "$BASE" --root "$CLONES"
  [[ "$RC" -ne 0 ]] && ok "exit $RC (a delete failed)" || bad "undeletable copy did not fail the run"
  case "$ERR" in *"could not remove"*) ok "the failure is named on stderr" ;;
    *) bad "no failure message: $ERR" ;; esac
  case "$OUT" in *"failed locked"*) ok "the failed copy is listed in the report" ;;
    *) bad "failed copy missing from the report: $OUT" ;; esac
  case "$OUT" in *"pruned 0 of"*) ok "it is not counted as pruned" ;;
    *) bad "the count includes the failed copy: $OUT" ;; esac
  chflags nouchg "$LOCKED/inner/held" 2>/dev/null || true
  rm -rf "$LOCKED"
fi

# --- 9. run: the memory gate ----------------------------------------------
# `run` passes a free queue straight through; with a build in flight it may
# only start a second one when the host facts clear the gate, and must refuse
# (nothing started, status printed, exit 1) otherwise. It never picks a target
# directory — that is scripts/vendor-cargo.sh's job. Host facts are fed with
# DSB_HOST_* so the boundaries are exact, not timing-dependent.
head_ "9. run — pass-through on a free queue, the memory gate otherwise"

run_capture "$SCRIPT" run --repo "$REPO" -- sh -c 'echo "T=${CARGO_TARGET_DIR:-unset} J=${CARGO_BUILD_JOBS:-unset}"'
[[ "$RC" -eq 0 ]] && ok "exit 0 on a free queue" || bad "exit $RC: $ERR"
case "$OUT" in *"T=unset J=unset"*) ok "the command ran as-is; env untouched" ;;
  *) bad "unexpected env on a free queue: $OUT" ;; esac

start_fixture "$REPO/$V/target/debug/.cargo-lock" plain
GATE_LOCK="$FIXTURE_PID"
run_capture env DSB_HOST_FREE_PERCENT=40 DSB_HOST_LOAD5M=5 DSB_HOST_SWAP_GIB=1 \
  "$SCRIPT" run --repo "$REPO" -- sh -c 'echo "T=${CARGO_TARGET_DIR:-unset} J=${CARGO_BUILD_JOBS:-unset}"'
[[ "$RC" -eq 0 ]] && ok "exit 0 when the gate passes" || bad "exit $RC: $ERR"
case "$OUT" in *"T=unset J=2"*) ok "CARGO_BUILD_JOBS=2 is injected and no target is chosen" ;;
  *) bad "unexpected env on the gated path: $OUT" ;; esac
case "$ERR" in *"memory gate passed"*) ok "the passing gate is announced" ;;
  *) bad "no gate notice: $ERR" ;; esac

# Each threshold, just outside: nothing may start.
for spec in \
  "free 24% < 25%|DSB_HOST_FREE_PERCENT=24 DSB_HOST_LOAD5M=5 DSB_HOST_SWAP_GIB=1" \
  "load5m 18.5 > 18|DSB_HOST_FREE_PERCENT=40 DSB_HOST_LOAD5M=18.5 DSB_HOST_SWAP_GIB=1" \
  "swap 4.5 GiB > 4 GiB|DSB_HOST_FREE_PERCENT=40 DSB_HOST_LOAD5M=5 DSB_HOST_SWAP_GIB=4.5"; do
  want="${spec%%|*}"
  envs="${spec#*|}"
  run_capture env $envs \
    "$SCRIPT" run --repo "$REPO" -- sh -c 'echo SHOULD-NOT-RUN'
  [[ "$RC" -eq 1 ]] && ok "refused ($want)" || bad "not refused for $want (exit $RC)"
  case "$OUT" in *"SHOULD-NOT-RUN"*) bad "the command ran despite: $want" ;;
    *) ok "nothing started ($want)" ;; esac
  case "$OUT" in *"$want"*) ok "the reason names the boundary ($want)" ;;
    *) bad "reason missing for $want: $OUT" ;; esac
done

# Exactly at the thresholds: allowed (>=, <=, <=).
run_capture env DSB_HOST_FREE_PERCENT=25 DSB_HOST_LOAD5M=18 DSB_HOST_SWAP_GIB=4 \
  "$SCRIPT" run --repo "$REPO" -- sh -c 'echo AT-BOUNDARY'
if [[ "$RC" -eq 0 ]] && [[ "$OUT" == *"AT-BOUNDARY"* ]]; then
  ok "the boundary values themselves pass"
else
  bad "boundary refused or did not execute (exit $RC): $ERR"
fi

kill "$FIXTURE_PID" 2>/dev/null || true
wait "$FIXTURE_PID" 2>/dev/null || true

printf '\n%s passed, %s failed\n' "$PASS" "$FAIL"
[[ "$FAIL" -eq 0 ]] || exit 1
