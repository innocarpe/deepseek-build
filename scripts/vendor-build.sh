#!/usr/bin/env bash
# The per-worktree queues behind vendored Grok builds.
#
#   vendor-build.sh status [--json] [--target DIR] [--repo DIR]
#   vendor-build.sh run [--repo DIR] -- <cmd> [args...]
#   vendor-build.sh prune [--days N] [--target DIR] [--root DIR]
#
# Every worktree of this repo builds the vendored tree in its own cargo target,
# <worktree>/third_party/grok-build/target. Two worktrees writing one target
# overwrite each other's artifacts, and cargo's mtime-only freshness then hands
# the worktree with the older sources the other worktree's code (measured
# 2026-09-27) — so there is no shared build target, and one lock file no longer
# answers "is a vendored build running?".
#
#   status  read every worktree's queue: holders, waiters, elapsed, command,
#           worktree per lock, plus host free/swap/load5m and the memory-gate
#           verdict. `--target DIR` reads one target instead.
#           Exit 0 free, 1 busy (branch on it), 2 usage/internal error.
#   run     run a command now. Nothing in flight → runs as-is. A build in
#           flight → runs only when the memory gate passes, with
#           CARGO_BUILD_JOBS=2; otherwise nothing starts, status is printed,
#           and the exit code is 1. The command keeps its own exit code when
#           it runs; its target directory is its own business
#           (`scripts/vendor-cargo.sh` pins it to the worktree).
#   prune   delete personal target copies idle for >= N days (default 3) under
#           ~/.cache/dsb-vendor-targets — the namespace the old `clone` left
#           behind. Never touches a base target. Exit 0 done, 1 a delete
#           failed, 2 usage.
#
# Default repo: this repo's primary checkout — that is the worktree set
# `status` reads. Logic: scripts/lib/vendor_build.py.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"

if ! command -v python3 >/dev/null 2>&1; then
  echo "vendor-build: error: python3 is not on PATH" >&2
  exit 2
fi

exec python3 "${ROOT}/scripts/lib/vendor_build.py" "$@"
