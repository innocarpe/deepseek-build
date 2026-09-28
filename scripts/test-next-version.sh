#!/usr/bin/env bash
# Regression: the bump level is read from the merges on main, and release.sh
# refuses a version below that judgment without a named reason.
#
# Why this exists: all ten releases 6.1.1..6.1.10 shipped as PATCH. Run at each
# release's pre-release merge, scripts/next-version.sh proposes MINOR for
# 6.1.1-6.1.6 and 6.1.10 (feat/ merges on the distribution surface — e.g.
# #321 phone-band-composer) and PATCH for 6.1.7-6.1.9. Nothing chose the patch
# digit; there was no rule to choose with (docs/contributing/versioning.md §1c).
#
# Cases:
#   1. feat on the surface -> MINOR, with the per-merge lines naming the
#      surface merge and the harness one
#   2. fix/perf on the surface -> PATCH
#   3. harness only -> none, and --version prints nothing
#   4. a direct commit on the surface floors at PATCH; a harness one stays none
#   5. --check: below exits 1 naming the deciding merge; at/above exits 0;
#      not newer than the last tag exits 1
#   6. release.sh stops before the bump when the version is below the judgment
#   7. release.sh --level-override "<reason>" passes the gate and the reason
#      lands in the release PR body
#   8. --level-override with an empty reason is a usage error
#   9. no v* tag merged -> next-version.sh exits 2 and release.sh warns and
#      carries on (there is nothing to compare)
#
# Hermetic: throwaway repos with a bare origin and GitHub-shaped merge
# subjects, and a fake `gh` that replays fixture JSON. Nothing reaches the
# network, nothing is bumped, tagged or published. Local-only by contract; not
# wired into CI.
#
# Usage: ./scripts/test-next-version.sh
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
NV="$ROOT/scripts/next-version.sh"
RELEASE="$ROOT/scripts/release.sh"
PR_CHECKS="$ROOT/scripts/lib/pr_checks.py"
SYSTEM_BASH="/bin/bash"

PASS=0
FAIL=0
ok()  { printf '  ok   %s\n' "$*"; PASS=$((PASS + 1)); }
bad() { printf '  FAIL %s\n' "$*" >&2; FAIL=$((FAIL + 1)); }
head_() { printf '\n== %s ==\n' "$*"; }
dump() { sed -n '1,30p' "$1" >&2 2>/dev/null || true; }

[[ -f "$NV" ]] || { echo "error: $NV missing" >&2; exit 1; }
[[ -f "$RELEASE" ]] || { echo "error: $RELEASE missing" >&2; exit 1; }
[[ -f "$PR_CHECKS" ]] || { echo "error: $PR_CHECKS missing" >&2; exit 1; }
if [[ ! -x "$SYSTEM_BASH" ]]; then
  echo "skip: $SYSTEM_BASH not found; the release lane runs under the system bash" >&2
  exit 0
fi

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

# --- sandbox ------------------------------------------------------------------
# mk_repo <name>: throwaway repo with a bare origin, a seed commit, tag v1.2.3
# and a copy of next-version.sh (the tool reads its own checkout, like
# release.sh does). Sets CASE/SB/BIN/STATE.
mk_repo() {
  CASE="$TMP/$1"
  SB="$CASE/sb"
  BIN="$CASE/bin"
  STATE="$CASE/state"
  mkdir -p "$SB/scripts/lib" "$SB/npm/scripts" "$SB/docs/product/versions" \
           "$BIN" "$STATE/home"

  git init -q -b main "$SB"
  git -C "$SB" config user.name "Bump Level Test"
  git -C "$SB" config user.email "bump-level@example.invalid"
  git init -q --bare "$CASE/origin.git"
  git -C "$SB" remote add origin "$CASE/origin.git"

  cp "$NV" "$SB/scripts/next-version.sh"
  chmod +x "$SB/scripts/next-version.sh"
  printf '[workspace.package]\nversion = "1.2.3"\n' > "$SB/Cargo.toml"
  : > "$SB/package.json"; : > "$SB/Cargo.lock"; : > "$SB/CHANGELOG.md"
  printf '# Version history\n' > "$SB/docs/product/versions/README.md"
  printf 'seed\n' > "$SB/README.md"
  git -C "$SB" add -A
  git -C "$SB" commit -qm "seed"
}

tag_seed() { # annotate the seed with the baseline tag
  git -C "$SB" tag v1.2.3
}

push_main() { git -C "$SB" push -q origin main; }

