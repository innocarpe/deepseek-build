#!/usr/bin/env bash
# Decide the bump level (MINOR / PATCH / none) and the next version from what
# merged since the last release tag.
#
# The judgment (docs/contributing/versioning.md §1c) reads the git graph only —
# no `gh`, no network:
#   - the first-parent merges on the release ref since the newest `v*` tag
#     merged into it
#   - the branch type of each merge
#     (`Merge pull request #N from <owner>/<type>/<slug>` in its subject)
#   - the paths each merge changed (`git diff --name-only <merge>^1 <merge>`)
#
#   feat/ changing the distribution surface  -> MINOR  (X.(Y+1).0)
#   another type changing it                 -> PATCH  (X.Y.(Z+1))
#   repository harness only                  -> none   (no release is owed)
#
# The distribution surface is what a user installs and runs: `crates/`,
# `third_party/`, `npm/`, `package.json`, `Cargo.toml`, `Cargo.lock`.
# Everything else — `scripts/`, `skills/`, `docs/`, `.github/`, tests, root
# markdown — is repository harness. A merge with no readable branch (a hand
# merge) or a direct commit counts by its paths with the type unknown, so a
# surface change there floors at PATCH. MAJOR is a product-line decision
# (versioning.md §1b) and is never computed here.
#
# Usage:
#   ./scripts/next-version.sh [--ref REF]      report: one line per merge, then the proposal
#   ./scripts/next-version.sh --level          none | patch | minor
#   ./scripts/next-version.sh --version        e.g. 6.2.0; empty when the level is none
#   ./scripts/next-version.sh --check <ver>    exit 0 when <ver> is at or above the
#                                              judgment, 1 when it is below (or not newer
#                                              than the last tag), 2 when no judgment can
#                                              be made (no repository, no tag)
#
# `--ref` defaults to origin/main, then main, then HEAD. `release.sh` runs
# `--check` before its bump and stops on 1; exit 2 only warns there.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

REF=""
MODE="report"
CHECK_VER=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --ref)
      REF="${2:-}"
      [[ -n "$REF" ]] || { echo "error: --ref needs a ref" >&2; exit 2; }
      shift 2 ;;
    --level) MODE="level"; shift ;;
    --version) MODE="version"; shift ;;
    --check)
      CHECK_VER="${2:-}"
      [[ -n "$CHECK_VER" ]] || { echo "error: --check needs a version" >&2; exit 2; }
      MODE="check"; shift 2 ;;
    -h|--help) sed -n '1,42p' "$0"; exit 0 ;;
    -*) echo "unknown option: $1" >&2; exit 2 ;;
    *) echo "unexpected argument: $1" >&2; exit 2 ;;
  esac
done

rank() { # <level> -> 0..3
  case "$1" in
    patch) echo 1 ;;
    minor) echo 2 ;;
    major) echo 3 ;;
    *) echo 0 ;;
  esac
}

upper() { printf '%s' "$1" | tr '[:lower:]' '[:upper:]'; }

git rev-parse --git-dir >/dev/null 2>&1 || {
  echo "error: not a git repository: $ROOT" >&2
  exit 2
}

if [[ -z "$REF" ]]; then
  for candidate in origin/main main HEAD; do
    if git rev-parse --verify -q "${candidate}^{commit}" >/dev/null 2>&1; then
      REF="$candidate"
      break
    fi
  done
fi
[[ -n "$REF" ]] || { echo "error: no release ref found (tried origin/main, main, HEAD)" >&2; exit 2; }

LAST_TAG="$(git tag --merged "$REF" -l 'v[0-9]*' --sort=-v:refname | head -1 || true)"
if [[ -z "$LAST_TAG" ]]; then
  echo "warn: no v* tag is merged into $REF — the bump level cannot be computed" >&2
  exit 2
