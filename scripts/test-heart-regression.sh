#!/usr/bin/env bash
# Heart regression under L3 (owner-bar G010 / 5x-H3-1).
#
# Re-runs L1/L2 unit hearts + Path A linkage after L3 stamps land.
# Does NOT require a live DeepSeek key.
#
# Usage:
#   ./scripts/test-heart-regression.sh
#   ./scripts/test-heart-regression.sh --with-e2e   # also public-entry e2e
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "${ROOT}"
# shellcheck source=lib/common.sh
source "${ROOT}/scripts/lib/common.sh"

WITH_E2E=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --with-e2e) WITH_E2E=1; shift ;;
    -h|--help)
      sed -n '2,12p' "$0"
      exit 0
      ;;
    *) fail "unknown arg: $1" ;;
  esac
done

OUT_DIR="${ROOT}/docs/product/evidence"
mkdir -p "${OUT_DIR}"
RESULTS="${OUT_DIR}/PATH_A_R0_G010_HEART_REGRESSION_last.tsv"
: >"${RESULTS}"
record() {
  printf '%s\t%s\t%s\n' "$1" "$2" "${3:-}" >>"${RESULTS}"
}

# Avoid sccache + CARGO_INCREMENTAL fights on this machine.
export CARGO_INCREMENTAL=0
unset RUSTC_WRAPPER || true
CARGO=(cargo --config 'build.rustc-wrapper=""')

FAILED=0
mark() {
  local id="$1" st="$2" note="${3:-}"
  record "${id}" "${st}" "${note}"
  if [[ "${st}" == "FAIL" ]]; then
    FAILED=1
    warn "${id}: FAIL ${note}"
  else
    ok "${id}: ${st} ${note}"
  fi
}

echo "=== test-heart-regression (G010) ==="
echo "git_sha=$(git rev-parse HEAD 2>/dev/null || echo unknown)"

# Linkage
if bash "${ROOT}/scripts/check-path-a-linkage.sh"; then
  mark LINKAGE PASS
else
  mark LINKAGE FAIL
fi

run_test() {
  local id="$1"
  shift
  local out
  set +e
  out="$("${CARGO[@]}" test "$@" -- --nocapture 2>&1)"
  local ec=$?
  set -e
  if [[ "${ec}" -eq 0 ]]; then
    mark "${id}" PASS
  else
    mark "${id}" FAIL "exit=${ec}"
    printf '%s\n' "${out}" | tail -30 >&2
  fi
}

run_test CTX_PATH_A -p dsb-context path_a
run_test CTX_SKILLS -p dsb-context skills
run_test AGENT_PATH_A -p dsb-agent path_a
run_test AGENT_ROUTING -p dsb-agent routing
run_test AGENT_PARALLEL -p dsb-agent parallel
run_test AGENT_SUBAGENT -p dsb-agent subagent
run_test CLI_STAMPS -p dsb-cli stamp_path_a

# Offline L3 smoke (CLI surface + worktree help)
if bash "${ROOT}/scripts/test-l3-smoke.sh" --offline-only; then
  mark L3_SMOKE_OFFLINE PASS
else
  mark L3_SMOKE_OFFLINE FAIL
fi

if [[ "${WITH_E2E}" -eq 1 ]]; then
  if bash "${ROOT}/scripts/test-path-a-public-entry-e2e.sh"; then
    mark PATH_A_E2E PASS
  else
    mark PATH_A_E2E FAIL
  fi
else
  mark PATH_A_E2E SKIP "pass --with-e2e to run"
fi

echo "--- results ${RESULTS} ---"
column -t -s $'\t' "${RESULTS}" 2>/dev/null || cat "${RESULTS}"

if [[ "${FAILED}" -ne 0 ]]; then
  echo "test-heart-regression: FAIL" >&2
  exit 1
fi
echo "test-heart-regression: PASS"
exit 0
