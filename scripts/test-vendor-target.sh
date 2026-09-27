#!/usr/bin/env bash
# Hermetic test for scripts/lib/vendor-target.sh — the pin vendored-cargo
# scripts call so a CARGO_TARGET_DIR from the caller's environment cannot make
# two worktrees write one target (measured 2026-09-27: that hands the worktree
# with the older sources the other worktree's code).
#
# Usage: ./scripts/test-vendor-target.sh
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
LIB="$ROOT/scripts/lib/vendor-target.sh"

PASS=0
FAIL=0
TMP="$(cd "$(mktemp -d)" && pwd -P)"
trap 'rm -rf "$TMP"' EXIT

ok()  { printf '  ok   %s\n' "$*"; PASS=$((PASS + 1)); }
bad() { printf '  FAIL %s\n' "$*" >&2; FAIL=$((FAIL + 1)); }
head_() { printf '\n== %s ==\n' "$*"; }

V=third_party/grok-build
mkdir -p "$TMP/checkout/$V/target"

run_pin() { # <root> [inherited value]
  RC=0
  OUT="$(
    {
      if [[ -n "${2-}" ]]; then export CARGO_TARGET_DIR="$2"; fi
      # shellcheck source=lib/vendor-target.sh
      source "$LIB"
      vendor_target_pin "$1"
      printf 'TARGET=%s\n' "$CARGO_TARGET_DIR"
    } 2>"$TMP/stderr.txt"
  )" || RC=$?
  ERR="$(cat "$TMP/stderr.txt")"
}

head_ "1. the pin exports this checkout's vendored target"
run_pin "$TMP/checkout"
[[ "$RC" -eq 0 ]] && ok "exit 0" || bad "exit $RC: $ERR"
case "$OUT" in *"TARGET=$TMP/checkout/$V/target"*) ok "target pinned to the checkout" ;;
  *) bad "target is $OUT" ;; esac
[[ -z "$ERR" ]] && ok "nothing on stderr when nothing was inherited" || bad "stderr: $ERR"

head_ "2. an inherited foreign value is replaced and named"
run_pin "$TMP/checkout" "/somewhere/else/third_party/grok-build/target"
case "$OUT" in *"TARGET=$TMP/checkout/$V/target"*) ok "the foreign value did not survive" ;;
  *) bad "target is $OUT" ;; esac
case "$ERR" in *"ignoring inherited CARGO_TARGET_DIR=/somewhere/else/third_party/grok-build/target"*)
  ok "the inherited value is named on stderr" ;;
  *) bad "no warning: $ERR" ;; esac

head_ "3. an inherited value that already matches is silent"
run_pin "$TMP/checkout" "$TMP/checkout/$V/target"
case "$OUT" in *"TARGET=$TMP/checkout/$V/target"*) ok "target pinned to the checkout" ;;
  *) bad "target is $OUT" ;; esac
[[ -z "$ERR" ]] && ok "nothing on stderr" || bad "stderr: $ERR"

head_ "4. sourcing alone exports nothing"
RC=0
OUT="$(CARGO_TARGET_DIR=pre-existing bash -c 'source "$1"; printf "%s" "${CARGO_TARGET_DIR-unset}"' _ "$LIB")" \
  || RC=$?
[[ "$RC" -eq 0 ]] && ok "exit 0" || bad "exit $RC"
[[ "$OUT" == "pre-existing" ]] && ok "the environment is untouched until the call" \
  || bad "CARGO_TARGET_DIR=$OUT"

printf '\n%d passed, %d failed\n' "$PASS" "$FAIL"
[[ "$FAIL" -eq 0 ]] || exit 1
