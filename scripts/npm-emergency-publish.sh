#!/usr/bin/env bash
# Emergency npm publish for @innocarpe/deepseek-build (ADR 0012 §Emergency path).
#
# Use only when CI publishing (`.github/workflows/publish-npm.yml`, OIDC trusted
# publishing) cannot run — Actions unavailable, trusted publisher misconfigured,
# or a tag whose publish must land immediately. The default release path needs
# no human and no proof of presence; this one does, so it drives that proof
# through the Aside browser agent instead of asking a person to be present.
#
# How the 2FA step actually works here (measured, not assumed):
#   The account has 2FA at `auth-and-writes` and a registered **security key**.
#   Under a terminal, npm answers an EOTP by printing a browser URL and polling
#   until that page is approved — there is no emailed code to read. Without a
#   terminal npm refuses outright, so the publish runs under a pty; with a
#   browser configured it blocks on "Press ENTER", so every call passes
#   `--browser=false` to make npm poll instead.
#
# What it does:
#   1. Refuses to run unless the release tarball for the version is attached to
#      the GitHub release (ADR 0009 / release skill hard rule 5).
#   2. Ensures an npm session, completing `npm login --auth-type=web` through
#      the browser agent if there is none.
#   3. Runs `npm publish` under a pty, captures the 2FA approval URL npm prints,
#      and hands it to `aside exec` to approve with the security key.
#   4. Verifies the registry.
#
# No code, token or single-use URL is written to a log, a commit or a PR.
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
    -h|--help) sed -n '1,29p' "$0"; exit 0 ;;
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

# Account and mailbox are configuration, not repo content: this repo is public,
# so no personal address is committed. The npm profile supplies both.
NPM_ACCOUNT="${NPM_ACCOUNT:-$(npm whoami 2>/dev/null || echo 'the npm account')}"
NPM_MAILBOX="${NPM_MAILBOX:-$(npm profile get email 2>/dev/null | sed -E 's/^email: *//; s/ \(.*\)$//' || true)}"
NPM_MAILBOX="${NPM_MAILBOX:-the mailbox that receives npm mail}"

aside_ok() { command -v aside >/dev/null 2>&1; }

# Wait for `npm` to print a browser-approval URL, echo it, or fail on timeout.
# $1 = log file, $2 = seconds to wait
wait_for_auth_url() {
  local log="$1" limit="$2" url=""
  for _ in $(seq 1 "$limit"); do
    url="$(rg -o 'https://www\.npmjs\.com/auth/cli/[A-Za-z0-9_-]+' "$log" 2>/dev/null | head -1 || true)"
    [[ -n "$url" ]] && { printf '%s' "$url"; return 0; }
    sleep 1
  done
  return 1
}

# $1 = URL, $2 = what is being approved (for the prompt text)
approve_in_browser() {
  local url="$1" what="$2"
  aside exec --permission full-access "Open this exact npm URL in the browser and complete the authentication it asks for: ${url}
This is an npm CLI approval page for the npm account ${NPM_ACCOUNT}. ${what}
If it asks for two-factor authentication, approve it with the account's registered security key. If it asks for an emailed code instead, read the newest code from the mailbox ${NPM_MAILBOX} and enter it.
Do NOT change any npm account or package setting (no tokens, no 2FA changes, no publishing access, no other packages). Only approve this pending CLI session. Report plainly whether the approval succeeded, and never print codes, URLs or tokens."
}

echo "== 2/4 npm session =="
if npm whoami >/dev/null 2>&1; then
  echo "an npm session already exists (npm whoami = $(npm whoami))"
else
  if ! aside_ok; then
    echo "error: no npm session and the 'aside' CLI is not installed" >&2
    echo "  Complete 'npm login --auth-type=web' yourself, then re-run this script." >&2
    exit 1
  fi

  LOGIN_LOG="$(mktemp)"
  cleanup() { rm -f "$LOGIN_LOG"; }
  trap cleanup EXIT

  # npm prints "Login at: <url>" and polls until the browser finishes.
  # --browser=false keeps it from blocking on "Press ENTER to open…".
  npm login --auth-type=web --browser=false > "$LOGIN_LOG" 2>&1 &
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

  approve_in_browser "$LOGIN_URL" "Approve the login so the CLI session becomes authenticated."

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

echo "== 3/4 npm publish @innocarpe/deepseek-build@${VERSION} =="
if [[ "$DRY_RUN" -eq 1 ]]; then
  echo "(dry-run: stopping before publish)"
  npm pack --dry-run >/dev/null
  echo "dry-run ok — package packs cleanly; not published"
  exit 0
fi

