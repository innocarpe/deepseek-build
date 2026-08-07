#!/usr/bin/env bash
# Fail if path_a_* hearts are orphaned or dead-wired. macOS bash 3.2 OK.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "${ROOT}"

echo "check-path-a-linkage: scanning for orphan PathA symbols"
ORPHANS=0

while IFS= read -r line; do
  [[ -z "${line}" ]] && continue
  file="${line%%:*}"
  rest="${line#*:}"
  name="$(printf '%s\n' "${rest}" | sed -n 's/.*fn \([a-zA-Z0-9_]*\).*/\1/p')"
  [[ -z "${name}" ]] && continue
  hits="$(rg -n --glob '*.rs' "\\b${name}\\b" crates third_party 2>/dev/null | \
    rg -v "${file}" | \
    rg -v 'mod tests|#\[cfg\(test\)|#\[test\]' || true)"
  if [[ -z "${hits}" ]]; then
    echo "ORPHAN (no non-test external call site): ${name} @ ${file}"
    ORPHANS=$((ORPHANS + 1))
  fi
done < <(rg -n 'fn (assemble_path_a_context|prepare_path_a_tool_call|path_a_default_router|route_path_a_turn)\b' \
  crates --glob '*.rs' 2>/dev/null || true)
# Note: matches both `fn` and `pub fn` via substring "fn name"

if ! rg -n 'dsb-(tools|context|agent)' third_party/grok-build --glob 'Cargo.toml' >/dev/null 2>&1; then
  echo "NOTE: third_party/grok-build has no dsb-* Cargo dependency (expected until F1)"
fi

if rg -n 'effective != .*FileToolset::Standard' \
  third_party/grok-build/crates/codegen/xai-grok-shell/src/agent --glob '*.rs' >/dev/null 2>&1; then
  echo "DEAD_WIRING: tool_configs applied only when effective != Standard"
  ORPHANS=$((ORPHANS + 1))
fi

READ_DIR="third_party/grok-build/crates/codegen/xai-grok-tools/src/implementations/grok_build/read_file"
if [[ -d "${READ_DIR}" ]]; then
  if ! rg -n 'file_version|snippet_id' "${READ_DIR}" >/dev/null 2>&1; then
    echo "NO_MINT: read_file has no file_version/snippet_id issuance"
    ORPHANS=$((ORPHANS + 1))
  fi
fi

if [[ "${ORPHANS}" -gt 0 ]]; then
  echo "check-path-a-linkage: FAIL (${ORPHANS} issues) — expected RED until fusion"
  exit 1
fi
echo "check-path-a-linkage: PASS"
exit 0
