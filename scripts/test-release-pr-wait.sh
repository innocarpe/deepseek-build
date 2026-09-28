#!/usr/bin/env bash
# Regression: the release PR is not merged before its checks finish.
#
# Measured three times (2026-09-27/28): release.sh merged the instant
# `gh pr create` returned and GitHub, still computing mergeability while the
# checks were starting, answered
#
#   6.7.0  (PR #303, published as 6.1.6)   Base branch was modified
#   6.7.1  (PR #311, published as 6.1.7)   GraphQL: Pull Request is not mergeable (mergePullRequest)
#   6.8.0  (PR #323, published as 6.1.10)  GraphQL: Pull Request is not mergeable (mergePullRequest)
#
# Each time a person merged the PR and resumed the script by hand.
#
# Cases:
#   1. empty rollup, then pending, then green — the script waits and merges,
#      and the run reaches the tag step (exit 0)
#   2. a check fails while others still run — stop before `gh pr merge`
#   3. `gh pr merge` itself fails — exit 1 with the PR and the resume command
#   4. the PR conflicts with its base — stop before `gh pr merge`
#   5. checks never finish — `--checks-timeout` bounds the wait
#
# Hermetic: a temp sandbox holds a copy of the real release.sh and
# scripts/lib/pr_checks.py, a throwaway git repo with a bare origin, and a fake
# `gh` that replays fixture JSON. Nothing reaches GitHub, nothing is published,
# no release is cut. Local-only by contract; not wired into CI.
#
# Usage: ./scripts/test-release-pr-wait.sh
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
RELEASE="$ROOT/scripts/release.sh"
PR_CHECKS="$ROOT/scripts/lib/pr_checks.py"
SYSTEM_BASH="/bin/bash"
VERSION="9.9.9"
PR_NUMBER="4242"

PASS=0
FAIL=0
ok()  { printf '  ok   %s\n' "$*"; PASS=$((PASS + 1)); }
bad() { printf '  FAIL %s\n' "$*" >&2; FAIL=$((FAIL + 1)); }
head_() { printf '\n== %s ==\n' "$*"; }
dump() { sed -n '1,24p' "$1" >&2 2>/dev/null || true; }

[[ -f "$RELEASE" ]] || { echo "error: $RELEASE missing" >&2; exit 1; }
[[ -f "$PR_CHECKS" ]] || { echo "error: $PR_CHECKS missing" >&2; exit 1; }
if [[ ! -x "$SYSTEM_BASH" ]]; then
  echo "skip: $SYSTEM_BASH not found; release.sh runs under the system bash" >&2
  exit 0
fi

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

# --- fixtures -----------------------------------------------------------------
empty_json() {
  cat > "$1" <<'EOF'
{"state":"OPEN","mergeable":"UNKNOWN","statusCheckRollup":[]}
EOF
}
pending_json() {
  cat > "$1" <<'EOF'
{"state":"OPEN","mergeable":"UNKNOWN","statusCheckRollup":[
 {"__typename":"CheckRun","name":"changes","status":"COMPLETED","conclusion":"SUCCESS","workflowName":"CI"},
 {"__typename":"CheckRun","name":"test","status":"IN_PROGRESS","conclusion":null,"workflowName":"CI"},
 {"__typename":"CheckRun","name":"required","status":"QUEUED","conclusion":null,"workflowName":"CI"},
 {"__typename":"StatusContext","context":"ci/third-party","state":"PENDING"}]}
EOF
}
green_json() {
  cat > "$1" <<'EOF'
{"state":"OPEN","mergeable":"MERGEABLE","statusCheckRollup":[
 {"__typename":"CheckRun","name":"changes","status":"COMPLETED","conclusion":"SUCCESS","workflowName":"CI"},
 {"__typename":"CheckRun","name":"test","status":"COMPLETED","conclusion":"SUCCESS","workflowName":"CI"},
 {"__typename":"CheckRun","name":"changelog move","status":"COMPLETED","conclusion":"SUCCESS","workflowName":"CI"},
 {"__typename":"CheckRun","name":"required","status":"COMPLETED","conclusion":"SUCCESS","workflowName":"CI"},
 {"__typename":"StatusContext","context":"ci/third-party","state":"SUCCESS"}]}
EOF
}
fail_json() {
  cat > "$1" <<'EOF'
{"state":"OPEN","mergeable":"MERGEABLE","statusCheckRollup":[
 {"__typename":"CheckRun","name":"test","status":"COMPLETED","conclusion":"FAILURE","workflowName":"CI"},
 {"__typename":"CheckRun","name":"changelog move","status":"COMPLETED","conclusion":"CANCELLED","workflowName":"CI"},
 {"__typename":"CheckRun","name":"required","status":"IN_PROGRESS","conclusion":null,"workflowName":"CI"}]}
EOF
}
conflict_json() {
  cat > "$1" <<'EOF'
{"state":"OPEN","mergeable":"CONFLICTING","statusCheckRollup":[
 {"__typename":"CheckRun","name":"test","status":"COMPLETED","conclusion":"SUCCESS","workflowName":"CI"}]}
EOF
}

