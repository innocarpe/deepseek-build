#!/usr/bin/env bash
# Release orchestrator: bump -> PR -> merge -> tag -> wait for prebuilt assets -> npm publish.
#
# The standard change cycle (see docs/contributing/release-cycle.md):
#   fix -> PR (pr-authoring skill) -> merge -> ./scripts/release.sh <ver>
#   -> npm i -g @innocarpe/deepseek-build@<ver>
#
# Usage:
#   ./scripts/release.sh 4.0.4 [--desc "one-line note"]
#     [--no-publish] [--skip-bump] [--skip-pr] [--skip-tag] [--publish-only]
#     [--platform ID] [--timeout SEC] [--wait-all]
#
# Human gate: npm OTP is only asked if npm demands one (EOTP). With a
# publish-capable token (granular/automation) publish is fully automatic.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

VERSION=""
DESC=""
SKIP_BUMP=0; SKIP_PR=0; SKIP_TAG=0; NO_PUBLISH=0; WAIT_ALL=0
PLATFORM=""; TIMEOUT=5400

while [[ $# -gt 0 ]]; do
  case "$1" in
    --desc) DESC="$2"; shift 2 ;;
    --skip-bump) SKIP_BUMP=1; shift ;;
    --skip-pr) SKIP_PR=1; shift ;;
    --skip-tag) SKIP_TAG=1; shift ;;
    --no-publish) NO_PUBLISH=1; shift ;;
    --publish-only) SKIP_BUMP=1; SKIP_PR=1; SKIP_TAG=1; shift ;;
    --platform) PLATFORM="$2"; shift 2 ;;
    --timeout) TIMEOUT="$2"; shift 2 ;;
    --wait-all) WAIT_ALL=1; shift ;;
    -h|--help) sed -n '1,24p' "$0"; exit 0 ;;
    -*) echo "unknown option: $1" >&2; exit 1 ;;
    *) VERSION="$1"; shift ;;
  esac
done

if [[ -z "$VERSION" ]]; then
  echo "usage: $0 <MAJOR.MINOR.PATCH> [options]" >&2
  exit 1
fi
if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "error: '$VERSION' is not full SemVer MAJOR.MINOR.PATCH (e.g. 4.0.4)" >&2
  exit 1
fi
for c in gh node npm; do
  command -v "$c" >/dev/null 2>&1 || { echo "error: $c not found on PATH" >&2; exit 1; }
done

# --- 1. bump ----------------------------------------------------------------
if [[ "$SKIP_BUMP" -eq 0 ]]; then
  B_ARGS=()
  [[ -n "$DESC" ]] && B_ARGS+=(--desc "$DESC")
  ./scripts/bump-version.sh "$VERSION" "${B_ARGS[@]}"
fi

# --- 2. verify ---------------------------------------------------------------
echo "== verify =="
node npm/scripts/check-version-match.js
./scripts/check-semver.sh
if rg -q "fill in before merge" CHANGELOG.md 2>/dev/null; then
  echo "warn: CHANGELOG.md still has a fill-in placeholder for $VERSION — fill it before merging" >&2
fi

# --- 3. PR -------------------------------------------------------------------
if [[ "$SKIP_PR" -eq 0 ]]; then
  PREV_TAG="$(git tag --merged HEAD --sort=-v:refname | awk '!seen[$0]++' | head -1 || true)"
  OLD_VER="${PREV_TAG#v}"
  BRANCH="chore/release-${VERSION}"
  echo "== branch: $BRANCH =="
  if [[ "$(git branch --show-current)" != "$BRANCH" ]]; then
    if git show-ref --verify --quiet "refs/heads/$BRANCH"; then
      git checkout "$BRANCH"
    else
      git checkout -b "$BRANCH"
    fi
  fi
  git add Cargo.toml package.json Cargo.lock CHANGELOG.md docs/product/versions/README.md
  git commit -m "chore(release): bump ${OLD_VER:-$VERSION} to $VERSION" || true
  git push -u origin "$BRANCH"

  BODY="$(mktemp)"
  cat > "$BODY" <<EOF
## Summary

