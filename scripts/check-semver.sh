#!/usr/bin/env bash
# Fail-close: workspace package version must be full SemVer MAJOR.MINOR.PATCH
# (optional -prerelease / +build). Rejects "1.0", "v1.0.0" in Cargo version field.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

ver="$(rg -n '^version = "' Cargo.toml | head -1 | sed -E 's/.*"([^"]+)".*/\1/')"
if [[ -z "$ver" ]]; then
  echo "check-semver: no version = in Cargo.toml" >&2
  exit 1
fi

# SemVer 2.0.0 core: MAJOR.MINOR.PATCH then optional pre/build
if [[ ! "$ver" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?(\+[0-9A-Za-z.-]+)?$ ]]; then
  echo "check-semver: invalid or incomplete SemVer: '$ver'" >&2
  echo "  require MAJOR.MINOR.PATCH (e.g. 1.0.0), not 1.0 or v1" >&2
  exit 1
fi

# When package.json exists, npm version must match workspace SemVer (ADR 0006).
if [[ -f package.json ]]; then
  npm_ver="$(node -e "console.log(require('./package.json').version)" 2>/dev/null || true)"
  if [[ -z "$npm_ver" ]]; then
    echo "check-semver: could not read package.json version" >&2
    exit 1
  fi
  if [[ "$npm_ver" != "$ver" ]]; then
    echo "check-semver: package.json version '$npm_ver' != Cargo.toml '$ver'" >&2
    exit 1
  fi
  echo "check-semver: npm matches cargo ($npm_ver)"
fi

echo "check-semver: ok ($ver)"