if [[ "$ASSUME_YES" -eq 0 ]]; then
  read -r -p "publish @innocarpe/deepseek-build@${VERSION} to the public registry? [y/N] " reply
  [[ "$reply" =~ ^[Yy]$ ]] || { echo "aborted"; exit 1; }
fi

# --provenance needs a CI OIDC provider; a local publish cannot produce an
# attestation, so this path publishes without one. Say so plainly rather than
# implying the release is provenance-signed.
echo "note: publishing without --provenance (no OIDC provider locally)"

if [[ "$ASSUME_YES" -eq 1 && -z "${NPM_OTP:-}" ]]; then
  echo "note: --yes given, so this will not prompt; 2FA is approved in the browser"
fi

# A pty is required: npm only takes its browser-approval path when stdin and
# stdout are terminals. Piped, it answers EOTP and exits without offering the
# approval URL. `script` allocates the pty and still propagates the exit code.
# `--browser=false` is also required: with a browser configured, npm prints
# "Press ENTER to open in the browser..." and blocks on that read instead of
# polling the approval URL, which would hang this script forever.
PUB_LOG="$(mktemp)"
cleanup_pub() { rm -f "$PUB_LOG"; }
trap cleanup_pub EXIT

PUB_CMD=(npm publish --access public --browser=false)
if [[ -n "${NPM_OTP:-}" ]]; then
  PUB_CMD=(npm publish --access public --browser=false --otp "$NPM_OTP")
fi

if command -v script >/dev/null 2>&1; then
  script -q /dev/null "${PUB_CMD[@]}" > "$PUB_LOG" 2>&1 &
else
  echo "warn: 'script' not found — running npm without a pty (2FA approval may not be offered)" >&2
  "${PUB_CMD[@]}" > "$PUB_LOG" 2>&1 &
fi
PUB_PID=$!

# If npm asks for 2FA, approve it in the browser; otherwise this times out
# harmlessly while the publish proceeds.
AUTH_URL="$(wait_for_auth_url "$PUB_LOG" 60 || true)"
if [[ -n "$AUTH_URL" ]]; then
  echo "2FA required — approving in the browser (URL not printed)"
  approve_in_browser "$AUTH_URL" "Approve the pending npm publish for @innocarpe/deepseek-build@${VERSION}."
else
  echo "no browser approval requested (session already sufficient, or an OTP prompt is waiting)"
fi

WAITED=0
while kill -0 "$PUB_PID" 2>/dev/null; do
  if [[ "$WAITED" -ge 600 ]]; then
    echo "error: npm publish still running after 600s" >&2
    kill "$PUB_PID" 2>/dev/null || true
    exit 1
  fi
  sleep 5
  WAITED=$((WAITED + 5))
done
wait "$PUB_PID" && PUB_RC=0 || PUB_RC=$?

# Show the outcome with the single-use URL redacted.
sed -E 's#(auth/cli/)[A-Za-z0-9_-]+#\1<redacted>#g; s#(authId=)[A-Za-z0-9_-]+#\1<redacted>#g' "$PUB_LOG" | tail -20
rm -f "$PUB_LOG"
trap - EXIT

if [[ "$PUB_RC" -ne 0 ]]; then
  cat >&2 <<EOF
error: publish failed (exit ${PUB_RC}).

If the output mentions EOTP or a one-time password, npm wanted proof of presence
and the browser approval above did not land. Two ways forward:

  # a) approve in the browser on the next attempt (this script's normal path), or
  # b) with an authenticator app, pass the current code:
  NPM_OTP=<code> $0 ${VERSION} --yes

If the output is ENEEDAUTH/403, the session expired — re-run this script.

CI (publish-npm.yml, OIDC) remains the default path; prefer fixing that.
EOF
  exit 1
fi

echo "== published (no provenance attestation) =="

echo "== 4/4 verifying the registry =="
# A publish that returned can still be invisible for a while (measured 76 s on
# the v5.7.0 release), so this retries instead of reading once. This is the
# read that sits closest to the race — it follows `npm publish` directly.
if ! LIVE="$(./scripts/verify-npm-version.sh "@innocarpe/deepseek-build@${VERSION}")"; then
  echo "error: the registry never served ${VERSION} — see the message above" >&2
  echo "  The publish may still have landed; re-check with:" >&2
  echo "    ./scripts/verify-npm-version.sh '@innocarpe/deepseek-build@${VERSION}'" >&2
  exit 1
fi
echo "registry confirms @innocarpe/deepseek-build@${LIVE}"
echo
echo "Verify like a user:"
echo "  npm i -g @innocarpe/deepseek-build@${VERSION}"
echo "  dsb --version"