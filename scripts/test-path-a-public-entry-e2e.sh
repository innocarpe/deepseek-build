#!/usr/bin/env bash
# Path A public-entry smoke (G002 substrate).
# Until full scripted server exists, this proves launch resolution only and exits non-zero
# if full R0 matrix is not available — fail-closed for owner-bar.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "${ROOT}"

echo "=== test-path-a-public-entry-e2e (substrate) ==="

# Prefer built CLI if present
CLI=""
for c in \
  "${ROOT}/target/debug/deepseek-build" \
  "${ROOT}/target/release/deepseek-build" \
  "$(command -v deepseek-build 2>/dev/null || true)" \
  "$(command -v dsb 2>/dev/null || true)"
do
  if [[ -n "${c}" && -x "${c}" ]]; then
    CLI="${c}"
    break
  fi
done

if [[ -z "${CLI}" ]]; then
  echo "NO_CLI: deepseek-build/dsb binary not found — build or install first" >&2
  echo "test-path-a-public-entry-e2e: FAIL"
  exit 1
fi

echo "cli=${CLI}"
unset DEEPSEEK_BUILD_AGENT_BIN || true

# Help/version must work without agent
set +e
"${CLI}" --version
VER=$?
"${CLI}" --help >/tmp/dsb-help.txt 2>&1
HELP=$?
set -e

if [[ "${VER}" -ne 0 || "${HELP}" -ne 0 ]]; then
  echo "CLI version/help failed" >&2
  exit 1
fi

# Full wire R0 not implemented in G001 substrate — hard fail with clear next step
if [[ ! -f "${ROOT}/scripts/lib/scripted_deepseek_server.py" ]] && \
   [[ ! -f "${ROOT}/scripts/lib/scripted_deepseek_server.sh" ]]; then
  echo "MISSING: scripted DeepSeek server harness (G002) — cannot claim Path A R0A" >&2
  echo "test-path-a-public-entry-e2e: FAIL (substrate incomplete — expected until G002)"
  exit 1
fi

echo "test-path-a-public-entry-e2e: PASS"
exit 0
