#!/usr/bin/env bash
# Sync .github/labels.json to the GitHub repository via gh.
# Usage: ./scripts/sync-labels.sh [--repo owner/name]
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
LABELS_JSON="${ROOT}/.github/labels.json"

GH=(gh)
if [[ "${1:-}" == "--repo" ]]; then
  REPO="${2:?repo required}"
  GH=(gh --repo "${REPO}")
elif [[ -n "${1:-}" ]]; then
  GH=(gh --repo "${1}")
fi

if ! command -v jq >/dev/null 2>&1; then
  echo "jq is required" >&2
  exit 1
fi

if [[ ! -f "${LABELS_JSON}" ]]; then
  echo "missing ${LABELS_JSON}" >&2
  exit 1
fi

count="$(jq 'length' "${LABELS_JSON}")"
echo "syncing ${count} labels…"

# Snapshot existing names once (paginate)
mapfile -t existing < <("${GH[@]}" label list --limit 200 --json name --jq '.[].name')

has_label() {
  local want="$1" e
  for e in "${existing[@]+"${existing[@]}"}"; do
    [[ "${e}" == "${want}" ]] && return 0
  done
  return 1
}

jq -c '.[]' "${LABELS_JSON}" | while read -r row; do
  name="$(jq -r '.name' <<<"${row}")"
  color="$(jq -r '.color' <<<"${row}")"
  desc="$(jq -r '.description // ""' <<<"${row}")"
  if has_label "${name}"; then
    "${GH[@]}" label edit "${name}" --color "${color}" --description "${desc}" >/dev/null
    echo "  updated: ${name}"
  else
    if "${GH[@]}" label create "${name}" --color "${color}" --description "${desc}" >/dev/null 2>/tmp/dsb-label-create.err; then
      echo "  created: ${name}"
      existing+=("${name}")
    else
      # Race or default GitHub label with different listing — force-edit
      if "${GH[@]}" label edit "${name}" --color "${color}" --description "${desc}" >/dev/null 2>/dev/null; then
        echo "  updated(after create race): ${name}"
      else
        cat /tmp/dsb-label-create.err >&2
        echo "failed: ${name}" >&2
        exit 1
      fi
    fi
  fi
done

echo "done."
