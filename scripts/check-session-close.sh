#!/usr/bin/env bash
# Lock the close inside the text a session loads without opening the skill,
# and reject a handoff brief that shrinks the quoted user turn.
#
#   ./scripts/check-session-close.sh
#   ./scripts/check-session-close.sh brief path/to/brief.md
#   ./scripts/check-session-close.sh --root /path/to/tree
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PY="${ROOT}/scripts/lib/session_close.py"

if [[ "${1:-}" == "brief" ]]; then
  if [[ -z "${2:-}" ]]; then
    echo "usage: $0 brief <file>" >&2
    exit 2
  fi
  exec python3 "${PY}" --brief "$2"
fi
if [[ "${1:-}" == "--root" ]]; then
  exec python3 "${PY}" --root "$2"
fi
exec python3 "${PY}" --root "${ROOT}"
