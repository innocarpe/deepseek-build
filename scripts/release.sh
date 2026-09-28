#!/usr/bin/env bash
# Release orchestrator: bump level -> bump -> PR -> merge -> tag -> wait for
# prebuilt assets -> CI publishes to npm over OIDC -> verify the registry.
#
# The standard change cycle (see docs/contributing/release-cycle.md):
#   fix -> PR (pr-authoring skill) -> merge -> ./scripts/next-version.sh picks
#   the number -> ./scripts/release.sh <ver>
#   -> npm install -g --allow-scripts=@innocarpe/deepseek-build @innocarpe/deepseek-build@<ver>
#
# Usage:
#   ./scripts/release.sh 4.0.4 [--desc "one-line note"]
#     [--level-override "<reason>"] [--no-publish] [--skip-bump] [--skip-pr]
#     [--skip-tag] [--publish-only] [--local-publish] [--platform ID]
#     [--timeout SEC] [--checks-timeout SEC] [--wait-all]
#
# Publishing (ADR 0012): the tag push triggers .github/workflows/publish-npm.yml,
# which publishes via OIDC trusted publishing — no npm token and no one-time
# code. This script watches that run and verifies the registry afterwards.
# --local-publish is the emergency path for when CI cannot publish; it needs an
# interactive npm login and may prompt for a one-time code (EOTP).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# Help may be read anywhere. Every other invocation edits or checks out
# a branch, which the primary checkout is not allowed to do.
if [[ "${1:-}" != "-h" && "${1:-}" != "--help" ]]; then
  "$ROOT/scripts/lib/refuse-primary-checkout.sh" "$ROOT"
fi

VERSION=""
DESC=""
LEVEL_OVERRIDE=""; OVERRIDE_SET=0; PUBLISH_ONLY=0
SKIP_BUMP=0; SKIP_PR=0; SKIP_TAG=0; NO_PUBLISH=0; WAIT_ALL=0; LOCAL_PUBLISH=0
PLATFORM=""; TIMEOUT=5400
# How long to wait for the release PR's checks before merging, and how often to
# ask. The interval is env-only so the hermetic regression
# (scripts/test-release-pr-wait.sh) can shorten a wait that is 10 s in a real
# release.
CHECKS_TIMEOUT="${DSB_PR_CHECKS_TIMEOUT_SEC:-3600}"
CHECKS_INTERVAL="${DSB_PR_CHECKS_INTERVAL_SEC:-10}"
# How long to keep asking the registry whether a version is live when that
# answer only shapes a diagnostic (the failed-CI-run path below). The
# post-publish confirmation uses the helper's full default window instead.
REGISTRY_PROBE_SEC=45

while [[ $# -gt 0 ]]; do
  case "$1" in
    --desc) DESC="$2"; shift 2 ;;
    --skip-bump) SKIP_BUMP=1; shift ;;
    --skip-pr) SKIP_PR=1; shift ;;
    --skip-tag) SKIP_TAG=1; shift ;;
    --no-publish) NO_PUBLISH=1; shift ;;
    --local-publish) LOCAL_PUBLISH=1; shift ;;
    --level-override)
      if [[ $# -lt 2 ]]; then
        echo "error: --level-override needs a reason" >&2
        exit 1
      fi
      LEVEL_OVERRIDE="$2"; OVERRIDE_SET=1; shift 2 ;;
    --publish-only) SKIP_BUMP=1; SKIP_PR=1; SKIP_TAG=1; PUBLISH_ONLY=1; shift ;;
    --platform) PLATFORM="$2"; shift 2 ;;
    --timeout) TIMEOUT="$2"; shift 2 ;;
    --checks-timeout) CHECKS_TIMEOUT="$2"; shift 2 ;;
    --wait-all) WAIT_ALL=1; shift ;;
    -h|--help) sed -n '1,25p' "$0"; exit 0 ;;
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
for pair in "checks timeout=$CHECKS_TIMEOUT" "checks interval=$CHECKS_INTERVAL"; do
  label="${pair%%=*}"; value="${pair#*=}"
  if [[ ! "$value" =~ ^[0-9]+$ ]]; then
    echo "error: ${label} must be whole seconds (got '$value')" >&2
    exit 1
  fi
done
if [[ "$OVERRIDE_SET" -eq 1 && -z "$LEVEL_OVERRIDE" ]]; then
  echo "error: --level-override needs a non-empty reason (it is recorded in the release PR body)" >&2
  exit 1
