#!/usr/bin/env bash
# Build or check the vendored Grok pager composition root (ADR-0008).
#
# Usage:
#   ./scripts/build-grok-pager.sh check     # cargo check -p xai-grok-pager-bin
#   ./scripts/build-grok-pager.sh release   # cargo build -p xai-grok-pager-bin --release
#   ./scripts/build-grok-pager.sh bin-path  # print release binary path if present
#
# Host tools: Rust (vendor rust-toolchain), protoc and/or dotslash.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
VENDOR="${ROOT}/third_party/grok-build"
PKG="xai-grok-pager-bin"
CMD="${1:-check}"

if [[ ! -d "$VENDOR" ]]; then
  echo "build-grok-pager: missing vendor tree: $VENDOR" >&2
  echo "See docs/architecture/GROK_VENDOR.md and ADR-0008." >&2
  exit 1
fi

if [[ ! -f "$VENDOR/SOURCE_REV" ]]; then
  echo "build-grok-pager: missing SOURCE_REV pin under vendor tree" >&2
  exit 1
fi

# Prefer Homebrew / cargo bin for protoc + dotslash on macOS/Linux dev machines.
export PATH="${HOME}/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:${PATH}"

if ! command -v cargo >/dev/null 2>&1; then
  echo "build-grok-pager: cargo not found" >&2
  exit 1
fi

if ! command -v protoc >/dev/null 2>&1 && ! command -v dotslash >/dev/null 2>&1; then
  echo "build-grok-pager: need protoc on PATH or cargo-installed dotslash (vendor bin/protoc uses dotslash)" >&2
  echo "  brew install protobuf   # or apt install protobuf-compiler" >&2
  echo "  cargo install dotslash --locked" >&2
  exit 1
fi

cd "$VENDOR"

case "$CMD" in
  check)
    echo "build-grok-pager: SOURCE_REV=$(tr -d '\n' < SOURCE_REV)"
    echo "build-grok-pager: cargo check -p ${PKG}"
    cargo check -p "${PKG}"
    ;;
  release)
    echo "build-grok-pager: SOURCE_REV=$(tr -d '\n' < SOURCE_REV)"
    echo "build-grok-pager: cargo build -p ${PKG} --release"
    cargo build -p "${PKG}" --release
    BIN="${VENDOR}/target/release/xai-grok-pager-bin"
    # Cargo may emit a different binary name; locate common names.
    if [[ ! -x "$BIN" ]]; then
      for cand in \
        "${VENDOR}/target/release/xai-grok-pager-bin" \
        "${VENDOR}/target/release/grok" \
        "${VENDOR}/target/release/xai-grok-pager"; do
        if [[ -x "$cand" ]]; then
          BIN="$cand"
          break
        fi
      done
    fi
    if [[ ! -x "$BIN" ]]; then
      echo "build-grok-pager: release binary not found under target/release/" >&2
      ls -la "${VENDOR}/target/release/" 2>/dev/null | head -40 || true
      exit 1
    fi
    echo "build-grok-pager: built $BIN"
    ;;
  bin-path)
    for cand in \
      "${VENDOR}/target/release/xai-grok-pager-bin" \
      "${VENDOR}/target/release/grok" \
      "${VENDOR}/target/release/xai-grok-pager"; do
      if [[ -x "$cand" ]]; then
        echo "$cand"
        exit 0
      fi
    done
    echo "build-grok-pager: no release binary; run: $0 release" >&2
    exit 1
    ;;
  -h|--help|help)
    sed -n '2,12p' "$0"
    exit 0
    ;;
  *)
    echo "build-grok-pager: unknown command: $CMD" >&2
    exit 1
    ;;
esac
