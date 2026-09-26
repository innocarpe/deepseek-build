#!/usr/bin/env bash
# The queue behind the shared vendored Grok build target.
#
#   vendor-build.sh status [--json] [--target DIR] [--repo DIR]
#   vendor-build.sh clone <slug> [--quiesce] [--full-copy] [--target DIR] [--root DIR]
#   vendor-build.sh prune [--days N] [--target DIR] [--root DIR]
#
# Sessions here share one warm cargo target for the vendored tree
# (CARGO_TARGET_DIR=<primary checkout>/third_party/grok-build/target). Cargo
# serializes builds in one target directory with an advisory flock on
# <target>/<profile-dir>/.cargo-lock, and a waiting cargo prints nothing, so
# the queue looks like a hung command in a TUI.
#
#   status  read it: holders, waiters, elapsed, command, worktree, per lock.
#           Exit 0 free, 1 busy (branch on it), 2 usage/internal error.
#   clone   skip it: copy the warm target copy-on-write into
#           ~/.cache/dsb-vendor-targets/<slug>. stdout is exactly one line —
#           `export CARGO_TARGET_DIR=<clone>` — so `eval "$(… clone x)"`
#           works; progress and warnings go to stderr.
#           Exit 0 created, 1 refused/failed, 2 usage.
#   prune   delete personal clones idle for >= N days (default 3). Never
#           touches the base target. Exit 0 done, 1 a delete failed, 2 usage.
#
# Default target: <primary checkout>/third_party/grok-build/target — the
# shared warm target, also when run from a linked worktree.
# Logic: scripts/lib/vendor_build.py.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"

if ! command -v python3 >/dev/null 2>&1; then
  echo "vendor-build: error: python3 is not on PATH" >&2
  exit 2
fi

exec python3 "${ROOT}/scripts/lib/vendor_build.py" "$@"