fi

# --- 0. bump level -----------------------------------------------------------
# The version is a judgment of what shipped, not a habit: the first-parent
# merges on origin/main since the last release tag decide whether this release
# moves MINOR or PATCH, and a harness-only range owes no release at all
# (docs/contributing/versioning.md §1c). A request below that judgment stops
# here unless the caller names a reason; --level-override "<reason>" ships it
# anyway and the reason rides in the release PR body. --publish-only resumes a
# release whose number was already decided, so it skips the judgment (the
# released work would read as unreleased there).
if [[ "$PUBLISH_ONLY" -eq 0 ]]; then
  echo "== bump level =="
  git fetch -q origin main 2>/dev/null || \
    echo "warn: could not fetch origin main; the level check reads the local ref"
  LEVEL_RC=0
  LEVEL_OUT="$("$ROOT/scripts/next-version.sh" --check "$VERSION" 2>&1)" || LEVEL_RC=$?
  printf '%s\n' "$LEVEL_OUT"
  # Exit 1 is the tool's judgment: the requested version is below the merges
  # (or not newer than the last tag). Anything else nonzero means no judgment
  # was rendered — no tag merged yet (a first release), no repository, the
  # tool missing — and there is nothing to compare, so the release continues
  # with the reason printed.
  if [[ "$LEVEL_RC" -eq 1 ]]; then
    if [[ "$OVERRIDE_SET" -eq 1 ]]; then
      echo "warn: shipping below the computed bump level — override reason: $LEVEL_OVERRIDE"
    else
      echo "error: $VERSION is below the computed bump level" >&2
      echo "  the number comes from ./scripts/next-version.sh; to ship this one anyway:" >&2
      echo "    $0 $VERSION --level-override \"<why this version is right>\"" >&2
      echo "  the reason is recorded in the release PR body." >&2
      exit 1
    fi
  elif [[ "$LEVEL_RC" -ne 0 ]]; then
    echo "warn: the bump level could not be computed (next-version.sh exit $LEVEL_RC) — continuing without the level gate" >&2
  fi
fi

# --- 1. bump ----------------------------------------------------------------
if [[ "$SKIP_BUMP" -eq 0 ]]; then
  B_ARGS=()
  [[ -n "$DESC" ]] && B_ARGS+=(--desc "$DESC")
  # macOS /bin/bash 3.2 reads an empty "${B_ARGS[@]}" as unbound under set -u;
  # the +"..." guard expands to zero words instead.
  ./scripts/bump-version.sh "$VERSION" "${B_ARGS[@]+"${B_ARGS[@]}"}"
fi

# --- 1b. MAJOR bump: version-log announcement gate ---------------------------
# A MAJOR release must already be logged in the product version history
# (docs/product/versions/README.md — the bump step above adds the row). The
# user-facing README stays clean of release-process signals; the version log
# is the internal record the gate checks, so a tag never ships ahead of the
# documented story.
OLD_VER="$(git show HEAD:Cargo.toml | python3 -c "
import re, sys
s = sys.stdin.read()
try:
    i = s.index('[workspace.package]')
except ValueError:
    sys.exit(0)
m = re.search(r'(?m)^version = \"([^\"]+)\"', s[i:])
print(m.group(1) if m else '')
")"
NEW_MAJOR="${VERSION%%.*}"
OLD_MAJOR="${OLD_VER%%.*}"
if [[ -n "$OLD_VER" && "$NEW_MAJOR" != "$OLD_MAJOR" ]]; then
  if ! rg -q "^\| [0-9]{4}-[0-9]{2}-[0-9]{2} \| .*${NEW_MAJOR}\.[0-9]+\.[0-9]+" docs/product/versions/README.md; then
    echo "error: MAJOR bump ${OLD_VER} -> ${VERSION} requires docs/product/versions/README.md" >&2
    echo "  to already log a ${NEW_MAJOR}.x row (bump-version.sh adds it; verify it landed)." >&2
    exit 1
  fi
  echo "== version log already announces major ${NEW_MAJOR}: ok =="
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

  OVERRIDE_SECTION=""
  if [[ "$OVERRIDE_SET" -eq 1 ]]; then
    OVERRIDE_SECTION="### Bump-level override
