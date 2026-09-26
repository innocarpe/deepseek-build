#!/usr/bin/env bash
# Exit 1 when the given repo root is the primary worktree.
# A linked worktree exits 0. release.sh calls this before it checks out
# a release branch: on 2026-09-26 that checkout ran in the maintainer's
# main worktree and left it on chore/release-6.0.2.
set -euo pipefail

root="${1:-}"
if [[ -z "$root" || ! -d "$root" ]]; then
  echo "error: refuse-primary-checkout.sh needs a repo root" >&2
  exit 2
fi

common="$(git -C "$root" rev-parse --path-format=absolute --git-common-dir)"
gitdir="$(git -C "$root" rev-parse --absolute-git-dir)"
common="${common%/}"
gitdir="${gitdir%/}"

if [[ "$common" == "$gitdir" ]]; then
  echo "error: refusing to run in the primary checkout ($root)." >&2
  echo "  That worktree stays on main. Create a linked worktree and run there." >&2
  echo "  skills/worktree-dispatch" >&2
  exit 1
fi
