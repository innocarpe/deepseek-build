#!/usr/bin/env bash
# Test for scripts/install-dsb-exempt.sh and the runner it builds
# (scripts/lib/dsb-exempt.c). macOS only; everything is installed into a temp
# dir, so the maintainer's ~/Applications/DsbExempt.app is never touched.
#
#   1. install   — builds the bundle, links the binary, --check passes: the
#                  runner is its own responsible process and its child
#                  inherits it (the whole point of the runner)
#   2. idempotent — a second install from the same source leaves the bundle
#                  alone (a rebuild would void its Developer Tools entry)
#   3. passthru  — exit status, a child killed by a signal, stdin, and no
#                  leaked DSB_EXEMPT_INNER in the command's environment
#
# What this cannot test: the Developer Tools exemption itself. That needs the
# bundle listed by hand in System Settings; the effect is measured by timing a
# freshly built binary's first run (300-590 ms outside, ~2 ms inside).
#
# Usage: ./scripts/test-dsb-exempt.sh
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
INSTALL="$ROOT/scripts/install-dsb-exempt.sh"

if [[ "$(uname -s)" != Darwin ]]; then
  echo "skip: the runner is macOS only" >&2
  exit 0
fi

PASS=0
FAIL=0
TMP="$(cd "$(mktemp -d)" && pwd -P)"
trap 'rm -rf "$TMP"' EXIT
ok()  { printf '  ok   %s\n' "$*"; PASS=$((PASS + 1)); }
bad() { printf '  FAIL %s\n' "$*" >&2; FAIL=$((FAIL + 1)); }
head_() { printf '\n== %s ==\n' "$*"; }

APPS="$TMP/Applications"
BINS="$TMP/bin"
RUNNER="$BINS/dsb-exempt"

head_ "1. install"
if out="$("$INSTALL" --app-dir "$APPS" --bin-dir "$BINS" 2>&1)"; then ok "install exit 0"; else bad "install failed: $out"; fi
[[ -x "$APPS/DsbExempt.app/Contents/MacOS/dsb-exempt" ]] && ok "bundle executable present" || bad "no bundle executable"
[[ -L "$RUNNER" ]] && ok "linked into the bin dir" || bad "no link at $RUNNER"
codesign -dv "$APPS/DsbExempt.app" 2>&1 | grep -q "Identifier=dev.wooseong.dsb-exempt" \
  && ok "signed with the bundle identifier" || bad "unexpected signature"
case "$out" in *"ok: children are attributed to this app"*) ok "--check: the child is attributed to the runner" ;;
  *) bad "--check did not pass: $out" ;; esac

head_ "2. a second install leaves the bundle alone"
before="$(stat -f %m "$APPS/DsbExempt.app/Contents/MacOS/dsb-exempt")"
out="$("$INSTALL" --app-dir "$APPS" --bin-dir "$BINS" 2>&1)"
after="$(stat -f %m "$APPS/DsbExempt.app/Contents/MacOS/dsb-exempt")"
case "$out" in *"left alone"*) ok "reported as left alone" ;; *) bad "not reported: $out" ;; esac
[[ "$before" == "$after" ]] && ok "the executable was not rewritten" || bad "the executable was rewritten"

head_ "3. pass-through"
rc=0; "$RUNNER" /bin/sh -c 'exit 7' || rc=$?
[[ "$rc" -eq 7 ]] && ok "exit status 7 passes through" || bad "exit $rc, want 7"
rc=0; "$RUNNER" /bin/sh -c 'kill -TERM $$' || rc=$?
[[ "$rc" -eq 143 ]] && ok "a child killed by SIGTERM exits 143" || bad "exit $rc, want 143"
got="$(printf 'hello\n' | "$RUNNER" /bin/cat)"
[[ "$got" == "hello" ]] && ok "stdin reaches the command" || bad "stdin: got '$got'"
if "$RUNNER" /usr/bin/env | grep -q '^DSB_EXEMPT_INNER='; then bad "DSB_EXEMPT_INNER leaked"; else ok "no DSB_EXEMPT_INNER in the command's environment"; fi
rc=0; "$RUNNER" /nonexistent/cmd 2>/dev/null || rc=$?
[[ "$rc" -eq 127 ]] && ok "a missing command exits 127" || bad "exit $rc, want 127"

printf '\n%d passed, %d failed\n' "$PASS" "$FAIL"
((FAIL == 0))
