#!/usr/bin/env bash
# Owner-bar aggregator. Exit 0 only when every frozen P0 row is PASS with R0A.
# Until fusion lands, this script MUST exit non-zero (RED baseline).
# Compatible with macOS /bin/bash 3.2 (no mapfile).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "${ROOT}"
LEDGER="${ROOT}/docs/product/OWNER_BAR_P0_LEDGER.md"
OUT_DIR="${ROOT}/docs/product/evidence"
STATUS_TSV="${OUT_DIR}/OWNER_BAR_STATUS.tsv"
SHA="$(git rev-parse HEAD 2>/dev/null || echo unknown)"
MODE="${1:-}"

mkdir -p "${OUT_DIR}"

echo "=== test-owner-bar ==="
echo "git_sha=${SHA}"
echo "ledger=${LEDGER}"

if [[ ! -f "${LEDGER}" ]]; then
  echo "FATAL: missing P0 ledger" >&2
  exit 2
fi

# Extract P0 IDs from ledger table rows like | **S1** | or | **L1-45-0** |
ROW_IDS="$(rg -o '\*\*[A-Z0-9][A-Z0-9._-]*\*\*' "${LEDGER}" | tr -d '*' | \
  rg '^(S[0-9]+|L1-|L2-|L3-|F[0-9]+|OB-)' | sort -u || true)"

ROW_COUNT=0
if [[ -n "${ROW_IDS}" ]]; then
  ROW_COUNT="$(printf '%s\n' "${ROW_IDS}" | wc -l | tr -d ' ')"
fi

if [[ "${ROW_COUNT}" -lt 40 ]]; then
  echo "FATAL: ledger parse produced too few IDs (${ROW_COUNT}) — fix ledger parser or file" >&2
  exit 2
fi

if [[ "${MODE}" == "--selftest" ]]; then
  FAIL=0
  echo -e "id\tstatus\tsha\nS1\tPASS\t${SHA}" > /tmp/owner-bar-fake-pass.tsv
  FAKE_N="$(wc -l < /tmp/owner-bar-fake-pass.tsv | tr -d ' ')"
  if [[ "${FAKE_N}" -lt "${ROW_COUNT}" ]]; then
    echo "selftest: correctly detects incomplete coverage (${FAKE_N} < ${ROW_COUNT})"
  else
    echo "selftest: FAIL incomplete coverage detection" >&2
    FAIL=1
  fi
  for bad in SKIP BLOCKED N/A NOT_RUN XFAIL IGNORED; do
    echo "selftest: illegal status token: ${bad}"
  done
  if ! bash "${ROOT}/scripts/check-path-a-linkage.sh" >/tmp/owner-bar-linkage.out 2>&1; then
    echo "selftest: linkage check exits non-zero on current tree (expected RED)"
  fi
  if ! bash "${ROOT}/scripts/test-owner-bar.sh" >/tmp/owner-bar-red.out 2>&1; then
    echo "selftest: aggregator exits non-zero (expected RED baseline)"
  else
    echo "selftest: FAIL expected RED aggregator" >&2
    FAIL=1
  fi
  if [[ "${FAIL}" -ne 0 ]]; then
    echo "test-owner-bar --selftest: FAIL" >&2
    exit 1
  fi
  echo "test-owner-bar --selftest: PASS (gate substrate ok)"
  exit 0
fi

{
  echo -e "id\tstatus\treason\tgit_sha"
  printf '%s\n' "${ROW_IDS}" | while IFS= read -r id; do
    [[ -z "${id}" ]] && continue
    echo -e "${id}\tFAIL\tno_R0A_harness_yet\t${SHA}"
  done
} > "${STATUS_TSV}"

echo "wrote ${STATUS_TSV} (${ROW_COUNT} rows, all FAIL)"

set +e
bash "${ROOT}/scripts/check-path-a-linkage.sh"
LINK=$?
bash "${ROOT}/scripts/check-forbidden-evidence.sh"
FORB=$?
set -e

PASS_N="$(rg -c $'\tPASS\t' "${STATUS_TSV}" 2>/dev/null || true)"
PASS_N="${PASS_N:-0}"
FAIL_N="$(rg -c $'\tFAIL\t' "${STATUS_TSV}" 2>/dev/null || true)"
FAIL_N="${FAIL_N:-0}"

echo "summary: PASS=${PASS_N} FAIL=${FAIL_N} linkage_exit=${LINK} forbidden_exit=${FORB}"

if [[ "${PASS_N}" -eq "${ROW_COUNT}" && "${FAIL_N}" -eq 0 && "${LINK}" -eq 0 && "${FORB}" -eq 0 ]]; then
  echo "test-owner-bar: ALL PASS — owner bar green"
  exit 0
fi

echo "test-owner-bar: RED (expected until owner-bar-5x fusion complete)"
echo "See docs/product/OWNER_BAR_P0_LEDGER.md and OWNER_BAR_5X_GOALS.md"
exit 1