# mk_merge <branch> <pr> <path...>: a merge whose subject is the shape GitHub
# writes. The branch carries one commit touching <path...>.
mk_merge() {
  local branch="$1" pr="$2" p
  shift 2
  git -C "$SB" checkout -q -b "$branch" main
  for p in "$@"; do
    mkdir -p "$SB/$(dirname "$p")"
    printf '%s\n' "$branch" >> "$SB/$p"
  done
  git -C "$SB" add -A
  git -C "$SB" commit -qm "$branch: change"
  git -C "$SB" checkout -q main
  git -C "$SB" merge -q --no-ff -m "Merge pull request #$pr from innocarpe/$branch" "$branch"
  git -C "$SB" branch -q -D "$branch"
}

mk_direct() { # <path> <subject>
  mkdir -p "$SB/$(dirname "$1")"
  printf 'direct\n' >> "$SB/$1"
  git -C "$SB" add -A
  git -C "$SB" commit -qm "$2"
}

run_nv() { # <args...>; sets RC, OUT, ERR
  RC=0
  OUT="$(cd "$SB" && "$SYSTEM_BASH" scripts/next-version.sh "$@" 2> "$STATE/nv.err")" || RC=$?
  ERR="$(cat "$STATE/nv.err" 2>/dev/null || true)"
  rm -f "$STATE/nv.err"
}

# --- release.sh sandbox -------------------------------------------------------
write_gh_stub() { # <path>
  cat > "$1" <<'STUB'
#!/usr/bin/env bash
# Fake `gh` for test-next-version.sh: records every call, replays green-check
# and released-asset fixtures, and copies the PR body aside for assertions.
set -u
printf '%s\n' "$*" >> "$GH_CALLS"

case "$1 $2" in
  "pr create")
    prev=""
    for a in "$@"; do
      if [[ "$prev" == "--body-file" ]]; then cp "$a" "$GH_STATE_DIR/pr-body.md"; fi
      prev="$a"
    done
    printf 'https://github.com/innocarpe/deepseek-build/pull/%s\n' "$GH_PR_NUMBER"
    ;;
  "pr view")
    case "$*" in
      *statusCheckRollup*)
        printf '%s\n' '{"state":"OPEN","mergeable":"MERGEABLE","statusCheckRollup":[{"__typename":"CheckRun","name":"required","status":"COMPLETED","conclusion":"SUCCESS","workflowName":"CI"}]}'
        ;;
      *)
        printf 'MERGED\n'
        ;;
    esac
    ;;
  "pr merge")
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

stage_release_stubs() {
  cp "$RELEASE" "$SB/scripts/release.sh"
  cp "$PR_CHECKS" "$SB/scripts/lib/pr_checks.py"
  printf '#!/usr/bin/env bash\nexit 0\n' > "$SB/scripts/lib/refuse-primary-checkout.sh"
  printf '#!/usr/bin/env bash\nexit 0\n' > "$SB/scripts/check-semver.sh"
  printf 'import sys\nsys.exit(0)\n'      > "$SB/scripts/lib/version_log.py"
  cat > "$SB/scripts/bump-version.sh" <<'STUB'
#!/usr/bin/env bash
# Recording stub: the real bump edits the tree. Writes argv (one per line) for
# the assertion and stays a no-op so the release run can reach its end.
printf '%s\n' "$@" > "$BUMP_ARGS_OUT"
exit 0
STUB
  chmod +x "$SB/scripts/lib/refuse-primary-checkout.sh" "$SB/scripts/check-semver.sh" \
           "$SB/scripts/bump-version.sh"
  printf '#!/usr/bin/env bash\nexit 0\n' > "$BIN/node"
  printf '#!/usr/bin/env bash\nexit 0\n' > "$BIN/npm"
  chmod +x "$BIN/node" "$BIN/npm"
  write_gh_stub "$BIN/gh"
}

run_release() { # <args...>; sets RC
  RC=0
  ( cd "$SB" && \
    PATH="$BIN:$PATH" HOME="$STATE/home" \
    GH_CALLS="$STATE/calls.log" GH_STATE_DIR="$STATE" \
    GH_PR_NUMBER=4242 GH_VERSION="${REQ_VERSION:-1.2.4}" \
    BUMP_ARGS_OUT="$STATE/bump.args" \
    DSB_PR_CHECKS_INTERVAL_SEC=1 \
    "$SYSTEM_BASH" scripts/release.sh "$@" ) \
    > "$STATE/out" 2> "$STATE/err" || RC=$?
}

bump_argv() { cat "$STATE/bump.args" 2>/dev/null || true; }

# ---------------------------------------------------------------------------
head_ "1. feat on the surface -> MINOR (harness merge does not raise it)"
mk_repo feat-surface
tag_seed
mk_merge feat/phone-band 101 third_party/grok-build/crates/pager/src/views/band.rs
mk_merge docs/readme 102 docs/notes.md
push_main
run_nv
if [[ "$RC" -eq 0 ]] && grep -q 'proposed: 1.3.0 (MINOR)' <<< "$OUT"; then
  ok "proposal is 1.3.0 (MINOR)"
