#!/usr/bin/env bash
# Run a cargo command in the vendored Grok tree, in this worktree's own target.
#
#   ./scripts/vendor-cargo.sh [wrapper flags] <cargo args...>
#
#   --jobs N             job count for cargo (default: CARGO_BUILD_JOBS, else 4)
#   --no-seed            never seed a cold target
#   --allow-concurrent   start even while another worktree's vendored build is
#                        in flight, when the memory gate passes (then 2 jobs)
#   --vendor DIR         vendored tree (default: <worktree>/third_party/grok-build)
#   --target-dir DIR     cargo target directory (default: <vendor>/target)
#   --repo DIR           repository whose worktrees the queue check reads
#
# Why
# ---
# Sessions used to export CARGO_TARGET_DIR=<tower>/third_party/grok-build/target
# so one warm target served every worktree. cargo names a path package's
# artifacts by its path relative to the workspace root, so the worktrees wrote
# the same file names into it, and cargo's mtime-only freshness then handed the
# worktree with the older sources the other worktree's code (measured
# 2026-09-27: worktree B's `cargo test` finished in 0.35s and ran worktree A's
# binary). This wrapper exports the target of the worktree it runs in, ignores
# a foreign one, and seeds a cold target once (scripts/lib/dsbdev.py) so the
# first command compiles what differs instead of the whole workspace.
#
# Everything after the wrapper flags goes to cargo unchanged; wrapper parsing
# stops at `--` or at the first token that is not a wrapper flag, so cargo's
# own `-j` and `--target <triple>` keep working.
#
# Logic: scripts/lib/vendor_cargo.py.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"

if ! command -v python3 >/dev/null 2>&1; then
  echo "vendor-cargo: error: python3 is not on PATH" >&2
  exit 2
fi

exec python3 "${ROOT}/scripts/lib/vendor_cargo.py" "$@"
