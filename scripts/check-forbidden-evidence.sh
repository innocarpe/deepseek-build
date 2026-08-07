#!/usr/bin/env bash
# Fail if *active* owner-bar / 5.x cut evidence uses forbidden sole-proof patterns.
# Historical 3.x/4.x CUT_* files are out of scope (already known false) unless they claim 5.0.0.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
EV="${ROOT}/docs/product/evidence"
FAIL=0

if [[ ! -d "${EV}" ]]; then
  echo "check-forbidden-evidence: no evidence dir (ok)"
  exit 0
fi

# Active scope only
SCOPE_FILES=()
while IFS= read -r f; do
  SCOPE_FILES+=("$f")
done < <(find "${EV}" -type f \( \
  -name 'OWNER_BAR*' -o -name 'CUT_5*' -o -name 'SHIP_5*' -o -path '*/owner-bar/*' \
  \) 2>/dev/null || true)

if [[ "${#SCOPE_FILES[@]}" -eq 0 ]]; then
  echo "check-forbidden-evidence: no active owner-bar evidence files yet (ok)"
  exit 0
fi

echo "check-forbidden-evidence: scanning ${#SCOPE_FILES[@]} active file(s)"

# Using cargo path_a as sole proof language is forbidden in active 5.x evidence
# (mention in adversarial postmortem is OK if prefixed SUPERSEDED/ADVERSARIAL)
for f in "${SCOPE_FILES[@]}"; do
  base="$(basename "${f}")"
  # Plan adversarial artifact may quote forbidden patterns
  if [[ "${base}" == *ADVERSARIAL* || "${base}" == *PLAN_ADVERSARIAL* ]]; then
    continue
  fi
  if rg -n --fixed-strings 'cargo test -p dsb-agent path_a' "${f}" 2>/dev/null; then
    echo "FORBIDDEN in ${f}: cargo test -p dsb-agent path_a as evidence" >&2
    FAIL=1
  fi
  if rg -n --fixed-strings 'cargo test -p dsb-context path_a' "${f}" 2>/dev/null; then
    echo "FORBIDDEN in ${f}: cargo test -p dsb-context path_a as evidence" >&2
    FAIL=1
  fi
  if rg -n --fixed-strings 'cargo test -p dsb-tools path_a' "${f}" 2>/dev/null; then
    echo "FORBIDDEN in ${f}: cargo test -p dsb-tools path_a as evidence" >&2
    FAIL=1
  fi
  if rg -n 'owner-bar green|owner bar green|5\.0\.0 complete' "${f}" 2>/dev/null | \
     rg -i 'pass|shipped|complete' >/dev/null 2>&1; then
    # Only fail if also no R0A mention — soft: require explicit R0A in same file
    if ! rg -n 'R0A|public entry|wire transcript' "${f}" >/dev/null 2>&1; then
      echo "FORBIDDEN in ${f}: 5.0.0 complete claim without R0A language" >&2
      FAIL=1
    fi
  fi
done

if [[ "${FAIL}" -ne 0 ]]; then
  echo "check-forbidden-evidence: FAIL" >&2
  exit 1
fi
echo "check-forbidden-evidence: PASS"
exit 0
