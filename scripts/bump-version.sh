#!/usr/bin/env bash
# Bump the release version across the repo.
#
# Single source of truth: root Cargo.toml [workspace.package] version.
# This script keeps Cargo.toml, package.json, Cargo.lock, CHANGELOG.md and the
# docs/product/versions/README.md decision log in sync (ADR 0006 / versioning.md).
#
# Usage:
#   ./scripts/bump-version.sh 4.0.4 [--desc "one-line release note"] [--dry-run]
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

NEW=""
DESC=""
DRY=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --desc) DESC="$2"; shift 2 ;;
    --dry-run) DRY=1; shift ;;
    -h|--help) sed -n '1,12p' "$0"; exit 0 ;;
    -*) echo "unknown option: $1" >&2; exit 1 ;;
    *) NEW="$1"; shift ;;
  esac
done

if [[ -z "$NEW" ]]; then
  echo "usage: $0 <MAJOR.MINOR.PATCH> [--desc \"...\"] [--dry-run]" >&2
  exit 1
fi
if [[ ! "$NEW" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "error: '$NEW' is not full SemVer MAJOR.MINOR.PATCH (e.g. 4.0.4)" >&2
  exit 1
fi

OLD="$(python3 - <<'PY'
import re
s = open('Cargo.toml').read()
i = s.index('[workspace.package]')
m = re.search(r'(?m)^version = "([^"]+)"', s[i:])
print(m.group(1) if m else '')
PY
)"
if [[ -z "$OLD" ]]; then
  echo "error: could not read [workspace.package] version from Cargo.toml" >&2
  exit 1
fi
if [[ "$OLD" == "$NEW" ]]; then
  echo "bump-version: already at $NEW" >&2
  exit 1
fi

if [[ "$DRY" -eq 1 ]]; then
  echo "bump-version (dry-run): $OLD -> $NEW"
  echo "  would edit: Cargo.toml, package.json, Cargo.lock (via cargo check), CHANGELOG.md, docs/product/versions/README.md"
  [[ -n "$DESC" ]] && echo "  desc: $DESC"
  exit 0
fi

if [[ -n "$(git status --porcelain)" ]]; then
  echo "error: working tree is not clean; commit or stash first" >&2
  git status --porcelain | head -20 >&2
  exit 1
fi

echo "bump-version: $OLD -> $NEW"

# Cargo.toml + package.json (targeted regex keeps formatting byte-identical).
python3 - "$NEW" <<'PY'
import re, sys
new = sys.argv[1]

s = open('Cargo.toml').read()
i = s.index('[workspace.package]')
rest = s[i:]
rest, n = re.subn(r'(?m)^version = "[^"]+"', f'version = "{new}"', rest, count=1)
if n != 1:
    sys.exit('error: could not update Cargo.toml [workspace.package] version')
open('Cargo.toml', 'w').write(s[:i] + rest)

s = open('package.json').read()
s, n = re.subn(r'("version"\s*:\s*)"[^"]+"', rf'\g<1>"{new}"', s, count=1)
if n != 1:
    sys.exit('error: could not update package.json version')
open('package.json', 'w').write(s)
PY

# Regenerate Cargo.lock (syncs the dsb-* workspace entries to the new version).
cargo check -p dsb-cli >/dev/null

# CHANGELOG section + versions README decision-log row.
python3 - "$NEW" "$(date +%Y-%m-%d)" "$OLD" "$DESC" <<'PY'
import re, sys
new, date, old, desc = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4]

s = open('CHANGELOG.md').read()
m = re.search(r'(?m)^## Unreleased[ \t]*\n', s)
if not m:
    sys.exit('error: no "## Unreleased" heading in CHANGELOG.md')
nxt = re.search(r'(?m)^## ', s[m.end():])
at = m.end() + (nxt.start() if nxt else len(s) - m.end())
note = desc if desc else '_release notes: fill in before merge_'
section = f'## {new} — {date}\n\n- {note}\n\n'
open('CHANGELOG.md', 'w').write(s[:at] + section + s[at:])

s = open('docs/product/versions/README.md').read()
needle = f'**`{old}`**'
lines = s.split('\n')
out = []
inserted = False
for line in lines:
    out.append(line)
    if not inserted and needle in line:
        note = desc if desc else '_short description (fill in)_'
        out.append(f'| {date} | **`{new}`** {note} | PR #_(fill in)_ |')
        inserted = True
if inserted:
    open('docs/product/versions/README.md', 'w').write('\n'.join(out) + '\n')
else:
    print(f'bump-version: warn: could not find row for `{old}` in versions README; add the {new} row manually')
PY

echo "bump-version: done. Next: fill CHANGELOG.md notes, then ./scripts/release.sh $NEW"
