#!/usr/bin/env bash
# Test for scripts/vendor-cargo.sh on a throwaway repository: four path crates
# laid out like the vendored tree (third_party/grok-build, package
# xai-grok-pager-bin, bin xai-grok-pager), real cargo, two git worktrees, a
# fake lock holder, and a stub cargo for the environment contract.
#
# Why (measured 2026-09-27, crates/common/xai-message-delivery-core in this
# repository): with one CARGO_TARGET_DIR shared by two worktrees, worktree B's
# `cargo test` finished in 0.35s and ran worktree A's test binary — cargo names
# a path package's artifacts by its path relative to the workspace root, and
# its freshness is mtime-only. Case 0 pins that hazard; every later case pins a
# part of the wrapper that closes it.
#
# Offline: the fixture has no registry dependencies. It never touches this
# repository's targets or ~/.cache; everything lives in a temp dir.
#
#   0. hazard    — raw cargo, one shared target: B runs A's code
#   1. pin       — the wrapper exports this worktree's target and ignores an
#                  inherited foreign CARGO_TARGET_DIR (stub cargo, no build)
#   2. passthru  — cargo's own `--target <triple>` and `-j` reach cargo intact
#   3. seed      — a cold target is seeded once from a sibling manifest, only
#                  the differing crate compiles, and B's code runs
#                  (the seed itself needs APFS clonefile: macOS only, and the
#                  two assertions that name a seed log skip elsewhere)
#   4. no-build  — `fmt` seeds nothing and passes a busy queue (stub cargo)
#   5. refuse    — a build in flight refuses with exit 1, nothing started
#   6. gate      — --allow-concurrent obeys the memory gate: 2 jobs or refuse
#   7. exempt    — with DSB_EXEMPT=<runner> cargo runs as the DsbExempt runner's
#                  child and is attributed to it; DSB_EXEMPT=0 runs it bare
#                  (macOS only; the runner is built into the temp dir)
#
# Usage: ./scripts/test-vendor-cargo.sh
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WRAPPER="$ROOT/scripts/vendor-cargo.sh"
export PATH="$HOME/.cargo/bin:$PATH"

SEEDABLE=0
[[ "$(uname -s)" == Darwin ]] && SEEDABLE=1

PASS=0
FAIL=0
TMP="$(cd "$(mktemp -d)" && pwd -P)"
FIXTURE_PIDS=()
cleanup() {
  for pid in ${FIXTURE_PIDS[@]+"${FIXTURE_PIDS[@]}"}; do
    kill "$pid" 2>/dev/null || true
  done
  wait 2>/dev/null || true
  rm -rf "$TMP"
}
trap cleanup EXIT

ok()  { printf '  ok   %s\n' "$*"; PASS=$((PASS + 1)); }
bad() { printf '  FAIL %s\n' "$*" >&2; FAIL=$((FAIL + 1)); }
head_() { printf '\n== %s ==\n' "$*"; }

for c in cargo git python3 lsof; do
  command -v "$c" >/dev/null 2>&1 || { echo "skip: $c not on PATH" >&2; exit 0; }
done
[[ -x "$WRAPPER" ]] || { echo "error: $WRAPPER not executable" >&2; exit 1; }
if [[ "$SEEDABLE" -eq 0 ]]; then
  printf 'note: not macOS — the seed assertions are skipped (APFS clonefile)\n' >&2
fi

# --- fixture ---------------------------------------------------------------
A="$TMP/a"
B="$TMP/b"
V=third_party/grok-build
mkdir -p "$A/$V/crates/codegen"
crate() { # name deps... ; lib.rs exposes name()
  local name="$1"; shift
  local dir="$A/$V/crates/codegen/$name"
  mkdir -p "$dir/src"
  {
    printf '[package]\nname = "%s"\nversion = "0.1.0"\nedition = "2021"\n\n[dependencies]\n' "$name"
    for d in "$@"; do printf '%s = { path = "../%s" }\n' "$d" "$d"; done
  } >"$dir/Cargo.toml"
  printf 'pub fn name() -> String { "%s".to_string() }\n' "$name" >"$dir/src/lib.rs"
}
crate core
crate mid core
crate other
mkdir -p "$A/$V/crates/codegen/xai-grok-pager-bin/src"
cat >"$A/$V/crates/codegen/xai-grok-pager-bin/Cargo.toml" <<'EOF'
[package]
name = "xai-grok-pager-bin"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "xai-grok-pager"
path = "src/main.rs"

