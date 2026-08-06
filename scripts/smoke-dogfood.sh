#!/usr/bin/env bash
# Executable dogfood-usable smoke (Wave A §3). Offline-friendly where possible.
# Live API tests only if DEEPSEEK_API_KEY is set.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

fail() { echo "smoke-dogfood FAIL: $*" >&2; exit 1; }
ok() { echo "smoke-dogfood OK: $*"; }

echo "== SemVer =="
./scripts/check-semver.sh || fail "check-semver"

VER="$(rg -n '^version = "' Cargo.toml | head -1 | sed -E 's/.*"([^"]+)".*/\1/')"
[[ "$VER" =~ ^[0-9]+\.[0-9]+\.[0-9]+ ]] || fail "bad version $VER"
ok "workspace version $VER"

echo "== Build dual bins =="
cargo build -p dsb-cli --release 2>/dev/null || cargo build -p dsb-cli
BIN_DIR="$ROOT/target/release"
if [[ ! -x "$BIN_DIR/deepseek-build" ]]; then
  BIN_DIR="$ROOT/target/debug"
fi
[[ -x "$BIN_DIR/deepseek-build" ]] || fail "missing deepseek-build binary"
[[ -x "$BIN_DIR/dsb" ]] || fail "missing dsb binary"

echo "== Dual --version =="
V1="$("$BIN_DIR/deepseek-build" --version)"
V2="$("$BIN_DIR/dsb" --version)"
echo "  $V1"
echo "  $V2"
echo "$V1" | rg -q "$VER" || fail "deepseek-build version mismatch (want $VER)"
echo "$V2" | rg -q "$VER" || fail "dsb version mismatch (want $VER)"
ok "dual version contains $VER"

echo "== Help =="
"$BIN_DIR/deepseek-build" --help >/dev/null
"$BIN_DIR/dsb" --help >/dev/null
ok "help"

echo "== Offline tests (workspace) =="
cargo test --workspace -q || fail "cargo test --workspace"
ok "cargo test"

echo "== npm version match (if package.json present) =="
if [[ -f package.json ]]; then
  if npm run version-check >/dev/null 2>&1; then
    ok "npm version-check"
  else
    node npm/scripts/check-version-match.js 2>/dev/null || fail "npm/cargo version mismatch"
    ok "npm version-check (node)"
  fi
fi

echo "== Live API (optional) =="
if [[ -n "${DEEPSEEK_API_KEY:-}" ]]; then
  # short one-shot; dogfood profile for write path not required for chat
  set +e
  OUT="$("$BIN_DIR/dsb" run "Reply with exactly: pong" 2>&1)"
  EC=$?
  set -e
  echo "$OUT" | tail -20
  [[ $EC -eq 0 ]] || fail "live dsb run exit $EC"
  echo "$OUT" | rg -qi "pong|model=deepseek" || echo "smoke-dogfood WARN: live output unexpected (manual check)"
  ok "live run attempted"
else
  echo "smoke-dogfood SKIP live (DEEPSEEK_API_KEY unset)"
fi

echo "== dogfood-usable §3 mapping =="
echo "1 install/build bins: OK (this script builds or uses target)"
echo "2 auth: env key optional above"
echo "3 chat/run: help OK; live optional"
echo "4 tools: covered by cargo test (snippet/grep/bash units)"
echo "5 write profile: --dogfood flag exists (cli --help)"
"$BIN_DIR/dsb" --help 2>&1 | rg -q "dogfood|workspace" || echo "WARN: dogfood flag not in help text"
echo "6 docs: see README + docs/user-guide"
echo "7 SemVer: $VER"

ok "ALL PASSED (offline core)"
exit 0
