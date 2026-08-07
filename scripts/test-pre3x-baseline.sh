#!/usr/bin/env bash
# Pre-3.0.0 baseline orchestrator. See docs/product/PRE_3X_TEST_MATRIX.md.
set -euo pipefail
# shellcheck source=lib/common.sh
source "$(cd "$(dirname "$0")" && pwd)/lib/common.sh"

DO_VENDOR=0
DO_LIVE=0
DO_EXTENDED=0
DO_ALL=0
VENDOR_LEVEL="${PRE3X_VENDOR_LEVEL:-light}"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --vendor) DO_VENDOR=1; shift ;;
    --vendor-medium) DO_VENDOR=1; VENDOR_LEVEL=medium; shift ;;
    --vendor-full) DO_VENDOR=1; VENDOR_LEVEL=full; shift ;;
    --live) DO_LIVE=1; shift ;;
    --extended) DO_EXTENDED=1; DO_LIVE=1; shift ;;
    --all) DO_ALL=1; shift ;;
    -h|--help)
      cat <<'EOF'
Usage: test-pre3x-baseline.sh [options]

  (default)          T0 product offline + T2 entry  (fast, everyday)
  --live             + T3/T4 DeepSeek live (API key; agent feature smoke)
  --vendor           + T1 vendor light (sampler/config only)
  --vendor-medium    + T1 medium (+ tools)
  --vendor-full      + T1 full (+ shell) — HEAVY disk; clean target after
  --extended         live + T5 optional cases
  --all              light vendor + live (still not vendor-full)

Everyday recommendation:
  ./scripts/test-pre3x-baseline.sh --live

Disk: vendor artifacts are third_party/grok-build/target (gitignored).
After --vendor-full:  rm -rf third_party/grok-build/target

Writes docs/product/evidence/_last_pre3x_results.tsv
EOF
      exit 0
      ;;
    *) fail "unknown arg: $1" ;;
  esac
done

if [[ "$DO_ALL" -eq 1 ]]; then
  DO_VENDOR=1
  DO_LIVE=1
  # --all stays light vendor; never auto full (disk safety)
  if [[ "$VENDOR_LEVEL" == "full" ]]; then
    : # honor explicit --vendor-full before --all
  else
    VENDOR_LEVEL=light
  fi
fi
export PRE3X_VENDOR_LEVEL="$VENDOR_LEVEL"

EVIDENCE_DIR="$ROOT/docs/product/evidence"
mkdir -p "$EVIDENCE_DIR"
RESULTS="$EVIDENCE_DIR/_last_pre3x_results.tsv"
: >"$RESULTS"
export PRE3X_RESULTS="$RESULTS"

FAILED=0
run_step() {
  local name="$1"
  shift
  log ">>> $name"
  if "$@"; then
    ok "step $name"
  else
    warn "step $name FAILED"
    FAILED=1
  fi
}

run_step T0_T2 ./scripts/test-product-offline.sh

if [[ "$DO_VENDOR" -eq 1 ]]; then
  log "T1 vendor level=$VENDOR_LEVEL"
  run_step T1 ./scripts/test-grok-vendor-offline.sh "--$VENDOR_LEVEL"
else
  log "T1 vendor skipped (pass --vendor / --vendor-medium / --vendor-full)"
  record_result "$RESULTS" T1 SKIP "not requested"
fi

if [[ "$DO_LIVE" -eq 1 ]]; then
  if load_deepseek_key; then
    if [[ "$DO_EXTENDED" -eq 1 ]]; then
      run_step T3_T4_T5 ./scripts/test-deepseek-live.sh --extended
    else
      run_step T3_T4 ./scripts/test-deepseek-live.sh
    fi
  else
    warn "live requested but no API key — recording SKIP"
    record_result "$RESULTS" T3 SKIP "no API key"
    record_result "$RESULTS" T4 SKIP "no API key"
  fi
else
  log "T3/T4 live skipped (pass --live or --all)"
  record_result "$RESULTS" T3 SKIP "not requested"
  record_result "$RESULTS" T4 SKIP "not requested"
fi

log "== Summary ($RESULTS) =="
if [[ -s "$RESULTS" ]]; then
  column -t -s $'\t' "$RESULTS" 2>/dev/null || cat "$RESULTS"
  pass_n="$(rg -c $'\tPASS\t' "$RESULTS" || true)"
  fail_n="$(rg -c $'\tFAIL\t' "$RESULTS" || true)"
  skip_n="$(rg -c $'\tSKIP\t' "$RESULTS" || true)"
  log "PASS=${pass_n:-0} FAIL=${fail_n:-0} SKIP=${skip_n:-0}"
fi

if [[ "$FAILED" -ne 0 ]] || rg -q $'\tFAIL\t' "$RESULTS"; then
  fail "pre-3.x baseline has failures — see $RESULTS and PRE_3X_TEST_MATRIX.md"
fi
ok "pre-3.x baseline complete"
log "Next: write docs/product/evidence/PRE3X_BASELINE_$(date -u +%Y-%m-%d).md from this run"