[dependencies]
core = { path = "../core" }
mid = { path = "../mid" }
other = { path = "../other" }
EOF
cat >"$A/$V/crates/codegen/xai-grok-pager-bin/src/main.rs" <<'EOF'
fn main() { println!("{} {} {}", core::name(), mid::name(), other::name()); }
EOF
cat >"$A/$V/Cargo.toml" <<'EOF'
[workspace]
resolver = "2"
members = ["crates/codegen/*"]
EOF
echo fixture >"$A/$V/SOURCE_REV"
echo "$V/target/" >"$A/.gitignore"
git -C "$A" init -q -b main
git -C "$A" -c user.name=t -c user.email=t@t add -A
git -C "$A" -c user.name=t -c user.email=t@t commit -q -m fixture
git -C "$A" worktree add -q "$B" -b b
# B's own `core`, older than anything A builds: the shape that makes cargo's
# mtime-only freshness pick A's artifact for B (the hazard Case 0 shows).
printf 'pub fn name() -> String { "core-b".to_string() }\n' >"$B/$V/crates/codegen/core/src/lib.rs"
touch -t 200001010000 "$B/$V/crates/codegen/core/src/lib.rs"

# A builds in its own target through dsbdev.sh, so its target records a seed
# manifest — the donor Case 3 seeds B from.
"$ROOT/scripts/dsbdev.sh" --no-run --jobs 2 "$A" >/dev/null 2>"$TMP/dsbdev.log"

# A stub cargo writes the wrapper's environment for assertions without
# building anything.
STUB="$TMP/stub"
mkdir -p "$STUB"
cat >"$STUB/cargo" <<'SH'
#!/usr/bin/env bash
{
  printf 'PWD=%s\n' "$PWD"
  printf 'TARGET=%s\n' "${CARGO_TARGET_DIR:-unset}"
  printf 'JOBS=%s\n' "${CARGO_BUILD_JOBS:-unset}"
  printf 'ARGS=%s\n' "$*"
  printf 'PARENT=%s\n' "$PPID"
  if [[ "$(uname -s)" == Darwin ]]; then
    printf 'RESPONSIBLE=%s\n' "$(/usr/bin/python3 -c 'import ctypes,sys;f=ctypes.CDLL(None).responsibility_get_pid_responsible_for_pid;f.restype=ctypes.c_int;f.argtypes=[ctypes.c_int];print(f(int(sys.argv[1])))' "$$")"
  fi
} >"${STUB_OUT:?}"
exit 0
SH
chmod +x "$STUB/cargo"

run_stub() { # <outfile> <wrapper args...>
  local out="$1"; shift
  RC=0
  env STUB_OUT="$out" PATH="$STUB:$PATH" "$WRAPPER" "$@" >"$TMP/out.txt" 2>"$TMP/err.txt" || RC=$?
  OUT="$(cat "$TMP/out.txt")"
  ERR="$(cat "$TMP/err.txt")"
}

env_field() { # <file> <FIELD>
  sed -n "s/^$2=//p" "$1" | head -1
}

# --- 0. the hazard ---------------------------------------------------------
head_ "0. one shared target: B's build runs A's code"
SHARED="$TMP/shared-target"
(cd "$A/$V" && CARGO_TARGET_DIR="$SHARED" cargo build -q -p xai-grok-pager-bin)
(cd "$B/$V" && CARGO_TARGET_DIR="$SHARED" cargo build -q -p xai-grok-pager-bin)
got="$("$SHARED/debug/xai-grok-pager")"
if [[ "$got" == "core mid other" ]]; then
  ok "B's no-op build produced A's binary (\"$got\") — the hazard this wrapper closes"
