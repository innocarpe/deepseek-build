#!/usr/bin/env bash
# The primary worktree is refused. A linked worktree is not.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
GUARD="$ROOT/scripts/lib/refuse-primary-checkout.sh"
fail=0

bad() { echo "not ok: $*" >&2; fail=1; }
ok() { echo "ok: $*"; }

[[ -x "$GUARD" ]] || chmod +x "$GUARD"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

export GIT_AUTHOR_NAME=test GIT_AUTHOR_EMAIL=test@example.com
export GIT_COMMITTER_NAME=test GIT_COMMITTER_EMAIL=test@example.com

git init -b main "$tmp/primary" >/dev/null
git -C "$tmp/primary" commit --allow-empty -m init >/dev/null
git -C "$tmp/primary" worktree add -b side "$tmp/linked" >/dev/null

if "$GUARD" "$tmp/primary" >/tmp/refuse-primary.out 2>&1; then
  bad "primary worktree was accepted"
else
  if grep -q "stays on main" /tmp/refuse-primary.out; then
    ok "primary worktree refused"
  else
    bad "primary refusal did not say the branch stays on main"
    cat /tmp/refuse-primary.out >&2
  fi
fi

if "$GUARD" "$tmp/linked" >/tmp/refuse-linked.out 2>&1; then
  ok "linked worktree accepted"
else
  bad "linked worktree was refused"
  cat /tmp/refuse-linked.out >&2
fi

if "$GUARD" >/tmp/refuse-usage.out 2>&1; then
  bad "missing argument was accepted"
else
  ok "missing argument refused"
fi

if grep -q 'refuse-primary-checkout.sh' "$ROOT/scripts/release.sh"; then
  ok "release.sh calls the guard"
else
  bad "release.sh does not call the guard"
fi

rm -f /tmp/refuse-primary.out /tmp/refuse-linked.out /tmp/refuse-usage.out
exit "$fail"
