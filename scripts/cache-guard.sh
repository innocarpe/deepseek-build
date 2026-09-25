#!/usr/bin/env bash
# Cache regression bench — the release gate for spec 10 §1.9.
#
# Scores `cache-first` as a curve: scripted turn scenarios against a
# prefix-accounting mock provider, on two layers (distinct-epoch exactness and
# the last-3-request hit rate). Two harnesses:
#
# - Overlay (Path B): `crates/dsb-agent/tests/cache_guard.rs`. Always runs
#   when this gate is on. `cache_guard_negative_control` catches a mock that
#   answers from constants.
# - Vendored (Path A): `cache_guard_path_a_*` in `xai-grok-shell`. Runs only
#   when this checkout has already compiled that crate into
#   `third_party/grok-build/target` — either `libxai_grok_shell-*.rlib` (a
#   dependency or product build) or the `xai_grok_shell-*` test harness
#   (`cargo test --lib` does not leave the rlib). A cold vendored build is
#   30–60+ minutes and this gate does not start one. A checkout that has
#   never compiled the crate skips with a message. The overlay tests still run.
#
# Release-gated like Reasonix's `scripts/cache-guard.sh`: it skips unless
# `DSB_RELEASE_CACHE_GUARD=1`, so the ordinary `cargo test --workspace` never
# pays for the bench. Run it before cutting a release:
#
#   DSB_RELEASE_CACHE_GUARD=1 ./scripts/cache-guard.sh
#
# `DSB_CACHE_GUARD_THRESHOLD` (default 90) tunes the rate-layer threshold for
# both harnesses.
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
echo "cache-guard: overlay bench (dsb-agent)"
cargo test -p dsb-agent --test cache_guard -- --nocapture

# `cargo test --lib` emits the harness `xai_grok_shell-<hash>` and no rlib.
# A pager or release build emits `libxai_grok_shell-<hash>.rlib`. Either
# means the crate has been compiled here, so the test below is incremental.
# Object files and rmeta are not enough: they appear mid-build.
vendor_artifact="$(
  find "$ROOT/third_party/grok-build/target" \
    \( -name 'libxai_grok_shell-*.rlib' -o -name 'xai_grok_shell-*' \) \
    -type f \
    ! -name '*.o' \
    ! -name '*.rmeta' \
    ! -name '*.d' \
    -print -quit 2>/dev/null || true
)"
if [[ -z "$vendor_artifact" ]]; then
  echo "cache-guard: vendored Path A guard skipped (xai-grok-shell has not been compiled under third_party/grok-build/target; this gate does not start a cold vendor build)"
else
  echo "cache-guard: vendored Path A guard (${vendor_artifact})"
  (cd "$ROOT/third_party/grok-build" && cargo test -p xai-grok-shell --lib cache_guard_path_a -- --nocapture)
fi
