#!/usr/bin/env bash
# Package deepseek-build / dsb / deepseek-build-agent into a platform tarball
# for GitHub Releases (ADR 0009).
#
# Usage:
#   ./scripts/package-release-binaries.sh              # use ~/.deepseek-build/bin
#   ./scripts/package-release-binaries.sh --from-dir DIR
#   ./scripts/package-release-binaries.sh --upload      # gh release upload
#
# Output:
#   dist/deepseek-build-{VERSION}-{platform}.tar.gz
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

VERSION="$(node -p "require('./package.json').version")"
PLATFORM=""
FROM_DIR="${DEEPSEEK_BUILD_HOME:-$HOME/.deepseek-build}/bin"
UPLOAD=0
OUT_DIR="$ROOT/dist"

usage() {
  cat <<EOF
Usage: $0 [options]

Options:
  --from-dir DIR   Directory containing deepseek-build, dsb, deepseek-build-agent
  --platform ID    Override platform id (default: detect via node npm/lib/platform.js)
  --upload         Upload tarball to GitHub release v{VERSION} (gh)
  --out-dir DIR    Output directory (default: ./dist)
  -h, --help
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --from-dir) FROM_DIR="$2"; shift 2 ;;
    --platform) PLATFORM="$2"; shift 2 ;;
    --upload) UPLOAD=1; shift ;;
    --out-dir) OUT_DIR="$2"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "unknown option: $1" >&2; usage >&2; exit 1 ;;
  esac
done

if [[ -z "$PLATFORM" ]]; then
  PLATFORM="$(node -e "console.log(require('./npm/lib/platform').platformId()||'')")"
fi
if [[ -z "$PLATFORM" ]]; then
  echo "error: could not detect platform" >&2
  exit 1
fi

ASSET="deepseek-build-${VERSION}-${PLATFORM}.tar.gz"
mkdir -p "$OUT_DIR"
STAGE="$(mktemp -d "${TMPDIR:-/tmp}/dsb-pack.XXXXXX")"
cleanup() { rm -rf "$STAGE"; }
trap cleanup EXIT

need=(deepseek-build dsb deepseek-build-agent)
for b in "${need[@]}"; do
  if [[ ! -x "$FROM_DIR/$b" ]]; then
    echo "error: missing executable $FROM_DIR/$b" >&2
    echo "  Build/install first: cargo install --path crates/dsb-cli --locked --force --root \"\$HOME/.deepseek-build\"" >&2
    echo "  and: ./scripts/build-grok-pager.sh release && cp third_party/grok-build/target/release/xai-grok-pager* \"\$HOME/.deepseek-build/bin/deepseek-build-agent\"" >&2
    exit 1
  fi
  cp -f "$FROM_DIR/$b" "$STAGE/$b"
  chmod 755 "$STAGE/$b"
done

# Sanity: wrapper --version should match package when possible
if ! "$STAGE/deepseek-build" --version 2>/dev/null | grep -q "$VERSION"; then
  echo "warn: deepseek-build --version does not contain $VERSION (got: $("$STAGE/deepseek-build" --version 2>/dev/null || true))" >&2
  echo "  Rebuild wrappers after bumping Cargo/package.json version." >&2
fi

tar -czf "$OUT_DIR/$ASSET" -C "$STAGE" deepseek-build dsb deepseek-build-agent
echo "wrote $OUT_DIR/$ASSET ($(wc -c <"$OUT_DIR/$ASSET" | tr -d ' ') bytes)"

if [[ "$UPLOAD" -eq 1 ]]; then
  if ! command -v gh >/dev/null; then
    echo "error: gh not found" >&2
    exit 1
  fi
  if ! gh release view "v${VERSION}" >/dev/null 2>&1; then
    echo "creating GitHub release v${VERSION}…"
    gh release create "v${VERSION}" \
      --title "v${VERSION}" \
      --notes "Prebuilt natives for npm install (ADR 0009). Platform: see assets."
  fi
  gh release upload "v${VERSION}" "$OUT_DIR/$ASSET" --clobber
  echo "uploaded to release v${VERSION}"
fi
