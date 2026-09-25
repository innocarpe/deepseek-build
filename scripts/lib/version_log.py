#!/usr/bin/env python3
"""Record the release PR number in the version-log row `bump-version.sh` adds.

`bump-version.sh` writes the decision-log row before the release PR exists, so
it writes `PR #_(fill in)_` and nothing ever came back to fill it in: measured
on `main`, six rows still read `PR #_(fill in)_` (`4.0.4`, `5.2.0`, `5.2.2`,
`5.5.3`, `5.5.4`, `6.0.0`). The MAJOR gate reads this file, so the row is part
of the release record, not decoration.

`release.sh` knows the number the moment `gh pr create` returns, so it calls
this and commits the filled row into the release PR — the number lands with the
release instead of waiting for a later cleanup.

    version_log.py set-pr <versions-README> <version> <pr-number>
        Fill the row for <version>. Exits non-zero when the row is missing or
        the placeholder is already gone, so a silent no-op cannot pass for a
        recorded number.
"""

import re
import sys


def fill(text, version, pr):
    """Return (text, changed, note) with the placeholder replaced.

    Idempotent so a resumed release (`--skip-bump` / `--skip-pr`) can re-run the
    step: a row that already carries a number is left alone and reported, not
    treated as an error. A missing row, an absent PR column or a malformed
    number is an error — those mean the release record cannot be completed, and
    a silent no-op would pass for a recorded number.
    """
    if not re.fullmatch(r'[0-9]+', str(pr)):
        raise ValueError(f'not a PR number: {pr!r}')
    lines = text.split('\n')
    for i, line in enumerate(lines):
        if f'**`{version}`**' not in line:
            continue
        if 'PR #_(fill in)_' in line:
            lines[i] = line.replace('PR #_(fill in)_', f'PR #{pr}')
            return '\n'.join(lines), True, ''
        m = re.search(r'PR #([0-9]+)', line)
        if m:
            return text, False, f'{version} already records PR #{m.group(1)}'
        raise ValueError(f'the row for {version} has no PR column: {line.strip()[:90]}')
    raise ValueError(f'no decision-log row for {version}')


def main(argv):
    if len(argv) != 4 or argv[0] != 'set-pr':
        sys.stderr.write('usage: version_log.py set-pr <versions-README> <version> <pr>\n')
        return 2
    _, path, version, pr = argv
    text = open(path).read()
    try:
        out, changed, note = fill(text, version, pr)
    except ValueError as e:
        sys.stderr.write(f'error: {e}\n')
        return 1
    if changed:
        open(path, 'w').write(out)
        print(f'version log: {version} -> PR #{pr}')
    else:
        print(f'version log: {note}')
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
