#!/usr/bin/env bash
# A merge claim is not a fact until this prints `pass`.
#
#   GH_TOKEN="$(gh auth token --user innocarpe)" ./scripts/check-pr-merged.sh <pr>
#   ./scripts/check-pr-merged.sh --fixture path/to.json
#
# Exit 0: MERGED, mergedAt, mergeCommit, two parents, CI / required passed.
# Exit 2: CI has not finished.
# Exit 1: not merged, squashed, or CI failed.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PY="${ROOT}/scripts/lib/pr_merged.py"

if [[ "${1:-}" == "--fixture" ]]; then
  exec python3 "${PY}" --fixture "$2"
fi
if [[ -z "${1:-}" ]]; then
  echo "usage: $0 <pr> | $0 --fixture <json>" >&2
  exit 2
fi
exec python3 "${PY}" --pr "$1" --repo "${DSB_REPO:-innocarpe/deepseek-build}"