# --- fake gh ------------------------------------------------------------------
write_gh_stub() { # <path>
  cat > "$1" <<'STUB'
#!/usr/bin/env bash
# Fake `gh` for test-release-pr-wait.sh: records every call and replays the
# fixture the case staged. Only the calls release.sh makes appear here.
set -u
printf '%s\n' "$*" >> "$GH_CALLS"

case "$1 $2" in
  "pr create")
    printf 'https://github.com/innocarpe/deepseek-build/pull/%s\n' "$GH_PR_NUMBER"
    ;;
  "pr view")
    case "$*" in
      *statusCheckRollup*)
        poll=$(( $(cat "$GH_STATE_DIR/poll" 2>/dev/null || echo 0) + 1 ))
        printf '%s\n' "$poll" > "$GH_STATE_DIR/poll"
        printf -v file '%s/responses/%03d.json' "$GH_STATE_DIR" "$poll"
        if [[ ! -f "$file" ]]; then
          file="$(ls "$GH_STATE_DIR"/responses/*.json | tail -n 1)"
        fi
        cat "$file"
        ;;
      *)
        printf '%s\n' "${GH_PR_STATE:-MERGED}"
        ;;
    esac
    ;;
  "pr merge")
    if [[ "${GH_MERGE_FAIL:-0}" == "1" ]]; then
      echo "GraphQL: Pull Request is not mergeable (mergePullRequest)" >&2
      exit 1
    fi
    ;;
  "release view")
    printf 'deepseek-build-%s-darwin-arm64.tar.gz\n' "$GH_VERSION"
    ;;
  *)
    echo "fake gh: unhandled call: $*" >&2
    exit 1
    ;;
esac
STUB
  chmod +x "$1"
}

# --- sandbox ------------------------------------------------------------------
# make_case <name>: fresh sandbox repo + bare origin + fake gh + call log.
# Sets CASE, SB, STATE, CALLS, BINPATH.
make_case() {
  CASE="$TMP/$1"
  SB="$CASE/sandbox"
  STATE="$CASE/state"
  CALLS="$STATE/calls.log"
  BINPATH="$CASE/bin"

  mkdir -p "$SB/scripts/lib" "$SB/npm/scripts" "$SB/docs/product/versions" \
           "$BINPATH" "$STATE/responses" "$CASE/home"

  cp "$RELEASE" "$SB/scripts/release.sh"
  cp "$PR_CHECKS" "$SB/scripts/lib/pr_checks.py"

  # The steps before the PR stage are stubbed — this test is about the wait and
  # the merge, not the bump. gh/node/npm are on PATH and stubbed so nothing
  # reaches the network (test-release-args.sh does the same).
  printf '#!/usr/bin/env bash\nexit 0\n' > "$SB/scripts/lib/refuse-primary-checkout.sh"
  printf '#!/usr/bin/env bash\nexit 0\n' > "$SB/scripts/bump-version.sh"
  printf '#!/usr/bin/env bash\nexit 0\n' > "$SB/scripts/check-semver.sh"
  printf 'import sys\nsys.exit(0)\n'          > "$SB/scripts/lib/version_log.py"
  printf '#!/usr/bin/env bash\nexit 0\n'     > "$BINPATH/node"
  printf '#!/usr/bin/env bash\nexit 0\n'     > "$BINPATH/npm"
  chmod +x "$SB/scripts/lib/refuse-primary-checkout.sh" "$SB/scripts/bump-version.sh" \
           "$SB/scripts/check-semver.sh" "$BINPATH/node" "$BINPATH/npm"
  write_gh_stub "$BINPATH/gh"

  printf '[workspace.package]\nversion = "9.9.8"\n' > "$SB/Cargo.toml"
  : > "$SB/package.json"; : > "$SB/Cargo.lock"; : > "$SB/CHANGELOG.md"
  printf '# Version history\n' > "$SB/docs/product/versions/README.md"

  git init -q -b main "$SB"
  git -C "$SB" config user.name "Release Test"
  git -C "$SB" config user.email "release-test@example.invalid"
  git init -q --bare "$CASE/origin.git"
  git -C "$SB" remote add origin "$CASE/origin.git"
  git -C "$SB" add -A
  git -C "$SB" commit -qm "seed"
  git -C "$SB" push -q origin main
}