fi
TAG_VER="${LAST_TAG#v}"
if [[ ! "$TAG_VER" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "error: the newest tag on $REF is $LAST_TAG, which is not vMAJOR.MINOR.PATCH" >&2
  exit 2
fi
IFS=. read -r V_MAJ V_MIN V_PAT <<< "$TAG_VER"

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

git log --first-parent --merges --format='%H%x09%s' "$LAST_TAG..$REF" > "$TMP/merges" 2>/dev/null || :
git log --first-parent --no-merges --format='%H%x09%s' "$LAST_TAG..$REF" > "$TMP/directs" 2>/dev/null || :

# One line per judged commit: <level>\t<label>\t<subject/branch>\t<surface|harness>
: > "$TMP/lines"
OVERALL="none"
N_MERGES=0
N_DIRECTS=0

# The distribution-surface roots a changed path belongs to (empty = harness).
# Reads a newline-separated path list on stdin; prints unique roots, space-joined.
surface_roots() {
  local out="" p r
  while IFS= read -r p; do
    [[ -n "$p" ]] || continue
    r=""
    case "$p" in
      crates/*) r="crates/" ;;
      third_party/*) r="third_party/" ;;
      npm/*) r="npm/" ;;
      package.json) r="package.json" ;;
      Cargo.toml) r="Cargo.toml" ;;
      Cargo.lock) r="Cargo.lock" ;;
    esac
    [[ -n "$r" ]] || continue
    case " $out " in *" $r "*) ;; *) out="$out $r" ;; esac
  done
  printf '%s' "${out# }"
}

while IFS= read -r line; do
  [[ -n "$line" ]] || continue
  sha="${line%%$'\t'*}"
  subject="${line#*$'\t'}"
  N_MERGES=$((N_MERGES + 1))

  pr=""
  branch=""
  type=""
  if [[ "$subject" =~ ^Merge[[:space:]]pull[[:space:]]request[[:space:]]#([0-9]+)[[:space:]]from[[:space:]][^/]+/(.+)$ ]]; then
    pr="#${BASH_REMATCH[1]}"
    branch="${BASH_REMATCH[2]}"
    case "$branch" in */*) type="${branch%%/*}" ;; esac
  fi

  roots="$(git diff --name-only "$sha^1" "$sha" 2>/dev/null | surface_roots || true)"
  level="none"
  klass="harness"
  if [[ -n "$roots" ]]; then
    klass="surface ($roots)"
    if [[ "$type" == "feat" ]]; then level="minor"; else level="patch"; fi
  fi
  if [[ "$(rank "$level")" -gt "$(rank "$OVERALL")" ]]; then OVERALL="$level"; fi
  printf '%s\t%s\t%s\t%s\n' \
    "$level" "${pr:--}" "${branch:-(unparsed)}" "$klass" >> "$TMP/lines"
done < "$TMP/merges"

while IFS= read -r line; do
  [[ -n "$line" ]] || continue
  sha="${line%%$'\t'*}"
  subject="${line#*$'\t'}"
  N_DIRECTS=$((N_DIRECTS + 1))

  roots="$(git diff-tree --no-commit-id --name-only -r "$sha" 2>/dev/null | surface_roots || true)"
  level="none"
  klass="harness"
  if [[ -n "$roots" ]]; then
    klass="surface ($roots)"
    level="patch"
  fi
  if [[ "$(rank "$level")" -gt "$(rank "$OVERALL")" ]]; then OVERALL="$level"; fi
  printf '%s\t%s\t%s\t%s\n' \
    "$level" "(direct)" "$(printf '%s' "$subject" | cut -c1-46)" "$klass" >> "$TMP/lines"
done < "$TMP/directs"

PROPOSED=""
case "$OVERALL" in
  minor) PROPOSED="$V_MAJ.$((10#$V_MIN + 1)).0" ;;
  patch) PROPOSED="$V_MAJ.$V_MIN.$((10#$V_PAT + 1))" ;;
esac

report() {
  printf 'ref: %s (%s)\n' "$REF" "$(git rev-parse --short "$REF")"
  printf 'last tag: %s\n' "$LAST_TAG"
  printf 'since %s: %s merges, %s direct commits\n' "$LAST_TAG" "$N_MERGES" "$N_DIRECTS"
  awk -F'\t' '{ printf "  %-7s %-46s %s\n", $2, $3, $4 }' "$TMP/lines"
  if [[ -n "$PROPOSED" ]]; then
    printf 'proposed: %s (%s)\n' "$PROPOSED" "$(upper "$OVERALL")"
  else
    printf 'proposed: none (no distribution-surface change since %s)\n' "$LAST_TAG"
  fi
}

case "$MODE" in
  level)
    printf '%s\n' "$OVERALL"
    ;;
  version)
    if [[ -n "$PROPOSED" ]]; then printf '%s\n' "$PROPOSED"; fi
    ;;
  check)
    if [[ ! "$CHECK_VER" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
      echo "error: '$CHECK_VER' is not MAJOR.MINOR.PATCH" >&2
      exit 2
    fi
    report
    printf '\n'
    IFS=. read -r R_MAJ R_MIN R_PAT <<< "$CHECK_VER"
    if (( 10#$R_MAJ > 10#$V_MAJ )); then
      R_LEVEL="major"
    elif (( 10#$R_MAJ < 10#$V_MAJ )); then
      R_LEVEL="notnewer"
    elif (( 10#$R_MIN > 10#$V_MIN )); then
      R_LEVEL="minor"
    elif (( 10#$R_MIN < 10#$V_MIN )); then
      R_LEVEL="notnewer"
    elif (( 10#$R_PAT > 10#$V_PAT )); then
      R_LEVEL="patch"
    else
      R_LEVEL="notnewer"
    fi
    if [[ "$R_LEVEL" == "notnewer" ]]; then
      echo "error: $CHECK_VER is not newer than the last tag $LAST_TAG" >&2
      exit 1
    fi
    if [[ "$(rank "$R_LEVEL")" -lt "$(rank "$OVERALL")" ]]; then
      echo "error: $CHECK_VER ($(upper "$R_LEVEL")) is below the proposed $PROPOSED ($(upper "$OVERALL"))" >&2
      echo "the deciding merges:" >&2
      awk -F'\t' -v lvl="$OVERALL" '$1 == lvl { printf "  %-7s %-46s %s\n", $2, $3, $4 }' "$TMP/lines" >&2
      exit 1
    fi
    if [[ "$OVERALL" == "none" ]]; then
      printf 'ok: %s (%s) — no bump is required by the merges since %s\n' \
        "$CHECK_VER" "$(upper "$R_LEVEL")" "$LAST_TAG"
    else
      printf 'ok: %s (%s) is at or above the proposed %s (%s)\n' \
        "$CHECK_VER" "$(upper "$R_LEVEL")" "$PROPOSED" "$(upper "$OVERALL")"
    fi
    ;;
  *)
    report
    ;;
esac
