#!/usr/bin/env bash
# Report where the vendored Grok Build pin stands against upstream.
#
# Answers, in one command, the questions a sync session would otherwise spend
# an hour re-deriving (skills/grok-sync/SKILL.md §0):
#
#   * What upstream commit is this tree actually pinned to, and which release
#     is that on disk?
#   * What is upstream HEAD today, and how far ahead is it?
#   * How large is the gap (releases, commits, files, lines)?
#   * What did those releases change, clustered for triage?
#
# Read-only with respect to the product repo: the upstream clone lives under
# the repo's sibling scratch directory (../_upstream by default) and the vendor
# tree is never modified.
#
# Usage:
#   ./scripts/grok-sync-inventory.sh                 # summary + cluster counts
#   ./scripts/grok-sync-inventory.sh --bullets       # also print every bullet
#   ./scripts/grok-sync-inventory.sh --bullets --version 1.0.9
#   ./scripts/grok-sync-inventory.sh --json          # machine-readable summary
#
# Env:
#   UPSTREAM_DIR   clone location   (default: first existing candidate, else
#                                   <repo>/../_upstream/grok-build)
#   UPSTREAM_URL   clone URL        (default: https://github.com/xai-org/grok-build)
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
VENDOR="${ROOT}/third_party/grok-build"
UPSTREAM_URL="${UPSTREAM_URL:-https://github.com/xai-org/grok-build}"

# Reuse an existing clone when there is one: a sync session usually already has
# it, and cloning this repo is not cheap.
if [[ -z "${UPSTREAM_DIR:-}" ]]; then
  for cand in \
    "$(cd "${ROOT}/.." && pwd)/_upstream/grok-build" \
    "${HOME}/Personal/Projects/OpenSources/_upstream/grok-build" \
    "${HOME}/_upstream/grok-build"
  do
    if [[ -d "${cand}/.git" ]]; then UPSTREAM_DIR="${cand}"; break; fi
  done
  UPSTREAM_DIR="${UPSTREAM_DIR:-$(cd "${ROOT}/.." && pwd)/_upstream/grok-build}"
fi

WANT_BULLETS=0
WANT_JSON=0
ONE_VERSION=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --bullets) WANT_BULLETS=1 ;;
    --json) WANT_JSON=1 ;;
    --version) ONE_VERSION="${2:-}"; shift ;;
    -h|--help) sed -n '2,26p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) echo "grok-sync-inventory: unknown argument: $1" >&2; exit 2 ;;
  esac
  shift
done

fail() { echo "grok-sync-inventory: $*" >&2; exit 1; }

[[ -f "${VENDOR}/SOURCE_REV" ]] || fail "missing ${VENDOR}/SOURCE_REV (see docs/architecture/GROK_VENDOR.md)"
PIN="$(tr -d '[:space:]' <"${VENDOR}/SOURCE_REV")"

# ---------------------------------------------------------------- upstream clone
if [[ ! -d "${UPSTREAM_DIR}/.git" ]]; then
  echo "grok-sync-inventory: cloning upstream into ${UPSTREAM_DIR}" >&2
  mkdir -p "$(dirname "${UPSTREAM_DIR}")"
  git clone --quiet "${UPSTREAM_URL}" "${UPSTREAM_DIR}" \
    || fail "clone failed; set UPSTREAM_DIR / UPSTREAM_URL or clone by hand"
else
  git -C "${UPSTREAM_DIR}" fetch --quiet origin
fi

up() { git -C "${UPSTREAM_DIR}" "$@"; }

HEAD_SHA="$(up rev-parse --short=10 HEAD)"
HEAD_FULL="$(up rev-parse HEAD)"
UP_REV="$(up show HEAD:SOURCE_REV 2>/dev/null | tr -d '[:space:]' || echo '')"

# Release a commit is on = newest changelog present in its tree.
release_at() {
  git -C "${UPSTREAM_DIR}" ls-tree -r --name-only "$1" \
      crates/codegen/xai-grok-shell/changelogs/ 2>/dev/null \
    | sed -n 's|.*/\([0-9][^/]*\)\.md$|\1|p' \
    | awk -F. '{ printf "%05d %05d %05d %s\n", $1, $2, $3, $0 }' \
    | sort | tail -1 | awk '{print $4}'
}

