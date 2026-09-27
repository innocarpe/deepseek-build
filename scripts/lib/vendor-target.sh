#!/usr/bin/env bash
# Pin a vendored-cargo run to the checkout it runs in.
#
# One worktree, one target: cargo names a path package's artifacts with a hash
# of its path *relative to the workspace root*, so two worktrees that share one
# CARGO_TARGET_DIR write the same file names, and cargo's mtime-only freshness
# then hands the worktree with the older sources the other worktree's code
# (measured 2026-09-27: worktree B's `cargo test` ran worktree A's binary).
#
# Scripts that run cargo under third_party/grok-build call the function below
# so a CARGO_TARGET_DIR the caller's environment carried cannot reintroduce
# that. Source this file, then:
#
#   vendor_target_pin <checkout root>
#
# It never touches the top-level workspace: scripts that build both (the
# test-path-a-* family) call it inside the vendored subshell only.
vendor_target_pin() {
  local root="${1:?vendor_target_pin needs the checkout root}"
  local target="$root/third_party/grok-build/target"
  if [[ -n "${CARGO_TARGET_DIR:-}" && "${CARGO_TARGET_DIR}" != "$target" ]]; then
    echo "vendor-target: ignoring inherited CARGO_TARGET_DIR=${CARGO_TARGET_DIR}; this checkout builds in ${target}" >&2
  fi
  export CARGO_TARGET_DIR="$target"
}