run_release() { # <release args...>; sets RC
  RC=0
  ( cd "$SB" && \
    PATH="$BINPATH:$PATH" HOME="$CASE/home" \
    GH_CALLS="$CALLS" GH_STATE_DIR="$STATE" GH_PR_NUMBER="$PR_NUMBER" \
    GH_VERSION="$VERSION" GH_MERGE_FAIL="${GH_MERGE_FAIL:-0}" \
    "$SYSTEM_BASH" ./scripts/release.sh "$@" ) \
    > "$STATE/out" 2> "$STATE/err" || RC=$?
}

merges() { grep -c '^pr merge' "$CALLS" 2>/dev/null || true; }

# ---------------------------------------------------------------------------
head_ "1. empty rollup, then pending, then green — wait, merge, tag (bash 3.2)"
make_case wait-then-merge
empty_json   "$STATE/responses/001.json"
pending_json "$STATE/responses/002.json"
green_json   "$STATE/responses/003.json"
DSB_PR_CHECKS_INTERVAL_SEC=1 run_release "$VERSION" --no-publish --checks-timeout 30
POLLED="$(cat "$STATE/poll" 2>/dev/null || echo 0)"
if [[ "$RC" -eq 0 && "$POLLED" -ge 3 && "$(merges)" -ge 1 ]]; then
  ok "waited through ${POLLED} polls, then merged (exit 0)"
else
  bad "rc=$RC polls=$POLLED merges=$(merges)"; dump "$STATE/err"; dump "$STATE/out"
fi
if grep -q '== merged:' "$STATE/out"; then
  ok "the run reached the merge"
else
  bad "no '== merged:' line"; dump "$STATE/out"
fi
if git -C "$CASE/origin.git" tag -l "v$VERSION" | grep -q .; then
  ok "the tag reached the origin after the merge"
else
  bad "no v$VERSION tag on origin — the run did not reach the tag step"
fi

# ---------------------------------------------------------------------------
head_ "2. a failing check stops the release before the merge call"
make_case failing-check
fail_json "$STATE/responses/001.json"
run_release "$VERSION" --no-publish --checks-timeout 30
if [[ "$RC" -ne 0 && "$(merges)" -eq 0 ]]; then
  ok "exit $RC with no merge call"
else
  bad "rc=$RC merges=$(merges)"; dump "$STATE/err"
fi
if grep -q 'test' "$STATE/err" && grep -q 'changelog move' "$STATE/err" \
   && grep -q -- '--skip-bump --skip-pr' "$STATE/err"; then
  ok "the diagnosis names the failed checks and the resume command"
else
  bad "stderr does not name the failed checks / resume command"; dump "$STATE/err"
fi

# ---------------------------------------------------------------------------
head_ "3. a rejected merge exits 1 with the PR and the resume command"
make_case merge-rejected
green_json "$STATE/responses/001.json"
GH_MERGE_FAIL=1 run_release "$VERSION" --no-publish --checks-timeout 30
if [[ "$RC" -ne 0 && "$(merges)" -ge 1 ]]; then
  ok "exit $RC after the merge attempt (not a quiet exit 0)"
else
  bad "rc=$RC merges=$(merges)"; dump "$STATE/err"
fi
if grep -q 'gh pr merge failed' "$STATE/err" && grep -q -- '--skip-bump --skip-pr' "$STATE/err"; then
  ok "the failure names the open PR and the resume command"
else
  bad "stderr does not name the merge failure / resume command"; dump "$STATE/err"
fi

# ---------------------------------------------------------------------------
head_ "4. a conflicting PR stops before the merge call"
make_case conflicting
conflict_json "$STATE/responses/001.json"
run_release "$VERSION" --no-publish --checks-timeout 30
if [[ "$RC" -ne 0 && "$(merges)" -eq 0 ]] && grep -qi 'conflict' "$STATE/err"; then
  ok "exit $RC with no merge call, conflict named"
else
  bad "rc=$RC merges=$(merges)"; dump "$STATE/err"
fi

# ---------------------------------------------------------------------------
head_ "5. the wait is bounded by --checks-timeout"
make_case checks-timeout
pending_json "$STATE/responses/001.json"
DSB_PR_CHECKS_INTERVAL_SEC=1 run_release "$VERSION" --no-publish --checks-timeout 2
if [[ "$RC" -ne 0 && "$(merges)" -eq 0 ]] && grep -q 'within 2s' "$STATE/err"; then
  ok "exit $RC on the deadline with no merge call"
else
  bad "rc=$RC merges=$(merges)"; dump "$STATE/err"
fi

printf '\n%s passed, %s failed\n' "$PASS" "$FAIL"
[[ "$FAIL" -eq 0 ]] || exit 1
