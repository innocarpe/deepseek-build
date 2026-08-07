#!/usr/bin/env bash
# Owner-bar aggregator. Exit 0 only when every frozen P0 row is PASS with R0A.
# Compatible with macOS /bin/bash 3.2 (no mapfile).
#
# Status source (cut): docs/product/evidence/OWNER_BAR_PASS_MAP.tsv
#   id<TAB>status<TAB>reason
# Until the map covers every ledger row with PASS, STATUS is FAIL and exit 1.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "${ROOT}"
LEDGER="${ROOT}/docs/product/OWNER_BAR_P0_LEDGER.md"
OUT_DIR="${ROOT}/docs/product/evidence"
STATUS_TSV="${OUT_DIR}/OWNER_BAR_STATUS.tsv"
PASS_MAP="${OUT_DIR}/OWNER_BAR_PASS_MAP.tsv"
SHA="$(git rev-parse HEAD 2>/dev/null || echo unknown)"
MODE="${1:-}"

mkdir -p "${OUT_DIR}"

echo "=== test-owner-bar ==="
echo "git_sha=${SHA}"
echo "ledger=${LEDGER}"
echo "pass_map=${PASS_MAP}"

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
  # Linkage must run (exit 0 or 1 both ok; must not crash)
  set +e
  bash "${ROOT}/scripts/check-path-a-linkage.sh" >/tmp/owner-bar-linkage.out 2>&1
  LINK_EC=$?
  set -e
  if [[ "${LINK_EC}" -gt 1 ]]; then
    echo "selftest: FAIL linkage crashed exit=${LINK_EC}" >&2
    FAIL=1
  else
    echo "selftest: linkage runnable exit=${LINK_EC}"
  fi
  # Aggregator must run; green or red both prove substrate
  set +e
  bash "${ROOT}/scripts/test-owner-bar.sh" >/tmp/owner-bar-agg.out 2>&1
  AGG_EC=$?
  set -e
  if [[ "${AGG_EC}" -gt 1 ]]; then
    echo "selftest: FAIL aggregator crash exit=${AGG_EC}" >&2
    FAIL=1
  else
    echo "selftest: aggregator runnable exit=${AGG_EC} (0=green 1=red)"
  fi
  if [[ "${FAIL}" -ne 0 ]]; then
    echo "test-owner-bar --selftest: FAIL" >&2
    exit 1
  fi
  echo "test-owner-bar --selftest: PASS (gate substrate ok)"
  exit 0
fi

# Load pass map if present
declare -a MAP_IDS=()
declare -a MAP_STATUS=()
declare -a MAP_REASON=()
if [[ -f "${PASS_MAP}" ]]; then
  while IFS=$'\t' read -r mid mst mreason || [[ -n "${mid:-}" ]]; do
    [[ -z "${mid:-}" || "${mid}" == "id" ]] && continue
    # reject illegal statuses
    case "${mst}" in
      PASS|FAIL|NOT_RUN) ;;
      *)
        echo "FATAL: illegal status '${mst}' for ${mid} in pass map" >&2
        exit 2
        ;;
    esac
    MAP_IDS+=("${mid}")
    MAP_STATUS+=("${mst}")
    MAP_REASON+=("${mreason:-}")
  done < "${PASS_MAP}"
fi

lookup_status() {
  local want="$1" i
  for i in "${!MAP_IDS[@]}"; do
    if [[ "${MAP_IDS[$i]}" == "${want}" ]]; then
      echo "${MAP_STATUS[$i]}"$'\t'"${MAP_REASON[$i]}"
      return 0
    fi
  done
  echo $'FAIL\tno_map_entry'
  return 0
}

{
  echo -e "id\tstatus\treason\tgit_sha"
  # Avoid pipe-subshell so MAP_* arrays stay visible (bash 3.2).
  while IFS= read -r id; do
    [[ -z "${id}" ]] && continue
    line="$(lookup_status "${id}")"
    st="${line%%$'\t'*}"
    reason="${line#*$'\t'}"
    echo -e "${id}\t${st}\t${reason}\t${SHA}"
  done <<EOF
${ROW_IDS}
EOF
} > "${STATUS_TSV}"

echo "wrote ${STATUS_TSV} (${ROW_COUNT} rows from ledger + pass map)"

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
NOT_RUN_N="$(rg -c $'\tNOT_RUN\t' "${STATUS_TSV}" 2>/dev/null || true)"
NOT_RUN_N="${NOT_RUN_N:-0}"

echo "summary: PASS=${PASS_N} FAIL=${FAIL_N} NOT_RUN=${NOT_RUN_N} linkage_exit=${LINK} forbidden_exit=${FORB} rows=${ROW_COUNT}"

if [[ "${PASS_N}" -eq "${ROW_COUNT}" && "${FAIL_N}" -eq 0 && "${NOT_RUN_N}" -eq 0 && "${LINK}" -eq 0 && "${FORB}" -eq 0 ]]; then
  echo "test-owner-bar: ALL PASS — owner bar green"
  exit 0
fi

echo "test-owner-bar: RED (expected until owner-bar-5x fusion complete)"
echo "See docs/product/OWNER_BAR_P0_LEDGER.md and OWNER_BAR_5X_GOALS.md"
if [[ "${FAIL_N}" -gt 0 || "${NOT_RUN_N}" -gt 0 ]]; then
  echo "FAIL/NOT_RUN sample:" >&2
  rg $'\t(FAIL|NOT_RUN)\t' "${STATUS_TSV}" | head -20 >&2 || true
fi
exit 1