else
  bad "the shared target did not reproduce the hazard: B's binary ran \"$got\""
fi

# --- 1. the pin ------------------------------------------------------------
head_ "1. the wrapper pins this worktree's target, ignoring an inherited one"
cd "$B"
export CARGO_TARGET_DIR="$SHARED"   # the old habit: one target for every worktree
run_stub "$TMP/env1" test -p core
unset CARGO_TARGET_DIR
[[ "$RC" -eq 0 ]] && ok "exit 0" || bad "exit $RC: $ERR"
target_field="$(env_field "$TMP/env1" TARGET)"
if [[ "$target_field" == "$B/$V/target" ]]; then ok "CARGO_TARGET_DIR=$target_field (B's own)"; else
  bad "CARGO_TARGET_DIR=$target_field, want $B/$V/target"; fi
case "$ERR" in *"ignoring inherited CARGO_TARGET_DIR=$SHARED"*) ok "the foreign value is named on stderr" ;;
  *) bad "no notice about the inherited value: $ERR" ;; esac
[[ "$(env_field "$TMP/env1" PWD)" == "$B/$V" ]] && ok "cargo runs from the vendored tree" \
  || bad "cargo ran from $(env_field "$TMP/env1" PWD)"
[[ "$(env_field "$TMP/env1" JOBS)" == "4" ]] && ok "CARGO_BUILD_JOBS=4 by default" \
  || bad "CARGO_BUILD_JOBS=$(env_field "$TMP/env1" JOBS), want 4"
case "$ERR" in *"test → target"*) ok "the wrapper says what it runs and where" ;;
  *) bad "no run line on stderr: $ERR" ;; esac

# Seed bookkeeping: case 1 must have seen a cold target and seeded it once.
case "$ERR" in *"cold target — seeding"*) ok "the cold target was seeded once" ;;
  *) bad "no seed line: $ERR" ;; esac
if [[ "$SEEDABLE" -eq 1 ]]; then
  case "$ERR" in *"seeded"*) ok "the seed reports its units" ;;
    *) bad "seed summary missing: $ERR" ;; esac
else
  printf '  skip the seed summary (needs macOS clonefile)\n'
fi

# --- 2. cargo's own flags survive -----------------------------------------
head_ "2. cargo's --target <triple> and -j pass through"
run_stub "$TMP/env2" --target aarch64-apple-darwin test -p core -j 8
[[ "$RC" -eq 0 ]] && ok "exit 0" || bad "exit $RC: $ERR"
[[ "$(env_field "$TMP/env2" ARGS)" == "--target aarch64-apple-darwin test -p core -j 8" ]] \
  && ok "argv reaches cargo unchanged" || bad "argv: $(env_field "$TMP/env2" ARGS)"
[[ "$(env_field "$TMP/env2" TARGET)" == "$B/$V/target" ]] \
  && ok "the wrapper still pinned the target directory" \
  || bad "target directory changed: $(env_field "$TMP/env2" TARGET)"

# --- 3. seed, then B's own code runs --------------------------------------
head_ "3. a cold target is seeded; only the differing crate compiles"
rm -rf "$B/$V/target"
rc=0
(cd "$B/$V" && "$WRAPPER" build -p xai-grok-pager-bin) >"$TMP/build.log" 2>&1 || rc=$?
[[ "$rc" -eq 0 ]] && ok "the wrapper built the fixture" || { bad "build exit $rc"; tail -20 "$TMP/build.log" >&2; }
got="$("$B/$V/target/debug/xai-grok-pager")"
if [[ "$got" == "core-b mid other" ]]; then ok "B's binary runs B's code (\"$got\")"
else bad "B's binary ran \"$got\" — the seed handed it the donor's code"; fi
case "$(cat "$TMP/build.log")" in
  *"cold target — seeding"*) ok "the first build seeded the cold target" ;;
  *) bad "no seed on the first build" ;;
