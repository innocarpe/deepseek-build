#!/usr/bin/env bash
# Sync .github/labels.json to the GitHub repository via gh.
# Usage: ./scripts/sync-labels.sh [--repo owner/name]
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
LABELS_JSON="${ROOT}/.github/labels.json"
REPO="${1:-}"

if [[ -n "${REPO}" && "${REPO}" == --repo ]]; then
  REPO="${2:?repo required}"
  GH=(gh --repo "${REPO}")
elif [[ -n "${REPO}" ]]; then
  GH=(gh --repo "${REPO}")
else
  GH=(gh)
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

jq -c '.[]' "${LABELS_JSON}" | while read -r row; do
  name="$(jq -r '.name' <<<"${row}")"
  color="$(jq -r '.color' <<<"${row}")"
  desc="$(jq -r '.description // ""' <<<"${row}")"
  if "${GH[@]}" label list --json name --jq '.[].name' | grep -Fxq "${name}"; then
    "${GH[@]}" label edit "${name}" --color "${color}" --description "${desc}" >/dev/null
    echo "  updated: ${name}"
  else
    "${GH[@]}" label create "${name}" --color "${color}" --description "${desc}" >/dev/null
    echo "  created: ${name}"
  fi
done

echo "done."
