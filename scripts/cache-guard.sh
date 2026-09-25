#!/usr/bin/env bash
# Cache regression bench — the release gate for spec 10 §1.9.
#
# Scores `cache-first` as a curve: scripted turn scenarios against a
# prefix-accounting mock provider, on two layers (distinct-epoch exactness and
# the last-3-request hit rate). The scenarios live in
# `crates/dsb-agent/tests/cache_guard.rs`; `cache_guard_negative_control` is
# the control that catches a mock answering from constants.
#
# Release-gated like Reasonix's `scripts/cache-guard.sh`: it skips unless
# `DSB_RELEASE_CACHE_GUARD=1`, so the ordinary `cargo test --workspace` never
# pays for the bench. Run it before cutting a release:
#
#   DSB_RELEASE_CACHE_GUARD=1 ./scripts/cache-guard.sh
#
# `DSB_CACHE_GUARD_THRESHOLD` (default 90) tunes the rate-layer threshold.
#
# Usage: ./scripts/cache-guard.sh
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

if [[ "${DSB_RELEASE_CACHE_GUARD:-}" != "1" ]]; then
  echo "cache-guard: skipped (set DSB_RELEASE_CACHE_GUARD=1 to run the release cache guard)"
  exit 0
fi

echo "cache-guard: threshold=${DSB_CACHE_GUARD_THRESHOLD:-90}% (DSB_CACHE_GUARD_THRESHOLD)"
cargo test -p dsb-agent --test cache_guard -- --nocapture
