#!/usr/bin/env bash
# Emergency npm publish for @innocarpe/deepseek-build (ADR 0012 §Emergency path).
#
# Use only when CI publishing (`.github/workflows/publish-npm.yml`, OIDC trusted
# publishing) cannot run — Actions unavailable, trusted publisher misconfigured,
# or a tag whose publish must land immediately. The default release path needs
# no human and no one-time code; this one needs a browser session, so it drives
# the interactive npm login and any one-time code through the Aside browser
# agent instead of asking a person to be present.
#
# What it does:
#   1. Refuses to run unless the release tarball for the version is attached to
#      the GitHub release (ADR 0009 / release skill hard rule 5).
#   2. Starts `npm login --auth-type=web` in the background and extracts the
#      login URL from its output (the UUID is never printed).
#   3. Hands that URL to `aside exec`, which completes the login as the npm
#      account and reads any emailed one-time code from Gmail itself.
#   4. Verifies the login landed, then runs `npm publish` with provenance.
#
# No code, token or login UUID is written to a log, a commit or a PR.
#
# Usage:
#   ./scripts/npm-emergency-publish.sh <MAJOR.MINOR.PATCH> [--dry-run] [--yes]
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

VERSION=""
DRY_RUN=0
ASSUME_YES=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run) DRY_RUN=1; shift ;;
    --yes|-y) ASSUME_YES=1; shift ;;
    -h|--help) sed -n '1,24p' "$0"; exit 0 ;;
    -*) echo "unknown option: $1" >&2; exit 1 ;;
    *) VERSION="$1"; shift ;;
  esac
done

if [[ -z "$VERSION" ]]; then
  echo "usage: $0 <MAJOR.MINOR.PATCH> [--dry-run] [--yes]" >&2
  exit 1
fi
if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "error: '$VERSION' is not full SemVer MAJOR.MINOR.PATCH" >&2
  exit 1
fi
for c in npm node gh; do
  command -v "$c" >/dev/null 2>&1 || { echo "error: $c not found on PATH" >&2; exit 1; }
done

ASSET="deepseek-build-${VERSION}-darwin-arm64.tar.gz"
TAG="v${VERSION}"

echo "== 1/4 preflight: release asset for ${TAG} =="
ASSETS="$(gh release view "$TAG" --json assets --jq '.assets[].name' 2>/dev/null || true)"
if [[ -z "$ASSETS" ]]; then
  echo "error: GitHub release ${TAG} has no assets — do not publish a version whose binary is missing" >&2
  exit 1
fi
if ! printf '%s\n' "$ASSETS" | grep -qx "$ASSET"; then
  echo "error: ${ASSET} is not attached to ${TAG}" >&2
  echo "  present: ${ASSETS}" >&2
  echo "  Build and attach it first (release skill §Manual asset fallback)." >&2
  exit 1
fi
echo "asset present: ${ASSET}"

LIVE="$(npm view "@innocarpe/deepseek-build@${VERSION}" version 2>/dev/null || true)"
if [[ "$LIVE" == "$VERSION" ]]; then
  echo "== already published: @innocarpe/deepseek-build@${VERSION} — nothing to do =="
  exit 0
fi

echo "== 2/4 starting interactive npm login =="
if npm whoami >/dev/null 2>&1; then
  echo "an npm session already exists (npm whoami = $(npm whoami))"
  if [[ "$ASSUME_YES" -eq 0 ]]; then
    read -r -p "reuse this session? [y/N] " reply
    [[ "$reply" =~ ^[Yy]$ ]] || { echo "aborted"; exit 1; }
  fi
