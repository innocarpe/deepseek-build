#!/usr/bin/env bash
# Shared helpers for owner-bar gate scripts.
# shellcheck shell=bash

owner_bar_repo_root() {
  local here
  here="$(cd "$(dirname "${BASH_SOURCE[1]}")/.." && pwd)"
  # scripts/lib → repo root is ../..
  if [[ -f "${here}/../Cargo.toml" ]]; then
    cd "${here}/.." && pwd
  elif [[ -f "${here}/Cargo.toml" ]]; then
    echo "${here}"
  else
    cd "$(dirname "${BASH_SOURCE[1]}")/../.." && pwd
  fi
}

owner_bar_git_sha() {
  git -C "${1:-.}" rev-parse HEAD 2>/dev/null || echo "unknown"
}

owner_bar_fail() {
  echo "OWNER_BAR_FAIL: $*" >&2
  return 1
}