- Shipping \`$VERSION\` below the level the merges since the last tag computed (\`./scripts/next-version.sh\`).
- Reason: $LEVEL_OVERRIDE"
  fi

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
$OVERRIDE_SECTION

## Testing
- [ ] \`node npm/scripts/check-version-match.js\` → ok ($VERSION)
- [ ] \`./scripts/check-semver.sh\` → ok ($VERSION)
- [ ] \`cargo check -p dsb-cli\` passes at $VERSION

## Kind / Related / Cache impact
- Kind: \`chore(release)\`. Cache impact: none (no agent/prompt/tool behavior change).

## Notes
- Publishing runs in CI: the \`v$VERSION\` tag push triggers \`publish-npm.yml\`,
  which publishes over OIDC trusted publishing (ADR 0012) — no token, no
  one-time code. \`release.sh\` watches that run and verifies the registry.
- Emergency fallback when CI cannot publish: a local interactive publish
  (\`./scripts/release.sh $VERSION --publish-only --local-publish\`) — see
  \`docs/contributing/release-cycle.md\` §Emergency path.
- \`release-prebuilt.yml\` attaches the prebuilt tarball to the \`v$VERSION\`
  release; \`publish-npm.yml\` waits for that asset and refuses to publish
  without it (ADR 0009).
EOF
  if [[ -x "$HOME/.local/bin/gh-public-english-gate" ]]; then
    "$HOME/.local/bin/gh-public-english-gate" --body-file "$BODY"
  fi
  PR_URL="$(gh pr create --base main --head "$BRANCH" \
    --title "chore(release): bump ${OLD_VER:-$VERSION} to $VERSION" \
    --body-file "$BODY" --label chore --label size/S)"
  echo "== PR: $PR_URL =="

  # Record the PR number in the decision-log row the bump opened. That row is
  # what the MAJOR gate reads, and the number only exists now — leaving it as
  # `PR #_(fill in)_` is how six rows on `main` shipped unrecorded.
  PR_NUM="${PR_URL##*/}"
  python3 scripts/lib/version_log.py set-pr docs/product/versions/README.md "$VERSION" "$PR_NUM"
  git add docs/product/versions/README.md
  git commit -m "docs(product): record the release PR in the $VERSION decision-log row" || true
  git push origin "$BRANCH"

  # Wait for the PR's checks before merging. Merging the instant `gh pr create`
  # returned lost the race against GitHub computing mergeability and the checks
  # starting: 6.2.0 (#258, published as 6.1.1), 6.7.1 (#311, published as
  # 6.1.7) and 6.8.0 (#323, published as 6.1.10) answered "GraphQL: Pull
  # Request is not mergeable (mergePullRequest)", and the release stopped
  # with the PR open for a person to merge. A red check must stop it here,
  # before the merge call, with what is left to do printed below.
  if ! python3 "$ROOT/scripts/lib/pr_checks.py" --pr "$PR_NUM" \
      --timeout "$CHECKS_TIMEOUT" --interval "$CHECKS_INTERVAL"; then
    echo "error: $PR_URL is not ready to merge — the release stops here" >&2
    echo "  once its checks are green, merge it by hand and resume with:" >&2
    echo "    $0 $VERSION --skip-bump --skip-pr" >&2
    exit 1
  fi
  if ! gh pr merge "$BRANCH" --merge; then
    echo "error: gh pr merge failed for $PR_URL — the release PR is still open" >&2
    echo "  merge it by hand, then resume with:" >&2
    echo "    $0 $VERSION --skip-bump --skip-pr" >&2
    exit 1
  fi
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
SUPPORTED_PLATFORM="darwin-arm64"
if [[ -z "$PLATFORM" ]]; then
  PLATFORM="$(node -e "console.log(require('./npm/lib/platform').platformId()||'')" 2>/dev/null || true)"
fi
if [[ -z "$PLATFORM" ]]; then
  PLATFORM="$SUPPORTED_PLATFORM"
fi
if [[ "$PLATFORM" != "$SUPPORTED_PLATFORM" ]]; then
  echo "error: this release harness currently supports $SUPPORTED_PLATFORM only (got $PLATFORM)" >&2
  exit 1
fi
ALL_PLATFORMS=("$SUPPORTED_PLATFORM")
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
# Default path: the tag push triggers .github/workflows/publish-npm.yml, which
# publishes over OIDC trusted publishing (ADR 0012) — no token, no one-time
# code. This script therefore only *watches* that run and verifies the result.
# A local `npm publish` is the emergency path (--local-publish) for a broken or
# unavailable workflow; see docs/contributing/release-cycle.md §Emergency path.
if [[ "$NO_PUBLISH" -eq 1 ]]; then
  echo "== skipping publish (--no-publish). CI publishes on the tag push (ADR 0012) =="
  exit 0
