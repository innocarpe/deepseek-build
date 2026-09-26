#!/usr/bin/env bash
# Pin the one-tab dsb launch in skills/orca-tab/SKILL.md.
#
# The 2026-09-26 shape: a dsb worktree ended with two tabs — the launcher
# shell and a `terminal create` tab that ran dsb. #248 added the §3 sentences
# that say the launcher shell is the tab, dsb is sent into it, and no second
# tab is opened. They are prose; nothing failed if a later edit dropped them.
#
# Hermetic. The live tree must keep all three sentences, and a copy with any
# one of them removed must fail — so every needle is proven load-bearing, not
# decorative.
#
# Usage: ./scripts/test-orca-tab-one-tab.sh
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SKILL_REL="skills/orca-tab/SKILL.md"

# Sentences §3 must keep, matched against the whitespace-collapsed file.
NEEDLES=(
  'That shell is the tab'
  'Send `dsb` into it'
  'Do not open a second one with `terminal create`'
)

fail=0
ok() { echo "ok: $*"; }
bad() { echo "not ok: $*" >&2; fail=1; }

collapsed() { tr '\n' ' ' < "$1" | tr -s '[:space:]' ' '; }

# One line per gap between a tree and the pin; empty output means it holds.
check_root() {
  local root="$1" text needle
  local skill="$root/$SKILL_REL"
  if [[ ! -f "$skill" ]]; then
    echo "$SKILL_REL is missing"
    return 0
  fi
  text="$(collapsed "$skill")"
  for needle in "${NEEDLES[@]}"; do
    [[ "$text" == *"$needle"* ]] || echo "$SKILL_REL no longer says: $needle"
  done
}

missing="$(check_root "$ROOT")"
if [[ -z "$missing" ]]; then
  ok "live $SKILL_REL keeps all ${#NEEDLES[@]} pinned sentences"
else
  bad "live $SKILL_REL"
  printf '%s\n' "$missing" >&2
fi

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

mkdir -p "$tmp/empty"
if [[ -n "$(check_root "$tmp/empty")" ]]; then
  ok "a tree without $SKILL_REL fails"
else
  bad "a tree without $SKILL_REL still passed"
fi

# Skipped only when the live file is already missing, which the check above
# reported.
if [[ -f "$ROOT/$SKILL_REL" ]]; then
  text="$(collapsed "$ROOT/$SKILL_REL")"
  for needle in "${NEEDLES[@]}"; do
    mkdir -p "$tmp/stripped/skills/orca-tab"
    printf '%s' "${text/"$needle"/}" > "$tmp/stripped/$SKILL_REL"
    if [[ -n "$(check_root "$tmp/stripped")" ]]; then
      ok "a copy without '$needle' fails"
    else
      bad "a copy without '$needle' still passed"
    fi
  done
fi

exit "$fail"
