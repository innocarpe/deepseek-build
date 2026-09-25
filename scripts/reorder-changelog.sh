#!/usr/bin/env bash
# Reorder CHANGELOG.md so the leading run of version sections is newest-first
# (SemVer descending, prereleases after their release) with "## Unreleased"
# pinned at the very top. A drifted Unreleased section anywhere below the run
# is pulled back up. Non-version sections (e.g. "## Prior unreleased notes
# (folded)") and everything after the leading run are left untouched.
#
# Invariant documented in docs/contributing/release-cycle.md:
#   # Changelog -> ## Unreleased (top) -> version sections newest-first.
#
# Usage:
#   ./scripts/reorder-changelog.sh            # rewrite CHANGELOG.md in place
#   ./scripts/reorder-changelog.sh --check    # exit 1 if order is wrong, no edits
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

CHECK=0
[[ "${1:-}" == "--check" ]] && CHECK=1

if [[ "$CHECK" -eq 1 ]]; then
  python3 - <<'PY'
import re, sys

def vkey(h):
    if h == 'Unreleased':
        return (10**9, 10**9, 10**9, 1, '', 0)
    m = re.match(r'^([0-9]+\.[0-9]+\.[0-9]+)(?:-([0-9A-Za-z.\-]+))?', h)
    if not m:
        return None
    core, pre = m.group(1), m.group(2)
    maj, min_, pat = (int(x) for x in core.split('.'))
    if not pre:
        return (maj, min_, pat, 1, '', 0)
    base, _, num = pre.partition('.')
    return (maj, min_, pat, 0, base, int(num) if num.isdigit() else 0)

text = open('CHANGELOG.md').read()
m = re.search(r'(?m)^## ', text)
if not m:
    sys.exit('error: no "## " sections in CHANGELOG.md')
body = text[m.start():]
starts = [mm.start() for mm in re.finditer(r'(?m)^## ', body)] + [len(body)]
sections = [body[starts[i]:starts[i + 1]] for i in range(len(starts) - 1)]

def heading(s):
    return re.match(r'(?m)^## ([^\n]+)', s).group(1).strip()

run = []
for s in sections:
    h = heading(s)
    if h == 'Unreleased' or vkey(h) is not None:
        run.append(s)
    else:
        break
if not run:
    sys.exit('error: first CHANGELOG section is not a version/Unreleased heading')

rest = sections[len(run):]
drifted = [s for s in rest if heading(s) == 'Unreleased']
rest = [s for s in rest if heading(s) != 'Unreleased']
run = run + drifted

# A glued junction is not an ordering problem, but it is a filing hazard: a
# branch that appends an item under Unreleased merges into a glued tree with no
# conflict and the item lands under the version heading. Measured: #209 landed
# under 5.7.0 and #206 under 6.0.0. Report it here because this is the check a
# release is already wired to run.
if re.search(r'(?m)^## Unreleased[ \t]*\n## ', text):
    sys.exit('CHANGELOG.md has a glued section junction: "## Unreleased" is '
             'immediately followed by a version heading — insert the blank line '
             '(a later merge would silently file new items under a version)')

keys = [vkey(heading(s)) for s in run]
if keys == sorted(keys, reverse=True):
    print('CHANGELOG.md order ok')
    sys.exit(0)
print('CHANGELOG.md NOT newest-first (Unreleased must be top)')
sys.exit(1)
PY
  exit $?
fi

python3 - <<'PY'
import re, sys

def vkey(h):
    if h == 'Unreleased':
        return (10**9, 10**9, 10**9, 1, '', 0)
    m = re.match(r'^([0-9]+\.[0-9]+\.[0-9]+)(?:-([0-9A-Za-z.\-]+))?', h)
    if not m:
        return None
    core, pre = m.group(1), m.group(2)
    maj, min_, pat = (int(x) for x in core.split('.'))
    if not pre:
        return (maj, min_, pat, 1, '', 0)
    base, _, num = pre.partition('.')
    return (maj, min_, pat, 0, base, int(num) if num.isdigit() else 0)

text = open('CHANGELOG.md').read()
m = re.search(r'(?m)^## ', text)
if not m:
    sys.exit('error: no "## " sections in CHANGELOG.md')
header, body = text[:m.start()], text[m.start():]
starts = [mm.start() for mm in re.finditer(r'(?m)^## ', body)] + [len(body)]
sections = [body[starts[i]:starts[i + 1]] for i in range(len(starts) - 1)]

def heading(s):
    return re.match(r'(?m)^## ([^\n]+)', s).group(1).strip()

run = []
for s in sections:
    h = heading(s)
    if h == 'Unreleased' or vkey(h) is not None:
        run.append(s)
    else:
        break
if not run:
    sys.exit('error: first CHANGELOG section is not a version/Unreleased heading')

rest = sections[len(run):]
drifted = [s for s in rest if heading(s) == 'Unreleased']
rest = [s for s in rest if heading(s) != 'Unreleased']
run = run + drifted

run.sort(key=lambda s: vkey(heading(s)), reverse=True)
out = header + ''.join(run) + ''.join(rest)
# Repair the glued junction while we are rewriting the file: the check above
# rejects it, and this is the command a reader is told to run.
if re.search(r'(?m)^## Unreleased[ \t]*\n## ', out):
    out = re.sub(r'(?m)^(## Unreleased[ \t]*)\n(## )', r'\1\n\n\2', out, count=1)
open('CHANGELOG.md', 'w').write(out)
print(f'reordered {len(run)} leading sections (Unreleased top, versions newest-first);')
print(f'  {len(rest)} sections below untouched')
PY