fi

if [[ "$LOCAL_PUBLISH" -eq 1 ]]; then
  echo "== LOCAL npm publish @innocarpe/deepseek-build@$VERSION (emergency path) =="
  echo "   CI (publish-npm.yml) is the default. See release-cycle.md §Emergency path."
  npm whoami >/dev/null
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
else
  echo "== waiting for CI publish (publish-npm.yml) of $VERSION =="
  DEADLINE=$(( $(date +%s) + TIMEOUT ))
  RUN_ID=""
  while [[ $(date +%s) -lt $DEADLINE ]]; do
    RUN_ID="$(gh run list --workflow publish-npm.yml --event push --limit 20 \
      --json databaseId,headBranch,status \
      --jq "[.[] | select(.headBranch == \"v$VERSION\")][0].databaseId" 2>/dev/null || true)"
    if [[ -n "$RUN_ID" && "$RUN_ID" != "null" ]]; then
      break
    fi
    echo "  waiting for the publish-npm run for tag v$VERSION…"
    sleep 20
  done
  if [[ -z "$RUN_ID" || "$RUN_ID" == "null" ]]; then
    echo "error: no publish-npm.yml run appeared for v$VERSION within ${TIMEOUT}s" >&2
    echo "  The trusted publisher may be unconfigured, or Actions may be queued." >&2
    echo "  Emergency path: ./scripts/release.sh $VERSION --publish-only --local-publish" >&2
    exit 1
  fi
  echo "== CI run $RUN_ID — waiting for it to finish =="
  if ! gh run watch "$RUN_ID" --exit-status >/dev/null 2>&1; then
    echo "error: publish-npm.yml run $RUN_ID did not succeed" >&2
    echo "  gh run view $RUN_ID --log-failed" >&2
    # A failed run does not always mean a failed release: on v5.7.0 the publish
    # itself succeeded and a later step went red on a registry read that raced
    # the publish, so the run's exit status alone misreports the outcome. Say
    # which case this is — still a failure, because something in CI is wrong,
    # but the reader should not have to look up whether the version shipped.
    # A short window is enough here: this runs long after the publish.
    if LIVE_NOW="$(DSB_NPM_VERIFY_TIMEOUT_SEC="${REGISTRY_PROBE_SEC}" \
        ./scripts/verify-npm-version.sh "@innocarpe/deepseek-build@$VERSION" 2>/dev/null)"; then
      echo "  note: the registry already serves ${LIVE_NOW} — the publish landed;" >&2
      echo "  the failing step is something after it (see the log above)." >&2
    else
      echo "  note: the registry does not serve $VERSION either — the publish did not land." >&2
    fi
    echo "  Emergency path: ./scripts/release.sh $VERSION --publish-only --local-publish" >&2
    exit 1
  fi
fi

# Verify the registry, whichever path published. The local tarball smoke is the
# caller's job (release skill §Post-publish verification); CI runs its own.
#
# This must not be a single immediate read: a publish that has returned can
# still be invisible for a while (measured 76 s on the v5.7.0 release), so an
# instant check fails on a healthy release. See scripts/verify-npm-version.sh
# for the measurement and the retry window. On the CI path the run above has
# already confirmed visibility, so this is a cheap re-confirmation; on the
# --local-publish path it follows `npm publish` directly and is the read that
# sits closest to the race.
if ! LIVE="$(./scripts/verify-npm-version.sh "@innocarpe/deepseek-build@$VERSION")"; then
  echo "error: the registry never served $VERSION — see the message above" >&2
  echo "  If the publish itself failed, see the run log; if it succeeded, re-check with:" >&2
  echo "    ./scripts/verify-npm-version.sh '@innocarpe/deepseek-build@$VERSION'" >&2
  exit 1
fi
echo "== registry reports @innocarpe/deepseek-build@$LIVE =="

echo
echo "== done. User verification: =="
echo "  npm install -g --allow-scripts=@innocarpe/deepseek-build @innocarpe/deepseek-build@$VERSION"
echo "  dsb --version"
echo "  dsb --resume                    # resumes most-recent TUI session (if any)"
echo "  (quit hint shows 'dsb --resume <id>' for full-screen sessions)"
echo
echo "  npm 12 blocks the install script without --allow-scripts, and the install"
echo "  still exits 0: the agent binary keeps the previous version."
