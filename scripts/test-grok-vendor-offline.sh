#!/usr/bin/env bash
# T1 — Grok vendor offline tests (tiered; default is LIGHT).
#
# Levels (PRE3X_VENDOR_LEVEL or --light|--medium|--full):
#   light   (default)  sampler + sampling-types + config*  — DeepSeek path signal, smallish disk
#   medium             + tools + test-support              — still no shell mega-graph
#   full               + shell --lib + sampling_client     — cold can be 20–40GB target/; rare
#
# After a heavy run:  rm -rf third_party/grok-build/target
set -euo pipefail
# shellcheck source=lib/common.sh
source "$(cd "$(dirname "$0")" && pwd)/lib/common.sh"

LEVEL="${PRE3X_VENDOR_LEVEL:-light}"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --light) LEVEL=light; shift ;;
    --medium) LEVEL=medium; shift ;;
    --full) LEVEL=full; shift ;;
    -h|--help)
      cat <<'EOF'
Usage: test-grok-vendor-offline.sh [--light|--medium|--full]

  --light   default: sampler, sampling-types, config, config-types
  --medium  + xai-grok-tools + xai-grok-test-support
  --full    + xai-grok-shell (--lib) + sampling_client integration
            WARNING: cold compile can use tens of GB under
            third_party/grok-build/target — clean after if disk is tight

Env:
  PRE3X_VENDOR_LEVEL=light|medium|full
  PRE3X_SKIP_VENDOR_CHECK=1   skip cargo check -p xai-grok-pager-bin (default skip on light)
  PRE3X_TEST_THREADS=4
EOF
      exit 0
      ;;
    *) fail "unknown arg: $1" ;;
  esac
done

RESULTS="${PRE3X_RESULTS:-}"
record() {
  [[ -n "$RESULTS" ]] || return 0
  record_result "$RESULTS" "$1" "$2" "${3:-}"
}

VENDOR="$ROOT/third_party/grok-build"
[[ -d "$VENDOR" ]] || fail "missing vendor tree $VENDOR"
[[ -f "$VENDOR/SOURCE_REV" ]] || fail "missing SOURCE_REV"

export PATH="${HOME}/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:${PATH}"

log "== T1 Grok vendor offline (level=$LEVEL) =="
log "SOURCE_REV=$(tr -d '\n' <"$VENDOR/SOURCE_REV")"
log "Disk tip: vendor artifacts land in third_party/grok-build/target (gitignored)."

if ! command -v cargo >/dev/null; then
  fail "cargo not found"
fi
if ! command -v protoc >/dev/null && ! command -v dotslash >/dev/null; then
  fail "need protoc or dotslash for vendor builds"
fi

cd "$VENDOR"

THREADS="${PRE3X_TEST_THREADS:-4}"
FAILED=0

# Light/medium: skip pager-bin check (pulls huge graph). Full may still skip unless forced.
SKIP_CHECK="${PRE3X_SKIP_VENDOR_CHECK:-}"
if [[ -z "$SKIP_CHECK" ]]; then
  if [[ "$LEVEL" == "light" || "$LEVEL" == "medium" ]]; then
    SKIP_CHECK=1
  else
    SKIP_CHECK=0
  fi
fi

if [[ "$SKIP_CHECK" != "1" ]]; then
  log "T1.0 cargo check -p xai-grok-pager-bin"
  if cargo check -p xai-grok-pager-bin; then
    record T1.0 PASS
    ok "T1.0 pager-bin check"
  else
    record T1.0 FAIL
    fail "T1.0 cargo check -p xai-grok-pager-bin"
  fi
else
  record T1.0 SKIP "level=$LEVEL (set PRE3X_SKIP_VENDOR_CHECK=0 to force)"
  skip "T1.0 pager-bin check skipped (light/medium default)"
fi

run_crate() {
  local pkg="$1"
  local id="$2"
  shift 2
  log "T1 $id cargo test -p $pkg $*"
  if cargo test -p "$pkg" -- --test-threads="$THREADS" "$@"; then
    record "$id" PASS "$pkg"
    ok "$id $pkg"
  else
    record "$id" FAIL "$pkg"
    warn "$id FAILED: $pkg"
    FAILED=1
  fi
}

# --- light core (DeepSeek chat_completions path) ---
run_crate xai-grok-sampler T1.1
run_crate xai-grok-sampling-types T1.2
run_crate xai-grok-config T1.4
run_crate xai-grok-config-types T1.5

if [[ "$LEVEL" == "light" ]]; then
  record T1.3 SKIP "medium+"
  record T1.6 SKIP "medium+"
  record T1.7 SKIP "full only"
  record T1.8 SKIP "full only"
  log "light level done (sampler/types/config only)"
else
  run_crate xai-grok-tools T1.3
  run_crate xai-grok-test-support T1.6
fi

if [[ "$LEVEL" == "full" ]]; then
  log "T1.7 xai-grok-shell --lib (HEAVY — large target/)"
  if cargo test -p xai-grok-shell --lib -- --test-threads="$THREADS"; then
    record T1.7 PASS "shell --lib"
    ok "T1.7 shell --lib"
  else
    record T1.7 FAIL "shell --lib"
    FAILED=1
    warn "T1.7 shell --lib FAILED"
  fi

  log "T1.8 shell integration: test_sampling_client"
  if cargo test -p xai-grok-shell --test test_sampling_client -- --test-threads="$THREADS"; then
    record T1.8 PASS
    ok "T1.8 sampling_client"
  else
    record T1.8 FAIL
    FAILED=1
    warn "T1.8 sampling_client FAILED"
  fi
else
  if [[ "$LEVEL" == "medium" ]]; then
    record T1.7 SKIP "full only"
    record T1.8 SKIP "full only"
  fi
fi

cd "$ROOT"

if [[ "$FAILED" -ne 0 ]]; then
  fail "T1 vendor suite had failures (level=$LEVEL)"
fi
ok "vendor offline level=$LEVEL ALL PASSED"
if [[ "$LEVEL" == "full" ]]; then
  warn "full level may leave a large third_party/grok-build/target — clean when done:"
  warn "  rm -rf third_party/grok-build/target"
fi
