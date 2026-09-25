#!/usr/bin/env bash
# Wait until the public npm registry serves a published version, then exit 0.
#
# Why a bounded retry and not a single immediate read (measured 2026-09-25):
#
#   In the v5.7.0 release, `.github/workflows/publish-npm.yml` published at
#   12:17:45Z (step `success`) and read the registry one second later at
#   12:17:51Z. That read got `E404 No match found for version 5.7.0`. The
#   registry's own metadata (`npm view <pkg> time`) records the version as
#   landing at 12:19:07Z — 76 s after the read. Under `set -euo pipefail` that
#   404 became a failed job, and the step after it (`Global install smoke`, the
#   end-to-end user path) was recorded as `skipped`.
#
#   Nothing in npm's public documentation promises that a successful
#   `npm publish` is immediately readable, and the read path below is not a
#   single authoritative server: npm's registry documentation describes reads
#   as passing through a CDN in front of `registry.npmjs.org`
#   (`docs/varnish-config.md` in npm/registry). That a response can be served
#   from a cache is directly visible for this package: a metadata read answers
#   with `cache-control: public, max-age=300` and a populated `age`. So a read
#   can legitimately be served a document that predates the publish, and "the
#   publish returned, so a read must see it" is an unbounded race rather than a
#   guarantee. The 76 s above is the price that race charged the last release:
#   a single immediate read fails whenever propagation has not finished.
#
# Usage:
#   ./scripts/verify-npm-version.sh <name@version> [options]
#
#   --timeout SEC       total retry window            (default 300)
#   --initial SEC       first interval between reads  (default 5)
#   --max-interval SEC  ceiling for the backoff       (default 30)
#
# Examples:
#   ./scripts/verify-npm-version.sh "@innocarpe/deepseek-build@5.7.0"
#   DSB_NPM_VERIFY_TIMEOUT_SEC=8 ./scripts/verify-npm-version.sh "@scope/name@9.9.9"
#
# Why 300 s: the observed propagation delay was 76 s, so the default window is
# about four times the measurement. It is also the order of magnitude of the
# cache lifetime this package's metadata is served with (`cache-control:
# public, max-age=300`, measured on a read of this package), so the window
# covers a full cache period rather than a guessed fraction of one. The
# interval starts at 5 s so a short delay passes almost immediately, and
# doubles to a 30 s ceiling so a long one costs fewer than 15 reads. Both are
# far inside the workflow's 60-minute job timeout and `release.sh`'s 5400 s
# default.
#
# A window is a judgement, not a proof: a read can in principle still be served
# a stale document at the end of it. The point is that the failure is then
# reported as a timeout with its own diagnostic instead of as a bare 404 that
# looks like a missing package.
#
# Environment (same meaning as the flags; a flag wins):
#   DSB_NPM_VERIFY_TIMEOUT_SEC   total retry window   (default 300)
#   DSB_NPM_VERIFY_INITIAL_SEC   first interval       (default 5)
#   DSB_NPM_VERIFY_MAX_SEC       interval ceiling     (default 30)
#
# Output: on success the confirmed version on stdout, one line. Progress and
# diagnostics go to stderr. Exit 1 if the registry still does not serve the
# version at the deadline; exit 2 for a usage error.
set -euo pipefail

SPEC=""
TIMEOUT="${DSB_NPM_VERIFY_TIMEOUT_SEC:-300}"
INITIAL="${DSB_NPM_VERIFY_INITIAL_SEC:-5}"
MAX_INTERVAL="${DSB_NPM_VERIFY_MAX_SEC:-30}"

usage() {
  cat >&2 <<'EOF'
usage: verify-npm-version.sh <name@version> [options]

Waits (bounded retry) until the public npm registry serves <name@version>.
Success prints the confirmed version on stdout.

  --timeout SEC       total retry window            (default 300)
  --initial SEC       first interval between reads  (default 5)
  --max-interval SEC  ceiling for the backoff       (default 30)
  -h, --help          show this help

Environment (same meaning; a flag wins):
  DSB_NPM_VERIFY_TIMEOUT_SEC, DSB_NPM_VERIFY_INITIAL_SEC, DSB_NPM_VERIFY_MAX_SEC

Exit status: 0 the version is served; 1 the window expired; 2 usage error.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --timeout) TIMEOUT="${2:-}"; shift 2 ;;
    --initial) INITIAL="${2:-}"; shift 2 ;;
    --max-interval) MAX_INTERVAL="${2:-}"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    -*) echo "error: unknown option: $1" >&2; exit 2 ;;
    *) SPEC="$1"; shift ;;
  esac
