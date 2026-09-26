#!/usr/bin/env bash
# Regression test for scripts/verify-npm-version.sh — the publish/verify race
# that made the v5.7.0 release run red.
#
# Hermetic: a local mock registry stands in for the public one, so this touches
# no network, publishes nothing, and needs no npm credentials. It reproduces
# the exact shape of the race — a version that is not readable yet and becomes
# readable later — which the real registry cannot be asked to do on demand.
#
# Paths covered:
#   1. pass          — the version is already served: succeeds on the first read
#   2. race          — absent at first, appears mid-window: retries, then passes
#   3. timeout       — never appears: exits 1 with a diagnostic naming what was
#                      waited for and what the registry reported
#   4. usage errors  — bad spec / bad window values exit 2
#   5. wiring        — the workflow, release.sh and the emergency path all call
#                      the shared helper instead of a one-shot `npm view`
#
# Usage: ./scripts/test-npm-verify-retry.sh
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

HELPER="$ROOT/scripts/verify-npm-version.sh"
MOCK="$ROOT/scripts/lib/mock_npm_registry.py"
PKG="@scope/verify-retry-fixture"

PASS=0
FAIL=0
TMP="$(mktemp -d)"
PIDS=()

cleanup() {
  # Reap the mock registries quietly: bash announces a signaled job with
  # "Terminated", which reads like a test failure in the output.
  for pid in "${PIDS[@]:-}"; do
    { kill "$pid" 2>/dev/null && wait "$pid" 2>/dev/null; } 2>/dev/null || true
  done
  rm -rf "$TMP"
}
trap cleanup EXIT

ok()   { printf '  ok   %s\n' "$*"; PASS=$((PASS + 1)); }
bad()  { printf '  FAIL %s\n' "$*" >&2; FAIL=$((FAIL + 1)); }
head_() { printf '\n== %s ==\n' "$*"; }

for c in npm python3; do
  command -v "$c" >/dev/null 2>&1 || { echo "error: $c not found on PATH" >&2; exit 1; }
done
[[ -f "$HELPER" && -f "$MOCK" ]] || { echo "error: helper or mock missing" >&2; exit 1; }

