#!/usr/bin/env python3
"""Move `## Unreleased` items into the new version section on a release bump.

`bump-version.sh` used to insert an empty version section and seed it with the
`--desc` one-liner, leaving whatever the `## Unreleased` section already held
in place. Both then shipped as "unreleased": measured on `v5.7.0`, the tag tree
carried seven items below `## Unreleased` that the release itself had shipped,
while `## 5.7.0` held only the `--desc` summary — so the release record did not
say what the release did (and a following major would read those items as its
own work).

This module is the one place that moves them, so `bump-version.sh --dry-run`
and the real bump cannot disagree about what will happen.

    changelog_release.py plan  <changelog> <new-version>
        Print what a bump would do. Reads only; exits 0.

    changelog_release.py apply <changelog> <new-version> <date> [desc]
        Insert `## <new-version> — <date>` below `## Unreleased`, moving the
        Unreleased items into it **verbatim**. With no items, the section is
        seeded with the `--desc` note (or a fill-in placeholder). Rewrites the
        file; exits non-zero if the file is not newest-first afterwards.

Rules the tests pin down:

* item text is copied byte for byte — a release record is not a rewrite;
* a `--desc` note never sits above items it does not describe (it is a
  fallback for an empty Unreleased, reported as unused when items exist);
* an empty Unreleased keeps the previous behavior: one `- <note>` line;
* a drifted `## Unreleased` is pulled back to the top;
* version sections stay newest-first.
"""

import re
import sys

FALLBACK_NOTE = '_release notes: fill in before merge_'


def vkey(v):
    """Sort key for a version heading: releases above their prereleases."""
    core, _, pre = v.partition('-')
    maj, min_, pat = (int(x) for x in core.split('.'))
    if not pre:
        return (maj, min_, pat, 1, '')
    base, _, num = pre.partition('.')
    return (maj, min_, pat, 0, base, int(num) if num.isdigit() else 0)


def parse(text):
    """Split `text` into (header, unreleased_body, rest).

    `rest` is everything after the Unreleased section, with the section itself
    removed. The Unreleased heading may sit anywhere; the caller re-emits it.
    """
    m = re.search(r'(?m)^## Unreleased[ \t]*\n', text)
    if not m:
        raise ValueError('no "## Unreleased" section in CHANGELOG.md')
    nxt = re.search(r'(?m)^## ', text[m.end():])
    stop = m.end() + (nxt.start() if nxt else len(text) - m.end())
    body = text[m.end():stop]
    rest = text[:m.start()] + text[stop:]
    if rest.startswith('# Changelog'):
        rest = rest[len('# Changelog\n'):].lstrip('\n')
    return body, rest


def extract_items(body):
    """Split an Unreleased body into (items, residue).

    An item starts at a line beginning `- ` and runs to the next such line; a
    blank line between items stays with the item above it, so the original
    spacing survives the move. Anything before the first item is residue and
    stays in `## Unreleased` (it is not a release note).
    """
    lines = body.splitlines(keepends=True)
    items = []
    residue = []
    current = None
    for line in lines:
        if re.match(r'^- ', line):
            if current is not None:
                items.append(''.join(current))
            current = [line]
        elif current is not None:
            current.append(line)
        else:
            residue.append(line)
    if current is not None:
        items.append(''.join(current))
    # Trailing blank lines belong to the section, not to the last item.
    trimmed = []
    for item in items:
        item = re.sub(r'\n+$', '\n', item)
        trimmed.append(item)
    return trimmed, ''.join(residue)


def build(text, new, date, desc):
    """Return (new_text, moved_count, desc_used) for a bump to `new`."""
    body, rest = parse(text)
    items, residue = extract_items(body)

    if items:
        section = f'## {new} — {date}\n\n' + ''.join(items) + '\n'
        desc_used = False
    else:
        note = desc or FALLBACK_NOTE
        section = f'## {new} — {date}\n\n- {note}\n\n'
        desc_used = bool(desc)

    unrel = '## Unreleased\n' + residue
    out = '# Changelog\n\n' + unrel + section + rest
    out = re.sub(r'\n{3,}', '\n\n', out).rstrip('\n') + '\n'

    vers = re.findall(r'(?m)^## ([0-9]+\.[0-9]+\.[0-9]+)(?:-[0-9A-Za-z.\-]+)?[ \t]', out)
    if [vkey(v) for v in vers] != sorted((vkey(v) for v in vers), reverse=True):
        raise ValueError('CHANGELOG.md version sections are not newest-first '
                         'after the bump — run ./scripts/reorder-changelog.sh '
                         'and commit before bumping')
    return out, len(items), desc_used


def main(argv):
    if len(argv) < 3:
        sys.stderr.write(
            'usage: changelog_release.py plan|apply <changelog> <version> [date] [desc]\n')
        return 2
    mode, path, new = argv[0], argv[1], argv[2]
    text = open(path).read()

    if mode == 'plan':
        date = argv[3] if len(argv) > 3 else 'YYYY-MM-DD'
        desc = argv[4] if len(argv) > 4 else ''
        try:
            body, _ = parse(text)
            items, residue = extract_items(body)
            # Fail the plan the same way the apply would, so a dry-run cannot
            # promise a bump the real run later rejects.
            build(text, new, date, desc)
        except ValueError as e:
            print(f'changelog: {e}', file=sys.stderr)
            return 1
        if items:
            print(f'changelog: {len(items)} Unreleased item(s) would move into '
                  f'"## {new} — {date}"; --desc will not seed that section')
        else:
            extra = ('--desc would seed the new section' if desc else
                     FALLBACK_NOTE + ' would be seeded')
            print(f'changelog: Unreleased is empty; {extra}')
        if residue.strip():
            print('changelog: warning: Unreleased holds non-item text that stays behind')
        return 0

    if mode != 'apply':
        sys.stderr.write(f'unknown mode: {mode}\n')
        return 2
    if len(argv) < 4:
        sys.stderr.write('apply needs a date: changelog_release.py apply <file> <ver> <date> [desc]\n')
        return 2

    date = argv[3]
    desc = argv[4] if len(argv) > 4 else ''
    try:
        out, moved, desc_used = build(text, new, date, desc)
    except ValueError as e:
        sys.stderr.write(f'error: {e}\n')
        return 1
    open(path, 'w').write(out)
    if moved:
        print(f'bump-version: moved {moved} Unreleased item(s) into "## {new}"'
              + (' (--desc did not seed the section; the items outrank a summary)'
                 if desc else ''))
    else:
        print(f'bump-version: seeded "## {new}" with '
              + ('the --desc note' if desc_used else 'the fill-in placeholder'))
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