done

if [[ -z "$SPEC" ]]; then
  echo "error: usage: $0 <name@version> [--timeout SEC] [--initial SEC] [--max-interval SEC]" >&2
  exit 2
fi
for pair in "timeout=$TIMEOUT" "initial=$INITIAL" "max-interval=$MAX_INTERVAL"; do
  name="${pair%%=*}"; value="${pair#*=}"
  if [[ ! "$value" =~ ^[0-9]+$ ]] || [[ "$value" -lt 1 ]]; then
    echo "error: --${name} must be a whole number of seconds >= 1 (got '${value}')" >&2
    exit 2
  fi
done
if [[ "$MAX_INTERVAL" -lt "$INITIAL" ]]; then
  MAX_INTERVAL="$INITIAL"
fi

# The version is everything after the last '@'; the name is what precedes it.
# A bare version with no package name cannot be looked up.
EXPECTED="${SPEC##*@}"
if [[ "$SPEC" == "$EXPECTED" || "$EXPECTED" == */* ]]; then
  echo "error: '${SPEC}' is not <name@MAJOR.MINOR.PATCH> (no version after '@')" >&2
  exit 2
fi
if [[ ! "$EXPECTED" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "error: '${EXPECTED}' is not full SemVer MAJOR.MINOR.PATCH" >&2
  exit 2
fi

command -v npm >/dev/null 2>&1 || { echo "error: npm not found on PATH" >&2; exit 2; }

# One metadata read. Empty output means "the registry does not serve it" (404,
# an error, or no version field); `|| true` keeps a failed read from tripping
# `set -e`, because a failed read is the expected state on early attempts.
#
# `--prefer-online` forces the staleness check on npm's *local* cache (npm's
# config docs: it makes the CLI look for updates immediately even for fresh
# package data). That only rules out the client answering from a copy this
# machine already holds; it does not affect the CDN or the registry's own
# caches, so it narrows the race without closing it — the retry loop is what
# closes it.
read_registry_version() {
  npm view --prefer-online "$SPEC" version 2>/dev/null | head -n 1 | tr -d '[:space:]' || true
}

STARTED="$(date +%s)"
DEADLINE=$(( STARTED + TIMEOUT ))
interval="$INITIAL"
attempt=0
live=""

while :; do
  attempt=$(( attempt + 1 ))
  live="$(read_registry_version)"
  if [[ "$live" == "$EXPECTED" ]]; then
    echo "$live"
    exit 0
  fi

  NOW="$(date +%s)"
  LEFT=$(( DEADLINE - NOW ))
  if [[ "$LEFT" -le 0 ]]; then
    break
  fi

  WAIT="$interval"
  [[ "$WAIT" -gt "$LEFT" ]] && WAIT="$LEFT"
  echo "  attempt ${attempt}: registry reports '${live:-none}' — retrying in ${WAIT}s (${LEFT}s left)" >&2
  sleep "$WAIT"
  interval=$(( interval * 2 ))
  [[ "$interval" -gt "$MAX_INTERVAL" ]] && interval="$MAX_INTERVAL"
done

WAITED=$(( $(date +%s) - STARTED ))
CURRENT="$(read_registry_version)"
{
  echo "error: the npm registry still does not serve ${SPEC} — waited ${WAITED}s over ${attempt} reads."
  echo "  last read:  '${live:-none}'"
  echo "  now:        '${CURRENT:-none}'"
  echo "  window:     ${TIMEOUT}s total, ${INITIAL}s initial interval, ${MAX_INTERVAL}s ceiling"
  echo
  echo "  This is a registry-visibility failure, not necessarily a publish failure:"
  echo "  the publish step can have succeeded while metadata has not propagated."
  echo "  Check the publishing run's log before treating the version as lost."
  echo
  echo "  Re-check by hand:   npm view '${SPEC}' version"
  echo "  Widen the window:   DSB_NPM_VERIFY_TIMEOUT_SEC=<seconds> $0 '${SPEC}'"
} >&2
exit 1
