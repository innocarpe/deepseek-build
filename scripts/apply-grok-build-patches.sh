#!/usr/bin/env bash
# Re-apply DSB's local patches to the vendored Grok Build tree.
#
# The vendor refresh procedure (docs/architecture/GROK_VENDOR.md) uses
# `rsync --delete`, which wipes any local edits under third_party/grok-build/.
# DSB's feature work (the DeepSeek status line) is carried as patches under
# patches/grok-build/ so a refresh never silently drops it.
#
# Usage:
#   ./scripts/apply-grok-build-patches.sh          # apply all patches
#   ./scripts/apply-grok-build-patches.sh --check  # dry-run: verify they apply
#
# Exit 0 when every patch applied (or, with --check, is applicable); nonzero
# with a clear message when a patch conflicts (e.g. upstream moved on) so the
# refresh PR author fixes it before merging.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PATCHES="${ROOT}/patches/grok-build"
VENDOR="${ROOT}/third_party/grok-build"
MODE="${1:-apply}"

if [[ ! -d "$PATCHES" ]]; then
  echo "apply-grok-build-patches: no patches dir: $PATCHES" >&2
  echo "Regenerate with: git format-patch <base>..HEAD -- third_party/grok-build -o patches/grok-build" >&2
  exit 1
fi

if [[ ! -f "$VENDOR/SOURCE_REV" ]]; then
  echo "apply-grok-build-patches: missing vendor tree: $VENDOR" >&2
  exit 1
fi

shopt -s nullglob
PATCH_FILES=("$PATCHES"/*.patch)
if [[ ${#PATCH_FILES[@]} -eq 0 ]]; then
  echo "apply-grok-build-patches: no *.patch files under $PATCHES" >&2
  exit 1
fi

cd "$ROOT"

applied=0
for patch in "${PATCH_FILES[@]}"; do
  name="$(basename "$patch")"
  if git apply --check "$patch" 2>/dev/null; then
    if [[ "$MODE" == "--check" ]]; then
      echo "apply-grok-build-patches: applicable: $name"
      continue
    fi
    git apply "$patch"
    echo "apply-grok-build-patches: applied: $name"
    applied=$((applied + 1))
  elif git apply --reverse --check "$patch" 2>/dev/null; then
    echo "apply-grok-build-patches: already applied, skipping: $name"
  else
    echo "apply-grok-build-patches: FAILED to apply $name — the vendor tree changed upstream." >&2
    echo "Fix the conflict by hand, re-run ./scripts/build-grok-pager.sh check, then regenerate:" >&2
    echo "  git format-patch <base>..HEAD -- third_party/grok-build -o patches/grok-build" >&2
    exit 1
  fi
done

if [[ "$MODE" == "--check" ]]; then
  echo "apply-grok-build-patches: all ${#PATCH_FILES[@]} patches applicable"
else
  echo "apply-grok-build-patches: $applied patch(es) applied, ${#PATCH_FILES[@]} total"
fi
