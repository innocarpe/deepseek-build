#!/usr/bin/env bash
# Regression test for the CHANGELOG release move — scripts/lib/changelog_release.py,
# the code `bump-version.sh` runs when it opens a version section.
#
# Hermetic: every case runs against a fixture CHANGELOG in a temp dir. It never
# touches this repo's CHANGELOG.md (the one dry-run case below only reads it),
# publishes nothing, and needs no credentials.
#
# The defect this pins: `bump-version.sh` inserted the new section and seeded it
# with the `--desc` line while `## Unreleased` kept its items. Measured on
# `v5.7.0`, seven items that the release shipped were still filed as unreleased,
# and `## 5.7.0` held only `- phone-width layout and Orca pane status` — the
# release record did not say what the release did.
#
# Paths covered:
#   1. items move  — Unreleased items land in the new section byte for byte
#   2. --desc      — never printed above items it does not describe
#   3. empty       — the previous behavior: one `- <note>` line
#   4. no items    — a bare Unreleased falls back to the placeholder
#   5. drifted     — a moved-down Unreleased is pulled back to the top
#   6. invariant   — version sections stay newest-first; a broken order fails
#   7. residue     — non-item text stays in Unreleased (and plan warns)
#   8. dry-run     — bump-version.sh --dry-run predicts the same move
#   9. wiring      — bump-version.sh runs the mover (no inline re-insert)
#
# Usage: ./scripts/test-changelog-release.sh
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

MOVER="$ROOT/scripts/lib/changelog_release.py"
BUMP="$ROOT/scripts/bump-version.sh"

PASS=0
FAIL=0
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

ok()  { printf '  ok   %s\n' "$*"; PASS=$((PASS + 1)); }
bad() { printf '  FAIL %s\n' "$*" >&2; FAIL=$((FAIL + 1)); }
head_() { printf '\n== %s ==\n' "$*"; }

for c in python3 git; do
  command -v "$c" >/dev/null 2>&1 || { echo "error: $c not found on PATH" >&2; exit 1; }
done
[[ -f "$MOVER" ]] || { echo "error: $MOVER missing" >&2; exit 1; }

# Extract the text of one section (heading line through the blank line before
# the next heading). Exact, so a reflowed or re-spaced heading is a failure.
section_text() { # <file> <version-or-Unreleased>
  python3 - "$1" "$2" <<'PY'
import re, sys
text = open(sys.argv[1]).read()
m = re.search(r'(?m)^## ' + re.escape(sys.argv[2]) + r'[^\n]*\n', text)
if not m:
    sys.exit(2)
nxt = re.search(r'(?m)^## ', text[m.end():])
stop = m.end() + (nxt.start() if nxt else len(text) - m.end())
sys.stdout.write(text[m.start():stop])
PY
}

assert_section() { # <file> <version> <expected-file> <label>
  if section_text "$1" "$2" > "$TMP/got.section" 2>/dev/null \
     && cmp -s "$TMP/got.section" "$3"; then
    ok "$4"
  else
    bad "$4"
    echo '--- got ---' >&2; cat "$TMP/got.section" >&2
    echo '--- want ---' >&2; cat "$3" >&2
  fi
}

DASH="$(python3 -c 'print("\u2014", end="")')"  # em dash; bash 3.2 has no $'\u…'

# ---------------------------------------------------------------------------
head_ "1. items move into the new section, byte for byte"
cat > "$TMP/CHANGELOG.md" <<EOF
# Changelog

## Unreleased

- First item, one line.
- Second item with a wrapped line
  and its continuation.
- Third item.

## 5.6.0 ${DASH} 2026-09-25

- A previously released thing.

## 5.5.0 ${DASH} 2026-08-08

- An older thing.
EOF
cat > "$TMP/exp-6.0.0.md" <<EOF
## 6.0.0 ${DASH} 2026-09-26

- First item, one line.
- Second item with a wrapped line
  and its continuation.
- Third item.

EOF

python3 "$MOVER" apply "$TMP/CHANGELOG.md" 6.0.0 2026-09-26 \
  'a summary nobody should see' > "$TMP/apply.log"