### Problem
Ship release \`$VERSION\` through the standard cycle: version bump, PR, merge,
tag, prebuilt attach, npm publish.

### What changed
- \`Cargo.toml\` / \`package.json\` → \`$VERSION\`; \`Cargo.lock\` synced
- \`CHANGELOG.md\` → \`$VERSION\` section${DESC:+ (${DESC})}
- \`docs/product/versions/README.md\` → decision-log row

### Out of scope
- No behavior change; release mechanics only.

## Testing
- [ ] \`node npm/scripts/check-version-match.js\` → ok ($VERSION)
- [ ] \`./scripts/check-semver.sh\` → ok ($VERSION)
- [ ] \`cargo check -p dsb-cli\` passes at $VERSION

## Kind / Related / Cache impact
- Kind: \`chore(release)\`. Cache impact: none (no agent/prompt/tool behavior change).

## Notes
- npm publish tries without OTP first; a one-time code is asked only if npm
  returns EOTP: \`./scripts/release.sh $VERSION --publish-only\`
- \`release-prebuilt.yml\` attaches prebuilt tarballs to the \`v$VERSION\` release;
  wait for the publishing platform's asset before \`npm publish\`.
EOF
  if [[ -x "$HOME/.local/bin/gh-public-english-gate" ]]; then
    "$HOME/.local/bin/gh-public-english-gate" --body-file "$BODY"
  fi
  PR_URL="$(gh pr create --base main --head "$BRANCH" \
    --title "chore(release): bump ${OLD_VER:-$VERSION} to $VERSION" \
    --body-file "$BODY" --label chore --label size/S)"
  echo "== PR: $PR_URL =="

  gh pr merge "$BRANCH" --merge
  STATE="$(gh pr view "$BRANCH" --json state --jq .state)"
  [[ "$STATE" == "MERGED" ]] || { echo "error: PR not merged (state=$STATE)" >&2; exit 1; }
  echo "== merged: $PR_URL =="
  # Local branch stays (main is often checked out in another worktree).
  git push origin --delete "$BRANCH" || true
fi

# --- 4. tag -------------------------------------------------------------------
if [[ "$SKIP_TAG" -eq 0 ]]; then
  git fetch origin
  git tag "v$VERSION" origin/main
  git push origin "v$VERSION"
  echo "== pushed tag v$VERSION =="
fi

# --- 5. wait for prebuilt assets ----------------------------------------------
if [[ -z "$PLATFORM" ]]; then
  PLATFORM="$(node -e "console.log(require('./npm/lib/platform').platformId()||'')" 2>/dev/null || true)"
fi
if [[ -z "$PLATFORM" ]]; then
  echo "error: could not detect platform; pass --platform <id>" >&2
  exit 1
fi
ALL_PLATFORMS=(darwin-arm64 darwin-x64 linux-x64)
WAIT_LABEL="$PLATFORM"; [[ "$WAIT_ALL" -eq 1 ]] && WAIT_LABEL="all (${ALL_PLATFORMS[*]})"
echo "== waiting for v$VERSION release assets ($WAIT_LABEL, timeout ${TIMEOUT}s) =="
DEADLINE=$(( $(date +%s) + TIMEOUT ))
ASSET_OK=0
while [[ $(date +%s) -lt $DEADLINE ]]; do
  PRESENT="$(gh release view "v$VERSION" --json assets --jq '[.assets[].name] | join(",")' 2>/dev/null || true)"
  if [[ "$WAIT_ALL" -eq 1 ]]; then
    MISSING=0
    for p in "${ALL_PLATFORMS[@]}"; do
      [[ "$PRESENT" == *"deepseek-build-${VERSION}-${p}.tar.gz"* ]] || MISSING=1
    done
    [[ "$MISSING" -eq 0 ]] && ASSET_OK=1
  else
    [[ "$PRESENT" == *"deepseek-build-${VERSION}-${PLATFORM}.tar.gz"* ]] && ASSET_OK=1
  fi
  if [[ "$ASSET_OK" -eq 1 ]]; then
    echo "== release assets ready: ${PRESENT} =="
    break
  fi
  echo "  waiting... present: ${PRESENT:-none}"
  sleep 30
done
if [[ "$ASSET_OK" -eq 0 ]]; then
  echo "error: v$VERSION assets not ready within ${TIMEOUT}s" >&2
  echo "  resume later with: $0 $VERSION --publish-only" >&2
  exit 1
fi

# --- 6. npm publish -------------------------------------------------------------
if [[ "$NO_PUBLISH" -eq 1 ]]; then
  echo "== skipping publish (--no-publish). Manual: npm publish --access public =="
  exit 0
fi
echo "== npm publish @innocarpe/deepseek-build@$VERSION =="
npm whoami >/dev/null
# Try without OTP first: a publish-capable token (granular/automation) needs no
# one-time code. Only fall back to OTP if npm actually demands one (EOTP).
ERR_LOG="$(mktemp)"
if ! npm publish --access public 2> "$ERR_LOG"; then
  if rg -qi "EOTP|one-time pass|two-factor|2fa" "$ERR_LOG"; then
    OTP="${NPM_OTP:-}"
    if [[ -z "$OTP" ]]; then
      read -rsp "npm OTP (one-time code): " OTP
      echo
    fi
    [[ -n "$OTP" ]] || { echo "error: empty OTP" >&2; rm -f "$ERR_LOG"; exit 1; }
    npm publish --access public --otp "$OTP"
  else
    cat "$ERR_LOG" >&2
    rm -f "$ERR_LOG"
    exit 1
  fi
fi
rm -f "$ERR_LOG"

echo
echo "== done. User verification: =="
echo "  npm i -g @innocarpe/deepseek-build@$VERSION"
echo "  dsb --version"
echo "  dsb --resume                    # resumes most-recent TUI session (if any)"
echo "  (quit hint shows 'dsb --resume <id>' for full-screen sessions)"