# Which upstream commit carries the pinned SOURCE_REV? `SOURCE_REV` in the
# vendored tree is a *monorepo* SHA, not a commit of this open-sync repo, so the
# match is: the upstream commit whose own SOURCE_REV file equals the pin.
# Newest first — a sync commit is what we want, not its ancestor.
PIN_COMMIT=""
if [[ -n "${UP_REV}" && "${UP_REV}" == "${PIN}" ]]; then
  PIN_COMMIT="${HEAD_FULL}"
else
  while read -r c; do
    [[ -z "$c" ]] && continue
    if [[ "$(up show "${c}:SOURCE_REV" 2>/dev/null | tr -d '[:space:]')" == "${PIN}" ]]; then
      PIN_COMMIT="$c"; break
    fi
  done < <(up log --format=%H -- SOURCE_REV 2>/dev/null | head -500)
fi

HEAD_REL="$(release_at "${HEAD_FULL}")"
PIN_REL="$(release_at "${PIN_COMMIT}")"

# ------------------------------------------------------------------- gap sizing
gap_commits=0 gap_files=0 gap_add=0 gap_del=0 gap_from="" gap_to="" pin_ver="(unknown)" head_ver="(unknown)"
if [[ -n "${PIN_COMMIT}" ]]; then
  gap_commits="$(up rev-list --count "${PIN_COMMIT}..HEAD" 2>/dev/null || echo 0)"
  read -r gap_files gap_add gap_del <<<"$(up diff --numstat "${PIN_COMMIT}..HEAD" 2>/dev/null \
    | awk '{f++; a+=$1; d+=$2} END {printf "%d %d %d", f, a, d}')"
  gap_from="$(up log -1 --format=%ad --date=short "${PIN_COMMIT}" 2>/dev/null)"
  gap_to="$(up log -1 --format=%ad --date=short HEAD 2>/dev/null)"
  pin_ver="$(up show "${PIN_COMMIT}:crates/codegen/xai-grok-version/Cargo.toml" 2>/dev/null \
    | sed -n 's/^version = "\(.*\)"/\1/p' | head -1)"
  head_ver="$(up show "HEAD:crates/codegen/xai-grok-version/Cargo.toml" 2>/dev/null \
    | sed -n 's/^version = "\(.*\)"/\1/p' | head -1)"
fi

# ------------------------------------------------------- changelog range on disk
range_versions() {
  local from="$1"
  up ls-tree -r --name-only HEAD crates/codegen/xai-grok-shell/changelogs/ \
    | sed -n 's|.*/\([0-9][^/]*\)\.md$|\1|p' \
    | awk -v from="$from" -F. '
        NF==3 { key=$1*1000000+$2*1000+$3
                fk=split(from, a, "."); if (from!="") fkey=a[1]*1000000+a[2]*1000+(a[3]+0); else fkey=-1
                if (key > fkey) print key, $0 }
      ' \
    | sort -n | awk '{print $2}'
}

if [[ -n "${ONE_VERSION}" ]]; then
  RANGE_VERSIONS="${ONE_VERSION}"
else
  RANGE_VERSIONS="$(range_versions "${pin_ver}")"
fi
RANGE_COUNT="$(printf '%s\n' ${RANGE_VERSIONS} | grep -c . || true)"

CHANGELOG_DIR="crates/codegen/xai-grok-shell/changelogs"

# Emit "<section>\t<bullet>" for a release. The section matters: "Breaking
# Changes" and "Performance" carry meaning a keyword guess cannot recover.
bullets_for() {
  local v="$1"
  up show "HEAD:${CHANGELOG_DIR}/${v}.md" 2>/dev/null | awk '
    /^## /  { s = $0; sub(/^##[ \t]*/, "", s); sub(/[ \t]*$/, "", s); next }
    /^- /   { print s "\t" substr($0, 3) }
  '
}