grep -q 'moved 3 Unreleased item(s)' "$TMP/apply.log" \
  && ok "apply reports 3 moved items" \
  || bad "apply did not report the move: $(cat "$TMP/apply.log")"

if grep -q 'a summary nobody should see' "$TMP/CHANGELOG.md"; then
  bad "--desc leaked into the file although items existed"
else
  ok "--desc stayed out of the release section"
fi

assert_section "$TMP/CHANGELOG.md" 6.0.0 "$TMP/exp-6.0.0.md" \
  "the release section is heading + the three items verbatim"

if [[ "$(section_text "$TMP/CHANGELOG.md" Unreleased)" == '## Unreleased' ]]; then
  ok "## Unreleased is still the first section and is empty"
else
  bad "## Unreleased is not an empty top section: $(section_text "$TMP/CHANGELOG.md" Unreleased)"
fi

# Nothing was lost or duplicated: the items exist exactly once.
COUNT_A="$(grep -c 'First item, one line\.' "$TMP/CHANGELOG.md")"
COUNT_C="$(grep -c 'Third item\.' "$TMP/CHANGELOG.md")"
[[ "$COUNT_A" == 1 && "$COUNT_C" == 1 ]] \
  && ok "items are not duplicated (A=$COUNT_A C=$COUNT_C)" \
  || bad "item counts wrong: A=$COUNT_A C=$COUNT_C"

for line in '- A previously released thing.' "## 5.5.0 ${DASH} 2026-08-08" '- An older thing.'; do
  grep -qF -- "$line" "$TMP/CHANGELOG.md" \
    && ok "older content preserved: $line" \
    || bad "older content lost: $line"
done

# ---------------------------------------------------------------------------
head_ "2. newest-first invariant"
if python3 - "$TMP/CHANGELOG.md" <<'PY'
import re, sys
sys.path.insert(0, 'scripts/lib')
from changelog_release import vkey
text = open(sys.argv[1]).read()
order = [h.strip() for h in re.findall(r'(?m)^## ([^\n]+)', text)]
assert order[0] == 'Unreleased', order
vers = re.findall(r'(?m)^## ([0-9]+\.[0-9]+\.[0-9]+)(?:-[0-9A-Za-z.\-]+)?[ \t]', text)
assert [vkey(v) for v in vers] == sorted((vkey(v) for v in vers), reverse=True), vers
PY
then ok "Unreleased is pinned at the top and versions are newest-first"
else bad "order invariant broken after the move"; fi

cat > "$TMP/broken.md" <<EOF
# Changelog

## Unreleased

- Something.

## 5.6.0 ${DASH} 2026-09-25

- Newer.

## 7.0.0 ${DASH} 2026-09-25

- Newest, but out of order.
EOF
if python3 "$MOVER" apply "$TMP/broken.md" 8.0.0 2026-09-26 > /dev/null 2>&1; then
  bad "a not-newest-first CHANGELOG was accepted"
else
  ok "a not-newest-first CHANGELOG is rejected (exit non-zero)"
fi

# ---------------------------------------------------------------------------
head_ "3. empty Unreleased keeps the old behavior (--desc as the seed)"
cat > "$TMP/empty.md" <<EOF
# Changelog

## Unreleased


## 5.6.0 ${DASH} 2026-09-25

- A previously released thing.
EOF
cat > "$TMP/exp-seed.md" <<EOF
## 6.0.0 ${DASH} 2026-09-26

- one-line release note

EOF
python3 "$MOVER" apply "$TMP/empty.md" 6.0.0 2026-09-26 'one-line release note' \
  > "$TMP/apply2.log"
grep -q 'seeded "## 6.0.0" with the --desc note' "$TMP/apply2.log" \
  && ok "apply reports the --desc seed" \
  || bad "apply did not report the seed: $(cat "$TMP/apply2.log")"
assert_section "$TMP/empty.md" 6.0.0 "$TMP/exp-seed.md" \
  "the --desc line is the seeded section"

head_ "4. a bare Unreleased falls back to the placeholder"
cat > "$TMP/bare.md" <<EOF
# Changelog

## Unreleased