else
  bad "rc=$RC out:"; printf '%s\n' "$OUT" >&2; printf '%s\n' "$ERR" >&2
fi
if grep -q '#101.*feat/phone-band.*surface (third_party/)' <<< "$OUT" \
   && grep -q '#102.*docs/readme.*harness' <<< "$OUT"; then
  ok "the per-merge lines name the surface merge and the harness one"
else
  bad "per-merge lines differ:"; printf '%s\n' "$OUT" >&2
fi
run_nv --level
if [[ "$OUT" == "minor" ]]; then ok "--level prints minor"; else bad "--level printed '$OUT'"; fi
run_nv --version
if [[ "$OUT" == "1.3.0" ]]; then ok "--version prints 1.3.0"; else bad "--version printed '$OUT'"; fi

# ---------------------------------------------------------------------------
head_ "2. fix and perf on the surface -> PATCH"
mk_repo fix-surface
tag_seed
mk_merge fix/clock 201 third_party/grok-build/crates/pager/src/views/clock.rs
mk_merge perf/record 202 crates/dsb-cli/src/main.rs
push_main
run_nv
if [[ "$RC" -eq 0 ]] && grep -q 'proposed: 1.2.4 (PATCH)' <<< "$OUT"; then
  ok "proposal is 1.2.4 (PATCH)"
else
  bad "rc=$RC out:"; printf '%s\n' "$OUT" >&2
fi

# ---------------------------------------------------------------------------
head_ "3. harness only -> none, and --version prints nothing"
mk_repo harness-only
tag_seed
mk_merge feat/harness 301 skills/session-unit/SKILL.md
mk_merge fix/docs 302 docs/contributing/branches.md
mk_merge chore/labels 303 .github/labels.json
mk_merge test/pin 304 scripts/test-something.sh
push_main
run_nv
if [[ "$RC" -eq 0 ]] && grep -q 'proposed: none' <<< "$OUT"; then
  ok "proposal is none"
else
  bad "rc=$RC out:"; printf '%s\n' "$OUT" >&2
fi
run_nv --level
if [[ "$OUT" == "none" ]]; then ok "--level prints none"; else bad "--level printed '$OUT'"; fi
run_nv --version
if [[ -z "$OUT" ]]; then ok "--version prints nothing"; else bad "--version printed '$OUT'"; fi

# ---------------------------------------------------------------------------
head_ "4. a direct commit on the surface floors at PATCH"
mk_repo direct-surface
tag_seed
mk_direct crates/dsb-cli/src/main.rs "direct fix on the shipped CLI"
push_main
run_nv
if [[ "$RC" -eq 0 ]] && grep -q 'proposed: 1.2.4 (PATCH)' <<< "$OUT" \
   && grep -q '(direct).*surface (crates/)' <<< "$OUT"; then
  ok "a direct surface commit floors at PATCH and is labelled (direct)"
else
  bad "rc=$RC out:"; printf '%s\n' "$OUT" >&2
fi

mk_repo direct-harness
tag_seed
mk_direct scripts/one-off.sh "direct harness touch"
push_main
run_nv
if [[ "$RC" -eq 0 ]] && grep -q 'proposed: none' <<< "$OUT"; then
  ok "a direct harness commit stays none"
else
  bad "rc=$RC out:"; printf '%s\n' "$OUT" >&2
fi

# ---------------------------------------------------------------------------
head_ "5. --check: below exits 1 naming the deciding merge; at/above exits 0"
mk_repo check-levels
tag_seed
mk_merge fix/band 401 third_party/grok-build/crates/pager/src/views/band.rs
mk_merge feat/welcome 402 third_party/grok-build/crates/pager/src/views/welcome.rs
push_main
run_nv --check 1.2.4
if [[ "$RC" -eq 1 ]] && grep -q 'below the proposed 1.3.0' <<< "$ERR" \
   && grep -q '#402.*feat/welcome' <<< "$ERR"; then
  ok "1.2.4 (PATCH) is refused below the proposed 1.3.0 (MINOR), deciding merge named"
else
  bad "rc=$RC err:"; printf '%s\n' "$ERR" >&2
fi
run_nv --check 1.3.0
if [[ "$RC" -eq 0 ]] && grep -q 'at or above the proposed 1.3.0' <<< "$OUT"; then
  ok "1.3.0 (MINOR) passes"
else
  bad "rc=$RC out:"; printf '%s\n' "$OUT" >&2