esac
if [[ "$SEEDABLE" -eq 1 ]]; then
  case "$(cat "$TMP/build.log")" in
    *"seeded 4 units from"*) ok "the donor manifest supplied its 4 units" ;;
    *) bad "seed summary: $(grep -m1 seeded "$TMP/build.log" || echo missing)" ;;
  esac
else
  printf '  skip the seed summary (needs macOS clonefile)\n'
fi
if compgen -G "$B/$V/target/debug/.fingerprint/xai-grok-pager-bin-*" >/dev/null; then
  ok "B built in its own target"
else bad "B's target has no fingerprint for the fixture package"; fi

head_ "3b. a warm target is not seeded again"
rc=0
(cd "$B/$V" && "$WRAPPER" build -p xai-grok-pager-bin) >"$TMP/build2.log" 2>&1 || rc=$?
[[ "$rc" -eq 0 ]] && ok "exit 0" || bad "exit $rc"
case "$(cat "$TMP/build2.log")" in
  *"cold target"*) bad "a warm target was seeded again" ;;
  *) ok "no second seed" ;;
esac

# --- 4. subcommands that compile nothing ----------------------------------
# A lock holder in A's target makes the queue busy machine-wide.
mkdir -p "$A/$V/target/debug"
: >"$A/$V/target/debug/.cargo-lock"
FAKEBIN="$TMP/fakebin"
mkdir -p "$FAKEBIN"
ln -sf "$(command -v sleep)" "$FAKEBIN/rustc"
cat >"$TMP/fx-holder.py" <<'PY'
import fcntl, os, subprocess, sys, time
lock, fakebin = sys.argv[1], sys.argv[2]
f = open(lock, "rb")
fcntl.flock(f, fcntl.LOCK_EX)
child = subprocess.Popen([os.path.join(fakebin, "rustc"), "120"])
print("holding", os.getpid(), "child", child.pid, flush=True)
time.sleep(240)
PY
(cd "$A" && exec python3 "$TMP/fx-holder.py" "$A/$V/target/debug/.cargo-lock" "$FAKEBIN") \
  >"$TMP/fx.out" 2>&1 &
HOLDER=$!
FIXTURE_PIDS+=("$HOLDER")
for _ in $(seq 1 100); do grep -q holding "$TMP/fx.out" 2>/dev/null && break; sleep 0.1; done

head_ "4. fmt seeds nothing and passes a busy queue"
run_stub "$TMP/env4" fmt --all -- --check
[[ "$RC" -eq 0 ]] && ok "exit 0 while a build is in flight" || bad "exit $RC: $ERR"
[[ "$(env_field "$TMP/env4" ARGS)" == "fmt --all -- --check" ]] \
  && ok "argv reaches cargo unchanged" || bad "argv: $(env_field "$TMP/env4" ARGS)"
case "$ERR" in *"cold target"*) bad "fmt tried to seed" ;; *) ok "no seed for a non-building subcommand" ;; esac
case "$ERR" in *"in flight"*) bad "a non-building subcommand was queue-blocked" ;;
  *) ok "no queue refusal for a non-building subcommand" ;; esac

# --- 5. refuse while a build is in flight ---------------------------------
head_ "5. a build in flight refuses (exit 1), nothing started"
run_stub "$TMP/env5" test -p core
[[ "$RC" -eq 1 ]] && ok "exit 1" || bad "exit $RC (wanted 1)"
case "$OUT$ERR" in *"a vendored build is in flight and one runs at a time"*) ok "the refusal names the reason" ;;
  *) bad "refusal reason missing: $OUT$ERR" ;; esac
[[ ! -f "$TMP/env5" ]] && ok "cargo never ran" || bad "cargo ran despite the refusal"
case "$OUT" in *"worktree $A"*) ok "the busy worktree is named" ;;
  *) bad "busy worktree not named: $OUT" ;; esac

