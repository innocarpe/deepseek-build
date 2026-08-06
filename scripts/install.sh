#!/usr/bin/env bash
# Install DeepSeek Build CLI binaries onto PATH.
#
# Installs both commands (ADR 0006):
#   - deepseek-build  (primary)
#   - dsb             (alias)
#
# Usage:
#   ./scripts/install.sh                 # → ~/.deepseek-build/bin (default)
#   ./scripts/install.sh --cargo         # → $CARGO_HOME/bin (usually ~/.cargo/bin)
#   ./scripts/install.sh --prefix DIR    # → DIR/bin via cargo install --root
#
# After install, ensure the bin dir is on PATH (script prints a note if missing).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

DEFAULT_PREFIX="${HOME}/.deepseek-build"
METHOD="prefix"
PREFIX="${DEEPSEEK_BUILD_INSTALL_PREFIX:-$DEFAULT_PREFIX}"

usage() {
  cat <<'EOF'
Install deepseek-build and dsb (same program, same SemVer).

Usage:
  ./scripts/install.sh [options]

Options:
  --cargo           Install into Cargo's bin dir ($CARGO_HOME/bin, default ~/.cargo/bin)
  --prefix DIR      Install into DIR/bin (default: ~/.deepseek-build)
  -h, --help        Show this help

Environment:
  DEEPSEEK_BUILD_INSTALL_PREFIX   Default prefix when not using --cargo
                                  (default: ~/.deepseek-build)

Examples:
  ./scripts/install.sh
  ./scripts/install.sh --cargo
  ./scripts/install.sh --prefix "$HOME/.local"

Smoke (after PATH is set):
  deepseek-build --version
  dsb --version
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --cargo)
      METHOD="cargo"
      shift
      ;;
    --prefix)
      if [[ $# -lt 2 ]]; then
        echo "install.sh: --prefix requires a directory" >&2
        exit 1
      fi
      METHOD="prefix"
      PREFIX="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "install.sh: unknown option: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

if ! command -v cargo >/dev/null 2>&1; then
  echo "install.sh: cargo not found. Install Rust (https://rustup.rs/) first." >&2
  exit 1
fi

if [[ ! -f "$ROOT/crates/dsb-cli/Cargo.toml" ]]; then
  echo "install.sh: expected crates/dsb-cli under repo root: $ROOT" >&2
  exit 1
fi

echo "install.sh: building and installing dsb-cli (deepseek-build + dsb)…"

if [[ "$METHOD" == "cargo" ]]; then
  cargo install --path "$ROOT/crates/dsb-cli" --locked --force
  CARGO_HOME_DIR="${CARGO_HOME:-$HOME/.cargo}"
  BIN_DIR="${CARGO_HOME_DIR}/bin"
else
  mkdir -p "$PREFIX"
  cargo install --path "$ROOT/crates/dsb-cli" --locked --force --root "$PREFIX"
  BIN_DIR="$PREFIX/bin"
fi

if [[ ! -x "$BIN_DIR/deepseek-build" || ! -x "$BIN_DIR/dsb" ]]; then
  echo "install.sh: expected both binaries under $BIN_DIR" >&2
  ls -la "$BIN_DIR" 2>/dev/null || true
  exit 1
fi

PRIMARY_VER="$("$BIN_DIR/deepseek-build" --version 2>&1 || true)"
ALIAS_VER="$("$BIN_DIR/dsb" --version 2>&1 || true)"

echo "install.sh: installed:"
echo "  $BIN_DIR/deepseek-build  →  $PRIMARY_VER"
echo "  $BIN_DIR/dsb             →  $ALIAS_VER"

# PATH check (best-effort; do not fail install if shell config is nonstandard)
path_has_bin=0
case ":$PATH:" in
  *":$BIN_DIR:"*) path_has_bin=1 ;;
esac

if [[ "$path_has_bin" -eq 1 ]]; then
  echo "install.sh: $BIN_DIR is already on PATH"
  echo "install.sh: smoke: deepseek-build --version && dsb --version"
else
  echo ""
  echo "install.sh: add this directory to your PATH (not currently present):"
  echo "  export PATH=\"$BIN_DIR:\$PATH\""
  echo ""
  echo "Example (zsh, permanent):"
  echo "  echo 'export PATH=\"$BIN_DIR:\$PATH\"' >> ~/.zshrc && source ~/.zshrc"
  echo ""
  echo "Then smoke:"
  echo "  deepseek-build --version"
  echo "  dsb --version"
fi