# Start the mock registry; sets MOCK_URL. npm is pointed at it through
# npm_config_registry, which applies to this process only — no .npmrc on the
# machine is read or written for the lookup.
#
# This deliberately sets a variable instead of printing the URL: the registry
# keeps running, so if the function's output were captured with `$(...)` the
# command substitution would wait for a pipe that the server never closes.
# It also has to run in this shell, or the pid recorded below would be lost
# with the subshell and the server would outlive the test.
start_mock() {
  local port_file="$TMP/port.$$.$RANDOM" log_file="${1:-}"
  local args=(--port-file "$port_file" --name "$PKG" --version "$TARGET")
  [[ -n "$log_file" ]] && args+=(--log "$log_file")
  if [[ $# -gt 1 ]]; then
    args+=("${@:2}")
  fi
  # Redirect away from any inherited pipe: the server holds its stdio open for
  # the whole test, and a caller capturing output would block on it.
  python3 "$MOCK" "${args[@]}" >"$TMP/mock.out" 2>&1 &
  PIDS+=("$!")

  local waited=0
  while [[ ! -s "$port_file" ]]; do
    sleep 0.1
    waited=$((waited + 1))
    if [[ "$waited" -gt 100 ]]; then
      echo "error: mock registry did not start" >&2
      exit 1
    fi
  done
  MOCK_URL="http://127.0.0.1:$(cat "$port_file")"
}

# Run the helper against a mock registry. Sets OUT_STDOUT / OUT_STDERR and
# RET, keeping the two streams apart: the helper's contract is the confirmed
# version alone on stdout and progress on stderr, so the test can check both.
run_helper() {
  local registry="$1"; shift
  local out_file="$TMP/out.$$.$RANDOM" err_file="$TMP/err.$$.$RANDOM"
  set +e
  npm_config_registry="$registry" "$HELPER" "$PKG@$TARGET" "$@" \
    >"$out_file" 2>"$err_file"
  RET=$?
  set -e
  OUT_STDOUT="$(cat "$out_file")"
  OUT_STDERR="$(cat "$err_file")"
}

TARGET="2.0.0"

head_ "1. pass — version already served (first read succeeds)"
start_mock
URL="$MOCK_URL"
START="$(date +%s)"
run_helper "$URL"
RC="$RET"
ELAPSED=$(( $(date +%s) - START ))
if [[ "$RC" -eq 0 && "$OUT_STDOUT" == "$TARGET" ]]; then
  ok "exit 0 and stdout is exactly '${TARGET}' (${ELAPSED}s)"
else
  bad "expected exit 0 with '${TARGET}', got rc=${RC} stdout='${OUT_STDOUT}' stderr='${OUT_STDERR}'"
fi
if [[ "$ELAPSED" -le 3 ]]; then
  ok "returned without waiting out the window (${ELAPSED}s)"
else
  bad "took ${ELAPSED}s — should not have retried"
fi

head_ "2. race — absent at first, appears mid-window (the 5.7.0 shape)"
start_mock "" --present-after 4
URL="$MOCK_URL"
START="$(date +%s)"
run_helper "$URL" --timeout 40 --initial 2 --max-interval 4
RC="$RET"
ELAPSED=$(( $(date +%s) - START ))
if [[ "$RC" -eq 0 && "$OUT_STDOUT" == "$TARGET" ]]; then
  ok "exit 0 after retrying (${ELAPSED}s)"
else
  bad "expected exit 0 with '${TARGET}', got rc=${RC} stdout='${OUT_STDOUT}' stderr='${OUT_STDERR}'"
fi
if [[ "$ELAPSED" -ge 4 && "$ELAPSED" -le 20 ]]; then
  ok "waited for propagation and then passed (${ELAPSED}s)"
else
  bad "elapsed ${ELAPSED}s is outside the expected 4-20s band"
fi
if printf '%s' "$OUT_STDERR" | grep -q 'attempt 1'; then
  ok "reported the failed attempt on stderr before succeeding"
else
  bad "no attempt trace on stderr; stderr was '${OUT_STDERR}'"
fi

head_ "3. timeout — version never appears"
start_mock "" --absent
URL="$MOCK_URL"
START="$(date +%s)"
run_helper "$URL" --timeout 6 --initial 2 --max-interval 3
RC="$RET"
ELAPSED=$(( $(date +%s) - START ))
if [[ "$RC" -eq 1 ]]; then
  ok "exit 1 (not 0, not a crash)"
else
  bad "expected exit 1, got rc=${RC}; stderr='${OUT_STDERR}'"
fi
if [[ -z "$OUT_STDOUT" ]]; then
  ok "printed nothing on stdout when it could not confirm"
else
  bad "stdout was not empty on failure: '${OUT_STDOUT}'"
fi
for needle in "still does not serve" "waited" "last read" "Re-check by hand"; do
  if printf '%s' "$OUT_STDERR" | grep -q "$needle"; then
    ok "diagnostic mentions '${needle}'"
  else
    bad "diagnostic missing '${needle}'; stderr='${OUT_STDERR}'"
  fi
done
if [[ "$ELAPSED" -ge 6 && "$ELAPSED" -le 15 ]]; then
  ok "honoured the window before giving up (${ELAPSED}s)"
else
  bad "elapsed ${ELAPSED}s is outside the expected 6-15s band"
fi

head_ "4. usage errors"

expect_rc() {
  local want="$1" label="$2"; shift 2
  local out rc
  set +e
  out="$("$HELPER" "$@" 2>&1)"; rc=$?
  set -e
  if [[ "$rc" -eq "$want" ]]; then
    ok "${label} → exit ${want}"
  else
    bad "${label}: expected exit ${want}, got ${rc}; out='${out}'"
  fi
}

expect_rc 2 "no argument"
expect_rc 2 "missing @version" "just-a-name"
expect_rc 2 "partial SemVer" "${PKG}@1.2"
expect_rc 2 "--timeout 0" "${PKG}@1.2.3" --timeout 0
expect_rc 2 "non-numeric --initial" "${PKG}@1.2.3" --initial abc

head_ "5. wiring — every publish path uses the shared helper"
if rg -q 'verify-npm-version\.sh' .github/workflows/publish-npm.yml; then
  ok "publish-npm.yml calls the helper"
else
  bad "publish-npm.yml does not call the helper"
fi
if rg -q 'verify-npm-version\.sh' scripts/release.sh; then
  ok "release.sh calls the helper"
else
  bad "release.sh does not call the helper"
fi
if rg -q 'verify-npm-version\.sh' scripts/npm-emergency-publish.sh; then
  ok "npm-emergency-publish.sh calls the helper"
else
  bad "npm-emergency-publish.sh does not call the helper"
fi

# A one-shot read *after* a publish is the bug: under `set -euo pipefail` a 404
# from a read that has no tolerance becomes a failed job. Tolerant probes are
# fine — the pre-publish "is it already there?" check should degrade quietly —
# so the invariant is that every remaining one-shot read tolerates failure.
check_no_failclose_read() {
  local file="$1" hits
  hits="$(rg -n 'npm view' "$file" \
    | grep 'version' \
    | grep -v 'dist\.attestations' \
    | grep -v '||' || true)"
  if [[ -z "$hits" ]]; then
    ok "${file}: no fail-close one-shot version read"
  else
    bad "${file}: one-shot version read without tolerance:"
    printf '%s\n' "$hits" >&2
  fi
}

# The verify step must route the version confirmation through the helper. The
# provenance probe right after it is deliberately a direct, tolerant read
# (attestations can lag the publish, so it degrades to a message) — that one is
# the documented exception, not the bug.
VERIFY_STEP="$(awk '/name: Verify the published package/{f=1} f&&/^      - name: /&&!/Verify the published package/{f=0} f' \
  .github/workflows/publish-npm.yml)"
if printf '%s' "$VERIFY_STEP" | grep -q 'verify-npm-version\.sh'; then
  ok "the workflow's verify step runs the shared helper"
else
  bad "the workflow's verify step does not run the helper"
fi
VERIFY_VERSION_READS="$(printf '%s' "$VERIFY_STEP" | grep 'npm view' | grep -v 'dist\.attestations' || true)"
if [[ -z "$VERIFY_VERSION_READS" ]]; then
  ok "the workflow's verify step confirms the version through the helper only"
else
  bad "the workflow's verify step still confirms the version with a direct read:"
  printf '%s\n' "$VERIFY_VERSION_READS" >&2
fi

check_no_failclose_read .github/workflows/publish-npm.yml
check_no_failclose_read scripts/release.sh
check_no_failclose_read scripts/npm-emergency-publish.sh

printf '\n%s passed, %s failed\n' "$PASS" "$FAIL"
[[ "$FAIL" -eq 0 ]] || exit 1

# Same job: the primary-checkout guard is hermetic and release.sh is already in this filter.
echo "=== primary checkout guard ==="
"$(cd "$(dirname "$0")" && pwd)/test-refuse-primary-checkout.sh"

# Same job: the one-tab launch pin is hermetic, and no path filter covers
# skills/orca-tab, so it rides in this job with the guard above.
echo "=== orca-tab one-tab pin ==="
"$(cd "$(dirname "$0")" && pwd)/test-orca-tab-one-tab.sh"