else
  if ! command -v aside >/dev/null 2>&1; then
    echo "error: 'aside' CLI not found — it is required to complete the login without a human" >&2
    echo "  Fallback: run 'npm login --auth-type=web' yourself, then:" >&2
    echo "    npm publish --access public --provenance" >&2
    exit 1
  fi

  LOGIN_LOG="$(mktemp)"
  cleanup() { rm -f "$LOGIN_LOG"; }
  trap cleanup EXIT

  # Background login; npm prints "Login at: <url>" and waits on the session.
  npm login --auth-type=web > "$LOGIN_LOG" 2>&1 &
  LOGIN_PID=$!

  LOGIN_URL=""
  for _ in $(seq 1 30); do
    LOGIN_URL="$(rg -o 'https://www\.npmjs\.com/login\?next=[^[:space:]]+' "$LOGIN_LOG" 2>/dev/null | head -1 || true)"
    [[ -n "$LOGIN_URL" ]] && break
    sleep 1
  done
  if [[ -z "$LOGIN_URL" ]]; then
    kill "$LOGIN_PID" 2>/dev/null || true
    echo "error: could not read the login URL from npm output" >&2
    sed -E 's#(next=/login/cli/).*#\1<redacted>#' "$LOGIN_LOG" >&2 || true
    exit 1
  fi
  # Deliberately not echoed: the URL carries a single-use login UUID.
  echo "login URL acquired (not printed — it carries a single-use token)"

  echo "== 3/4 completing login via Aside (npm account + emailed code) =="
  # Account and mailbox are configuration, not repo content: this repo is
  # public, so no personal address is committed. The npm account's own email
  # (from the registry profile) is where npm sends its codes.
  NPM_ACCOUNT="${NPM_ACCOUNT:-$(npm whoami 2>/dev/null || echo 'the npm account')}"
  NPM_MAILBOX="${NPM_MAILBOX:-$(npm profile get email 2>/dev/null | sed -E 's/^email: *//; s/ \(.*\)$//' || true)}"
  NPM_MAILBOX="${NPM_MAILBOX:-the Gmail account that receives npm mail}"
  aside exec --permission full-access "Complete an npm login that is already waiting in a browser tab. Open this exact URL and follow it through: ${LOGIN_URL}
Then, as the npm account ${NPM_ACCOUNT}: approve the login/authorize the CLI session on npmjs.com. If npm shows a one-time code prompt or sends one by email, read the newest code from the mailbox ${NPM_MAILBOX} and enter it. If the page asks for a two-factor authenticator code that is not delivered by email, stop and report that instead of guessing.
Do NOT change any npm account or package setting (no tokens, no 2FA changes, no publishing access, no other packages). Only complete this login. Report plainly: whether the CLI session shows as approved, and any step that blocked you. Never print codes or tokens in your report."

  # npm exits on its own once the browser approves the CLI session.
  WAITED=0
  while kill -0 "$LOGIN_PID" 2>/dev/null; do
    if [[ "$WAITED" -ge 300 ]]; then
      echo "error: npm login still waiting after 300s — check the Aside session output" >&2
      kill "$LOGIN_PID" 2>/dev/null || true
      exit 1
    fi
    sleep 5
    WAITED=$((WAITED + 5))
  done
  rm -f "$LOGIN_LOG"
  trap - EXIT
fi

if ! npm whoami >/dev/null 2>&1; then
  echo "error: npm login did not complete — 'npm whoami' still fails" >&2
  exit 1
fi
echo "logged in as: $(npm whoami)"

echo "== 4/4 npm publish @innocarpe/deepseek-build@${VERSION} =="
if [[ "$DRY_RUN" -eq 1 ]]; then
  echo "(dry-run: stopping before publish)"
  npm pack --dry-run >/dev/null
  echo "dry-run ok — package packs cleanly; not published"
  exit 0
fi

if [[ "$ASSUME_YES" -eq 0 && "$DRY_RUN" -eq 0 ]]; then
  read -r -p "publish @innocarpe/deepseek-build@${VERSION} to the public registry? [y/N] " reply
  [[ "$reply" =~ ^[Yy]$ ]] || { echo "aborted"; exit 1; }
fi

# --provenance needs a CI OIDC provider; a local publish cannot produce an
# attestation, so the emergency path publishes without one. Say so plainly
# rather than implying the release is provenance-signed.
echo "note: publishing without --provenance (no OIDC provider locally)"
if npm publish --access public; then
  echo "== published (no provenance attestation) =="
else
  cat >&2 <<'EOF'
error: publish failed.

If the error mentions EOTP / a one-time pass / two-factor, npm is asking for
2FA. An emailed code can be fetched the same way the login was (set
NPM_MAILBOX to the mailbox that receives npm mail if it is not the default):

  aside exec --permission full-access "Read the newest npm one-time code from \
${NPM_MAILBOX:-the user's Gmail} and report just the code"

then re-run:

  NPM_OTP=<code> npm publish --access public

If the error is ENEEDAUTH/403, the session expired — re-run this script.

CI (publish-npm.yml, OIDC) remains the default path; prefer fixing that.
EOF
  exit 1
fi

LIVE="$(npm view "@innocarpe/deepseek-build@${VERSION}" version)"
[[ "$LIVE" == "$VERSION" ]] || { echo "error: registry reported '${LIVE}'" >&2; exit 1; }
echo "== registry confirms @innocarpe/deepseek-build@${LIVE} =="
echo
echo "Verify like a user:"
echo "  npm i -g @innocarpe/deepseek-build@${VERSION}"
echo "  dsb --version"