## 5.6.0 ${DASH} 2026-09-25

- A previously released thing.
EOF
cat > "$TMP/exp-placeholder.md" <<EOF
## 6.0.0 ${DASH} 2026-09-26

- _release notes: fill in before merge_

EOF
python3 "$MOVER" apply "$TMP/bare.md" 6.0.0 2026-09-26 > /dev/null
assert_section "$TMP/bare.md" 6.0.0 "$TMP/exp-placeholder.md" \
  "the placeholder is seeded when there is no --desc"

# ---------------------------------------------------------------------------
head_ "5. a drifted Unreleased is pulled back to the top"
cat > "$TMP/drifted.md" <<EOF
# Changelog

## 5.6.0 ${DASH} 2026-09-25

- A previously released thing.

## Unreleased

- First item, one line.
EOF
cat > "$TMP/exp-drift.md" <<EOF
## 6.0.0 ${DASH} 2026-09-26

- First item, one line.

EOF
python3 "$MOVER" apply "$TMP/drifted.md" 6.0.0 2026-09-26 > /dev/null
[[ "$(section_text "$TMP/drifted.md" Unreleased)" == '## Unreleased' ]] \
  && ok "drifted Unreleased returned to the top, empty" \
  || bad "drifted Unreleased stayed down: $(sed -n '1,9p' "$TMP/drifted.md")"
assert_section "$TMP/drifted.md" 6.0.0 "$TMP/exp-drift.md" \
  "its item moved with it"

# ---------------------------------------------------------------------------
head_ "6. non-item text in Unreleased stays behind (and plan warns)"
cat > "$TMP/note.md" <<EOF
# Changelog

## Unreleased

A holder note that is not a release item.

- A real item.

## 5.6.0 ${DASH} 2026-09-25

- Older.
EOF
python3 "$MOVER" apply "$TMP/note.md" 6.0.0 2026-09-26 > /dev/null
grep -q 'A holder note that is not a release item.' "$TMP/note.md" \
  && ok "the non-item note was not swept into the release" \
  || bad "the non-item note disappeared"
grep -qF -- '- A real item.' "$TMP/note.md" \
  && ok "the real item still moved" \
  || bad "the real item did not move"
if python3 "$MOVER" plan "$TMP/note.md" 7.0.0 2026-09-27 2>/dev/null | grep -q 'warning'; then
  ok "plan warns that non-item text stays in Unreleased"
else
  bad "plan did not warn about the non-item text left behind"
fi

# ---------------------------------------------------------------------------
head_ "7. bump-version.sh --dry-run predicts the same move"
# The target must differ from the version on disk (the script refuses to bump
# to the version it is already at), and the repo moves, so derive it here.
CUR_VER="$(python3 -c 'import re
s=open("Cargo.toml").read(); i=s.index("[workspace.package]")
print(re.search(r"(?m)^version = \"([^\"]+)\"", s[i:]).group(1))')"
TARGET="$(python3 -c 'import sys; v=sys.argv[1].split("."); v[2]=str(int(v[2])+1); print(".".join(v))' "$CUR_VER")"
REAL_COUNT="$(python3 -c 'import re,sys
text=open(sys.argv[1]).read()
m=re.search(r"(?m)^## Unreleased[ \t]*\n", text)
nxt=re.search(r"(?m)^## ", text[m.end():])
body=text[m.end():m.end()+nxt.start()]
print(sum(1 for line in body.splitlines() if line.startswith("- ")))' "$ROOT/CHANGELOG.md")"

DRY="$(./scripts/bump-version.sh "$TARGET" --desc "dry-run desc" --dry-run 2>&1)"
if [[ "$REAL_COUNT" -gt 0 ]]; then
  printf '%s\n' "$DRY" | grep -q "item(s) would move into \"## $TARGET" \
    && ok "--dry-run reports the move into $TARGET" \
    || bad "--dry-run did not report the move: $DRY"
  printf '%s\n' "$DRY" | grep -q -e '--desc will not seed that section' \
    && ok "--dry-run says --desc will not seed the section while items exist" \
    || bad "--dry-run did not flag --desc as unseeded: $DRY"
  REPORTED_COUNT="$(printf '%s\n' "$DRY" | sed -n 's/^changelog: \([0-9][0-9]*\) Unreleased item(s).*/\1/p')"
  [[ -n "$REPORTED_COUNT" && "$REPORTED_COUNT" == "$REAL_COUNT" ]] \
    && ok "--dry-run count ($REPORTED_COUNT) matches the $REAL_COUNT Unreleased items in CHANGELOG.md" \
    || bad "--dry-run count '$REPORTED_COUNT' != actual '$REAL_COUNT'"