# --- 6. --allow-concurrent obeys the memory gate --------------------------
head_ "6. --allow-concurrent: through the gate at 2 jobs, or refused"
run_stub_with() { # <outfile> <env assignments...> -- <wrapper args...>
  local out="$1"; shift
  local envs=(STUB_OUT="$out" PATH="$STUB:$PATH")
  while [[ "$1" != "--" ]]; do envs+=("$1"); shift; done
  shift
  RC=0
  env "${envs[@]}" "$WRAPPER" "$@" >"$TMP/out.txt" 2>"$TMP/err.txt" || RC=$?
  OUT="$(cat "$TMP/out.txt")"
  ERR="$(cat "$TMP/err.txt")"
}
run_stub_with "$TMP/env6" DSB_HOST_FREE_PERCENT=40 DSB_HOST_LOAD5M=5 DSB_HOST_SWAP_GIB=1 -- \
  --allow-concurrent test -p core
[[ "$RC" -eq 0 ]] && ok "exit 0 when the gate passes" || bad "exit $RC: $ERR"
[[ "$(env_field "$TMP/env6" JOBS)" == "2" ]] && ok "CARGO_BUILD_JOBS=2 for the second build" \
  || bad "CARGO_BUILD_JOBS=$(env_field "$TMP/env6" JOBS), want 2"
case "$ERR" in *"memory gate passed"*) ok "the passing gate is announced" ;;
  *) bad "no gate notice: $ERR" ;; esac
[[ "$(env_field "$TMP/env6" TARGET)" == "$B/$V/target" ]] && ok "the target is still B's own" \
  || bad "target changed on the concurrent path: $(env_field "$TMP/env6" TARGET)"

run_stub_with "$TMP/env6b" DSB_HOST_FREE_PERCENT=10 DSB_HOST_LOAD5M=5 DSB_HOST_SWAP_GIB=1 -- \
  --allow-concurrent test -p core
[[ "$RC" -eq 1 ]] && ok "exit 1 when the gate denies" || bad "exit $RC (wanted 1)"
[[ ! -f "$TMP/env6b" ]] && ok "cargo never ran under a denied gate" || bad "cargo ran despite the denial"
case "$ERR" in *"memory gate denies a second build"*) ok "the denial names the gate" ;;
  *) bad "denial reason missing: $ERR" ;; esac

# --- 7. the DsbExempt runner ---------------------------------------------
head_ "7. DSB_EXEMPT: cargo runs under the runner, or bare with 0"
if [[ "$(uname -s)" == Darwin ]]; then
  "$ROOT/scripts/install-dsb-exempt.sh" --app-dir "$TMP/apps" --bin-dir "$TMP/runner-bin" >/dev/null 2>&1
  RUNNER="$TMP/runner-bin/dsb-exempt"
  run_stub_with "$TMP/env7" DSB_EXEMPT="$RUNNER" -- fmt --all -- --check
  [[ "$RC" -eq 0 ]] && ok "exit 0 under the runner" || bad "exit $RC: $ERR"
  parent="$(env_field "$TMP/env7" PARENT)"
  [[ "$(env_field "$TMP/env7" RESPONSIBLE)" == "$parent" ]] \
    && ok "cargo is attributed to its parent, the runner (pid $parent)" \
    || bad "cargo responsible $(env_field "$TMP/env7" RESPONSIBLE), parent $parent"
  case "$ERR" in *"under DsbExempt"*) ok "the runner is announced" ;; *) bad "no runner notice: $ERR" ;; esac
  run_stub_with "$TMP/env7b" DSB_EXEMPT=0 -- fmt --all -- --check
  [[ "$(env_field "$TMP/env7b" RESPONSIBLE)" != "$(env_field "$TMP/env7b" PARENT)" ]] \
    && ok "DSB_EXEMPT=0: cargo is not attributed to a runner" || bad "DSB_EXEMPT=0 still ran under a runner"
  case "$ERR" in *"under DsbExempt"*) bad "DSB_EXEMPT=0 still announced the runner" ;; *) ok "no runner notice with DSB_EXEMPT=0" ;; esac
else
  ok "skipped: the runner is macOS only"
fi

printf '\n%d passed, %d failed\n' "$PASS" "$FAIL"
[[ "$FAIL" -eq 0 ]] || exit 1