fi
run_nv --check 2.0.0
if [[ "$RC" -eq 0 ]]; then ok "2.0.0 (a declared major) passes"; else bad "rc=$RC err:"; printf '%s\n' "$ERR" >&2; fi
run_nv --check 1.2.3
if [[ "$RC" -eq 1 ]] && grep -q 'not newer than the last tag v1.2.3' <<< "$ERR"; then
  ok "1.2.3 (the last tag) is refused as not newer"
else
  bad "rc=$RC err:"; printf '%s\n' "$ERR" >&2
fi
run_nv --check 9.9.9
if [[ "$RC" -eq 0 ]]; then ok "a far major passes the level gate"; else bad "rc=$RC err:"; printf '%s\n' "$ERR" >&2; fi

head_ "5b. --check passes when the merges require no bump"
mk_repo check-none
tag_seed
mk_merge docs/note 501 docs/notes.md
push_main
run_nv --check 1.2.4
if [[ "$RC" -eq 0 ]] && grep -q 'no bump is required' <<< "$OUT"; then
  ok "1.2.4 passes against a none proposal"
else
  bad "rc=$RC out:"; printf '%s\n' "$OUT" >&2
fi

# ---------------------------------------------------------------------------
head_ "6. release.sh stops before the bump when the version is below the judgment"
mk_repo gate-stop
tag_seed
mk_merge feat/welcome 601 third_party/grok-build/crates/pager/src/views/welcome.rs
push_main
stage_release_stubs
REQ_VERSION=1.2.4 run_release 1.2.4 --no-publish
if [[ "$RC" -ne 0 ]]; then
  ok "exit $RC for a PATCH request against a MINOR judgment"
else
  bad "release.sh ran through: rc=$RC"; dump "$STATE/out"; dump "$STATE/err"
fi
if [[ ! -f "$STATE/bump.args" ]] && grep -q 'below the computed bump level' "$STATE/err"; then
  ok "no bump call, and the stop names the computed level"
else
  bad "bump called or message missing:"; dump "$STATE/err"
fi

# ---------------------------------------------------------------------------
head_ "7. --level-override <reason> passes the gate and the reason reaches the PR body"
mk_repo gate-override
tag_seed
mk_merge feat/welcome 701 third_party/grok-build/crates/pager/src/views/welcome.rs
push_main
stage_release_stubs
REQ_VERSION=1.2.4 run_release 1.2.4 --no-publish \
  --level-override "the vendored rerun change is build plumbing; no user-visible behavior"
if [[ "$RC" -eq 0 ]]; then
  ok "release run completed (exit 0) below the computed level"
else
  bad "rc=$RC"; dump "$STATE/out"; dump "$STATE/err"
fi
if [[ "$(bump_argv | head -1)" == "1.2.4" ]]; then
  ok "the bump ran with the requested version"
else
  bad "bump argv: $(bump_argv)"
fi
if [[ -f "$STATE/pr-body.md" ]] \
   && grep -q 'Bump-level override' "$STATE/pr-body.md" \
   && grep -q 'build plumbing; no user-visible behavior' "$STATE/pr-body.md"; then
  ok "the override reason is in the release PR body"
else
  bad "PR body missing the override reason:"; dump "$STATE/pr-body.md"
fi

# ---------------------------------------------------------------------------
head_ "8. --level-override with an empty reason is a usage error"
REQ_VERSION=1.2.4 run_release 1.2.4 --no-publish --level-override ""
if [[ "$RC" -ne 0 ]] && ! grep -q 'below the computed bump level' "$STATE/err" \
   && grep -qi 'non-empty reason' "$STATE/err"; then
  ok "exit $RC at the argument check, before the gate"
else
  bad "rc=$RC"; dump "$STATE/err"
fi

# ---------------------------------------------------------------------------
head_ "9. no v* tag merged: exit 2, and release.sh warns and carries on"
mk_repo no-tag
mk_merge feat/welcome 901 third_party/grok-build/crates/pager/src/views/welcome.rs
push_main
run_nv --level
if [[ "$RC" -eq 2 ]] && grep -q 'no v\* tag is merged' <<< "$ERR"; then
  ok "next-version.sh exits 2 with the reason"
else
  bad "rc=$RC err:"; printf '%s\n' "$ERR" >&2
fi
stage_release_stubs
REQ_VERSION=1.2.4 run_release 1.2.4 --no-publish
if [[ "$RC" -eq 0 ]] && grep -q 'bump level could not be computed' "$STATE/err" \
   && [[ "$(bump_argv | head -1)" == "1.2.4" ]]; then
  ok "release.sh warned and reached the bump (exit 0)"
else
  bad "rc=$RC bump=$(bump_argv)"; dump "$STATE/out"; dump "$STATE/err"
fi

printf '\n%s passed, %s failed\n' "$PASS" "$FAIL"
[[ "$FAIL" -eq 0 ]] || exit 1