else
  printf '%s\n' "$DRY" | grep -q 'Unreleased is empty; --desc would seed the new section' \
    && ok "--dry-run says --desc would seed the section (Unreleased is empty)" \
    || bad "--dry-run wrong for an empty Unreleased: $DRY"
fi

# Hash before/after: a `git diff` check would be fooled by whatever else the
# branch is already carrying.
BEFORE_HASH="$(python3 -c 'import hashlib,sys;print(hashlib.sha256(open(sys.argv[1],"rb").read()).hexdigest())' "$ROOT/CHANGELOG.md")"
./scripts/bump-version.sh "$TARGET" --desc "dry-run desc" --dry-run >/dev/null 2>&1
AFTER_HASH="$(python3 -c 'import hashlib,sys;print(hashlib.sha256(open(sys.argv[1],"rb").read()).hexdigest())' "$ROOT/CHANGELOG.md")"
[[ "$BEFORE_HASH" == "$AFTER_HASH" ]] \
  && ok "--dry-run left CHANGELOG.md byte-identical" \
  || bad "--dry-run rewrote CHANGELOG.md"

# ---------------------------------------------------------------------------
head_ "8. the junction after Unreleased keeps its blank line"
# A glued junction (`## Unreleased` immediately followed by `## <version>`) is
# how two shipped items got mis-filed: a branch that appends an item under
# Unreleased merges into a glued tree *without a conflict*, and the item lands
# under the version heading. Measured: #209 (cache attribution) landed under
# 5.7.0, #206 (SSH paste) under 6.0.0 — each byte-identical to `git merge-tree`.
cat > "$TMP/glued.md" <<EOF
# Changelog

## Unreleased
## 5.7.0 ${DASH} 2026-09-25

- An older released thing.
EOF
python3 "$MOVER" apply "$TMP/glued.md" 6.0.0 2026-09-26 'note' > /dev/null
if [[ "$(sed -n '3,5p' "$TMP/glued.md" | tr '\n' '|')" == "## Unreleased||## 6.0.0 "* ]]; then
  ok "a glued junction is repaired on the next bump"
else
  bad "the glued junction survived the bump: $(sed -n '1,6p' "$TMP/glued.md" | tr '\n' '|')"
fi

# The guard is not only a fix-up: a real two-branch merge must CONFLICT (so a
# human files the item) instead of silently filing it under a version.
MERGE_TMP="$(mktemp -d)"
(
  cd "$MERGE_TMP" || exit 1
  git init -q; git config user.email t@t; git config user.name t
  cat > CHANGELOG.md <<EOF
# Changelog

## Unreleased

- The TUI fits a phone-width pane.

## 5.6.0 ${DASH} 2026-09-25

- An older thing.
EOF
  git add -A; git commit -qm base
  BASE_BRANCH="$(git symbolic-ref --short HEAD)"
  git checkout -qb mainside
  python3 "$MOVER" apply CHANGELOG.md 6.0.0 2026-09-26 'release note' > /dev/null
  git add -A; git commit -qm 'release opens 6.0.0'
  git checkout -q "$BASE_BRANCH"; git checkout -qb feature
  python3 - <<'PY'
import pathlib
p = pathlib.Path('CHANGELOG.md'); s = p.read_text()
p.write_text(s.replace('- The TUI fits a phone-width pane.',
                       '- A cache epoch change now says what moved.\n- The TUI fits a phone-width pane.'))
PY
  git add -A; git commit -qm 'feature adds an item under Unreleased'
  git checkout -q mainside
) > /dev/null 2>&1
if (cd "$MERGE_TMP" && git merge --no-edit feature > /dev/null 2>&1); then
  bad "the merge did not conflict — the item would be silently mis-filed"
