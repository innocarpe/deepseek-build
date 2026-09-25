#!/usr/bin/env bash
# Re-apply DSB's local patches to the vendored Grok Build tree.
#
# **When the overlay is carried as patches**, a refresh that swaps the tree
# (see docs/contributing/grok-sync-runbook.md) wipes DSB's feature work, so the
# series under patches/grok-build/ is re-applied afterwards.
#
# **When the overlay is carried in the tree**, there is no series to apply: the
# refresh is a three-way merge that already carries the overlay, and this script
# reports that and exits 0. That is the state after a large-overlay sync -- see
# the sync ledger for which is current.
#
# Usage:
#   ./scripts/apply-grok-build-patches.sh          # apply all patches
#   ./scripts/apply-grok-build-patches.sh --check  # dry-run: verify they apply
#
# Exit 0 when every patch applied (or, with --check, is applicable), or when
# the overlay is in-tree and there is nothing to apply; nonzero with a clear
# message when a patch conflicts (e.g. upstream moved on) so the refresh PR
# author fixes it before merging.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PATCHES="${ROOT}/patches/grok-build"
VENDOR="${ROOT}/third_party/grok-build"
MODE="${1:-apply}"

if [[ ! -f "$VENDOR/SOURCE_REV" ]]; then
  echo "apply-grok-build-patches: missing vendor tree: $VENDOR" >&2
  exit 1
fi

shopt -s nullglob
PATCH_FILES=("$PATCHES"/*.patch)
if [[ ! -d "$PATCHES" ]]; then
  echo "apply-grok-build-patches: no patches dir ($PATCHES)."
  echo "apply-grok-build-patches: the overlay is carried in-tree, so there is nothing to apply."
  echo "apply-grok-build-patches: that is the post-merge state; see docs/product/UPSTREAM_SYNC_LEDGER.md."
  exit 0
fi

shopt -s nullglob
PATCH_FILES=("$PATCHES"/*.patch)
if [[ ${#PATCH_FILES[@]} -eq 0 ]]; then
  echo "apply-grok-build-patches: no *.patch files under $PATCHES."
  echo "apply-grok-build-patches: the overlay is carried in-tree, so there is nothing to apply."
  echo "apply-grok-build-patches: that is the post-merge state; see docs/product/UPSTREAM_SYNC_LEDGER.md."
  exit 0
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