# Cluster a bullet by the first matching keyword rule. Order matters: the first
# match wins, so put the more specific rules before the broader ones. The
# section wins over the keyword guess when it is one of the special ones.
cluster_of() {
  local section="$1" b="$2" s
  case "$section" in
    "Breaking Changes") echo "breaking"; return ;;
    "Performance")      echo "performance"; return ;;
  esac
  s="$(printf '%s' "$b" | tr '[:upper:]' '[:lower:]')"
  case "$s" in
    *telemetry*|*"consent notice"*|*"privacy banner"*|*"grok trace"*) echo "xai-only" ;;
    *billing*|*credit*|*"upsell"*|*"team_id"*|*oidc*|*"self-update"*|*"auto-update"*|*"grok_channel"*|*voice*|*dictation*|*"video generation"*|*zdr*|*"computer hub"*|*"desktop app"*|*workspaced*|*"x.ai/"*) echo "xai-only" ;;
    *worktree*|*"git status"*|*"git histor"*|*"grok clone"*|*reflog*|*submodule*) echo "worktree-git" ;;
    *subagent*|*"workflow-spawned"*) echo "subagents" ;;
    *mcp*) echo "mcp" ;;
    *skill*|*plugin*|*"extension"*|*workflow*|*memory*) echo "skills-plugins" ;;
    *permission*|*sandbox*|*hook*|*"deny"*|*allow*) echo "permissions" ;;
    *model*|*effort*|*reasoning*|*sampling*|*retry*|*"rate limit"*|*stream*) echo "model-plumbing" ;;
    *config*|*settings*|*"requirements.toml"*|*policy*) echo "config" ;;
    *session*|*resume*|*compact*|*rewind*|*headless*|*recap*) echo "sessions" ;;
    *tool*|*"read_file"*|*"edit_file"*|*bash*|*"terminal"*|*image*|*pdf*|*notebook*) echo "tools" ;;
    *) echo "tui-ux" ;;
  esac
}

if [[ "${WANT_JSON}" == "1" ]]; then
  python3 - "$PIN" "$PIN_REL" "$HEAD_SHA" "$HEAD_REL" "$gap_commits" "$gap_files" "$gap_add" "$gap_del" "$gap_from" "$gap_to" "$pin_ver" "$head_ver" "$RANGE_COUNT" <<'PY'
import json, sys
(pin, pin_rel, head, head_rel, commits, files, add, dele, frm, to, pinv, headv, rng) = sys.argv[1:14]
print(json.dumps({
    "pin": pin,
    "pin_release": pin_rel,
    "pin_version": pinv,
    "upstream_head": head,
    "upstream_release": head_rel,
    "upstream_version": headv,
    "gap_commits": int(commits or 0),
    "gap_files": int(files or 0),
    "gap_additions": int(add or 0),
    "gap_deletions": int(dele or 0),
    "gap_from_date": frm,
    "gap_to_date": to,
    "releases_behind": int(rng or 0),
}, indent=2))
PY
  exit 0
fi

# ------------------------------------------------------------------- human output
cat <<EOF
grok-build vendored pin vs upstream
-----------------------------------
pin (SOURCE_REV)      ${PIN:0:12}   release on disk: ${PIN_REL}
pin resolves to       ${PIN_COMMIT:0:10}   version ${pin_ver}   ${gap_from}
upstream HEAD         ${HEAD_SHA}   version ${head_ver}   ${gap_to}
releases behind       ${RANGE_COUNT}   (${PIN_REL} .. ${HEAD_REL})
commits behind        ${gap_commits}
files changed         ${gap_files}   (+${gap_add} / -${gap_del})
EOF

if [[ -z "${PIN_COMMIT}" ]]; then
  echo
  echo "note: the pinned SHA was not found in the upstream clone; the pin may"
  echo "      come from a rewritten history or a private sync. Clone history is"
  echo "      ${UPSTREAM_DIR}."
fi

if [[ "${RANGE_COUNT}" == "0" ]]; then
  echo
  echo "up to date — nothing to sync."
  exit 0
fi

echo
echo "changelog clusters (covered range)"
echo "----------------------------------"
tmp="$(mktemp)"; trap 'rm -f "$tmp"' EXIT
for v in ${RANGE_VERSIONS}; do
  while IFS=$'\t' read -r section b; do
    [[ -z "$b" ]] && continue
    printf '%s\t%s\n' "$(cluster_of "$section" "$b")" "[$v] $b" >>"$tmp"
  done < <(bullets_for "$v")
done

awk -F'\t' '{c[$1]++} END {for (k in c) printf "%6d  %s\n", c[k], k}' "$tmp" | sort -rn
printf '%6d  %s\n' "$(wc -l <"$tmp" | tr -d ' ')" "TOTAL"
echo
echo "xai-only (N/A candidates): $(awk -F'\t' '$1=="xai-only"' "$tmp" | wc -l | tr -d ' ')"
echo "breaking changes:          $(awk -F'\t' '$1=="breaking"' "$tmp" | wc -l | tr -d ' ')"

if [[ "${WANT_BULLETS}" == "1" ]]; then
  echo
  echo "every bullet"
  echo "------------"
  sort -t$'\t' -k1,1 "$tmp"
fi

echo
echo "next: skills/grok-sync/SKILL.md — classify every bullet, then merge (§3)."