else
  ok "merging a new Unreleased item into a released tree conflicts (not silent)"
fi
# With the glue present the same merge is silent — the failure this pins.
(
  cd "$MERGE_TMP" || exit 1
  git merge --abort 2>/dev/null || true
  python3 - <<'PY'
import pathlib, re
p = pathlib.Path('CHANGELOG.md'); s = p.read_text()
p.write_text(re.sub(r'(?m)^(## Unreleased)\n\n', r'\1\n', s))
PY
  git add -A; git commit -qm 'glue the junction (the defect)'
) > /dev/null 2>&1
if (cd "$MERGE_TMP" && git merge --no-edit feature > /dev/null 2>&1); then
  ok "with a glued junction the same merge is silent (reproduces the mis-filing)"
else
  bad "expected the glued merge to be silent; the reproduction does not hold"
fi
rm -rf "$MERGE_TMP"

# ---------------------------------------------------------------------------
head_ "9. the version-log row gets its PR number"
# `bump-version.sh` writes `PR #_(fill in)_` and that was the end of it - six
# rows on `main` shipped unrecorded. `release.sh` now fills the row from the
# number `gh pr create` returns, so the placeholder is a state that must not
# survive a release.
VLOG="$ROOT/scripts/lib/version_log.py"
cat > "$TMP/vlog.md" <<'VEOF'
# Product versions

| Date | Version | Link |
|------|---------|------|
| 2026-09-25 | **`5.7.0`** an older release | PR #201 |
VEOF
if python3 "$VLOG" set-pr "$TMP/vlog.md" 9.9.9 7 > /dev/null 2>&1; then
  bad "a version with no row was accepted"
else
  ok "a missing decision-log row is an error (not a silent no-op)"
fi

printf '| 2026-09-25 | **`6.0.0`** a release | PR #_(fill in)_ |\n' >> "$TMP/vlog.md"
python3 "$VLOG" set-pr "$TMP/vlog.md" 6.0.0 208 > /dev/null
grep -q 'PR #208' "$TMP/vlog.md" \
  && ok "the placeholder is replaced with the real PR number" \
  || bad "the placeholder survived: $(grep 6.0.0 "$TMP/vlog.md")"

# Resumed releases re-run this step; it must not clobber a recorded number.
python3 "$VLOG" set-pr "$TMP/vlog.md" 6.0.0 999 > /dev/null
if grep -q 'PR #208' "$TMP/vlog.md" && ! grep -q '999' "$TMP/vlog.md"; then
  ok "a second run leaves a recorded number alone (resume-safe)"
else
  bad "re-running overwrote a recorded PR number"
fi

if python3 "$VLOG" set-pr "$TMP/vlog.md" 6.0.0 abc > /dev/null 2>&1; then
  bad "a non-numeric PR was accepted"
else
  ok "a non-numeric PR number is rejected"
fi

grep -q 'version_log.py set-pr' "$ROOT/scripts/release.sh" \
  && ok "release.sh records the number it got from gh pr create" \
  || bad "release.sh does not call version_log.py"

# ---------------------------------------------------------------------------
head_ "10. wiring: bump-version.sh runs the shared mover"
grep -q 'changelog_release.py apply CHANGELOG.md' "$BUMP" \
  && ok "bump-version.sh calls the mover on the real path" \
  || bad "bump-version.sh does not call the mover"
grep -q 'changelog_release.py plan CHANGELOG.md' "$BUMP" \
  && ok "bump-version.sh calls the mover's plan in --dry-run" \
  || bad "bump-version.sh --dry-run does not use the mover's plan"
# The inline section builder is the original defect; its signature must be gone.
if grep -q "section = f'## " "$BUMP"; then
  bad "bump-version.sh still builds the section inline (the old defect code)"
else
  ok "no inline section builder remains in bump-version.sh"
fi

printf '\n%s passed, %s failed\n' "$PASS" "$FAIL"
[[ "$FAIL" -eq 0 ]] || exit 1
