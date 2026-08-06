#!/usr/bin/env bash
# Validate a Conventional Commits PR/commit title.
# Usage: ./scripts/check-pr-title.sh "docs(contributing): add PR guide"
set -euo pipefail

TITLE="${1:-}"
if [[ -z "${TITLE}" ]]; then
  echo "usage: $0 \"<title>\"" >&2
  exit 2
fi

pattern='^(feat|fix|docs|spec|chore|refactor|test|ci|perf|build)(\([a-z0-9][a-z0-9/_-]*\))?\!?: .+'
if ! printf '%s\n' "${TITLE}" | grep -Eq "${pattern}"; then
  echo "invalid title: ${TITLE}" >&2
  echo "expected: type(scope)?: summary" >&2
  exit 1
fi
if printf '%s\n' "${TITLE}" | grep -Eq '\.$'; then
  echo "title should not end with a period" >&2
  exit 1
fi
echo "ok: ${TITLE}"
