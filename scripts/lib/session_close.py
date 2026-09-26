#!/usr/bin/env python3
"""Fail when a session can stop before the unit's close.

Codex loads about the first 160 characters of a skill description (measured
2026-09-23, 163–166). Grok loads about the first 400 UTF-8 bytes. On
2026-09-26 the session-unit description was 188 characters and its first 160
ended at "through the PR, then hand". The body already said to merge. Sessions
stopped at the window.

This module locks the phrases that have to sit inside that 160-character
window, and it rejects a handoff brief whose prohibition contradicts the
quoted user turn. It does not classify intent. It compares two strings in
the same brief.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

CODEX_WINDOW = 160

# Phrases that must appear inside the Codex window, not merely in the file.
DESCRIPTION_WINDOW = {
    "skills/session-unit/SKILL.md": (
        "the PR",
        "CI",
        "merge commit",
        "the report",
    ),
    "skills/release/SKILL.md": (
        "ship a version",
        "child brief",
        "release lane",
    ),
    "skills/worktree-dispatch/SKILL.md": ("cannot shrink the user's turn",),
    "skills/pr-authoring/SKILL.md": (
        "not the end",
        "session-unit",
    ),
}

# Phrases the body must keep so the slip record and the close checklist
# cannot move back into a paragraph the window does not show.
BODY_NEEDLES = {
    "skills/session-unit/SKILL.md": (
        "e914dfb",
        "through the PR, then hand",
        "MERGED",
        "mergedAt",
        "mergeCommit",
        "Session disposition",
        "Close now",
        "More in this session",
        "To hand off",
    ),
    "skills/release/SKILL.md": (
        "An empty `## Unreleased` is not evidence that a version ask is finished.",
    ),
    "skills/worktree-dispatch/SKILL.md": ("check-session-close.sh brief",),
    "skills/pr-authoring/SKILL.md": ("The unit is not done",),
    "docs/contributing/review-checklist.md": (
        "does not end the unit at an open PR",
    ),
    "docs/product/ULTRAGOAL_PROMPT_COLD_START_DEEPSEEK_DEPTH_6X.md": (
        "do not copy this bullet into a new brief",
    ),
}

BODY_FORBIDDEN = {
    "skills/session-unit/SKILL.md": (
        "The PR is open, or the opening explicitly stopped earlier",
    ),
    "AGENTS.md": ("then the PR. Stop short of push or PR only when the",),
}

AGENTS_SECTION = "## One session, one unit"
AGENTS_SECTION_NEEDLES = (
    "merge commit",
    "which clause",
    "child brief",
    "release lane",
    "scripts/check-pr-merged.sh",
    "scripts/check-session-close.sh brief",
    "Session disposition",
)
AGENTS_CLAIM_NEEDLE = "do not end the unit"

USER_TURN_RE = re.compile(r"(?im)^[ \t]*user-turn:[ \t]*(.+?)[ \t]*$")

# A live ban in the brief body. Matching the user-turn line itself does not
# count: that line is the user, not the brief.
# Imperative bans only. Mentioning the words "release lane" in a warning is
# not a ban; "do not touch the release lane" is the sentence that was copied.
RELEASE_BAN_RE = re.compile(
    r"do not touch the [`'\"]?\d+\.\d+\.\d+[`'\"]? release"
    r"|do not touch the release lane"
    r"|don't touch the release lane"
    r"|릴리스 레인을 건드리지"
    r"|릴리스 레인 금지"
    r"|(?:do not|don't) touch[^\n]{0,80}(?:CHANGELOG|npm publish|version bump)",
    re.IGNORECASE,
)
VERSION_ASK_RE = re.compile(
    r"ship a version|ship the version|publish to npm|bump|release"
    r"|새 버전|버전을 올",
    re.IGNORECASE,
)
USER_STOP_RELEASE_RE = re.compile(
    r"do not release|don't release|stop before (?:the )?release"
    r"|do not (?:bump|ship|publish)|don't (?:bump|ship|publish)"
    r"|버전은 올리지|릴리스는 하지|릴리스 하지|릴리스 레인",
    re.IGNORECASE,
)
CLOSE_WITHHELD_RE = re.compile(
    r"do not merge|don't merge|stop before merge|stop before the PR"
    r"|do not open (?:a |the )?PR|don't open (?:a |the )?PR"
    r"|머지하지|PR 전에 멈추|PR을 열지",
    re.IGNORECASE,
)
USER_STOP_MERGE_RE = re.compile(
    r"stop before merge|do not merge|don't merge|review only|look at first"
    r"|머지 전에|리뷰만|머지하지",
    re.IGNORECASE,
)


def repo_root_from_here() -> Path:
    return Path(__file__).resolve().parents[2]


def description_of(text: str) -> str:
    if not text.startswith("---"):
        raise ValueError("SKILL.md has no frontmatter")
    end = text.find("\n---", 3)
    if end < 0:
        raise ValueError("SKILL.md frontmatter does not close")
    lines = text[4:end].splitlines()
    for i, line in enumerate(lines):
        if not line.startswith("description:"):
            continue
        rest = line[len("description:") :].strip()
        if rest in {">", "|", ">-", "|-", ">+", "|+"}:
            fold = rest.startswith(">")
            buf: list[str] = []
            for nxt in lines[i + 1 :]:
                if nxt and not nxt.startswith((" ", "\t")):
                    break
                buf.append(nxt.strip())
            if fold:
                return " ".join(part for part in buf if part)
            return "\n".join(buf).strip()
        if len(rest) >= 2 and rest[0] == rest[-1] and rest[0] in {'"', "'"}:
            return rest[1:-1]
        return rest
    raise ValueError("SKILL.md has no description")


def collapsed(text: str) -> str:
    return re.sub(r"\s+", " ", text)


def section_after(text: str, heading: str) -> str:
    start = text.find(heading)
    if start < 0:
        raise ValueError(f"missing heading {heading}")
    rest = text[start + len(heading) :]
    nxt = rest.find("\n## ")
    return rest if nxt < 0 else rest[:nxt]


def check_root(root: Path) -> list[str]:
    errors: list[str] = []
    for rel, needles in DESCRIPTION_WINDOW.items():
        path = root / rel
        if not path.is_file():
            errors.append(f"missing {rel}")
            continue
        try:
            desc = description_of(path.read_text(encoding="utf-8"))
        except ValueError as exc:
            errors.append(f"{rel}: {exc}")
            continue
        window = desc[:CODEX_WINDOW]
        for needle in needles:
            if needle not in window:
                errors.append(
                    f"{rel}: first {CODEX_WINDOW} chars lack {needle!r}: {window!r}"
                )
    for rel, needles in BODY_NEEDLES.items():
        path = root / rel
        if not path.is_file():
            errors.append(f"missing {rel}")
            continue
        body = collapsed(path.read_text(encoding="utf-8"))
        for needle in needles:
            if needle not in body:
                errors.append(f"{rel}: body lacks {needle!r}")
    for rel, needles in BODY_FORBIDDEN.items():
        path = root / rel
        if not path.is_file():
            errors.append(f"missing {rel}")
            continue
        body = collapsed(path.read_text(encoding="utf-8"))
        for needle in needles:
            if needle in body:
                errors.append(f"{rel}: still contains the old finish line {needle!r}")
    agents = root / "AGENTS.md"
    if not agents.is_file():
        errors.append("missing AGENTS.md")
    else:
        raw = agents.read_text(encoding="utf-8")
        text = collapsed(raw)
        try:
            block = collapsed(section_after(raw, AGENTS_SECTION))
        except ValueError as exc:
            errors.append(f"AGENTS.md: {exc}")
            block = ""
        for needle in AGENTS_SECTION_NEEDLES:
            if needle not in block:
                errors.append(f"AGENTS.md §One session, one unit lacks {needle!r}")
        if AGENTS_CLAIM_NEEDLE not in text:
            errors.append(f"AGENTS.md lacks {AGENTS_CLAIM_NEEDLE!r}")
    return errors


def check_brief(text: str) -> list[str]:
    turns = [match.group(1).strip() for match in USER_TURN_RE.finditer(text)]
    if not turns:
        return ["brief has no user-turn: line"]
    user = " ".join(turns)
    body = "\n".join(
        line for line in text.splitlines() if not USER_TURN_RE.match(line)
    )
    errors: list[str] = []
    if (
        RELEASE_BAN_RE.search(body)
        and VERSION_ASK_RE.search(user)
        and not USER_STOP_RELEASE_RE.search(user)
    ):
        errors.append(
            "brief bans the release lane but user-turn still asks for a version"
        )
    if CLOSE_WITHHELD_RE.search(body) and not USER_STOP_MERGE_RE.search(user):
        errors.append(
            "brief withholds the PR or the merge but user-turn does not say to stop"
        )
    return errors


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=None)
    parser.add_argument("--brief", type=Path, default=None)
    args = parser.parse_args(argv)
    if args.brief is not None:
        errors = check_brief(args.brief.read_text(encoding="utf-8"))
    else:
        root = args.root if args.root is not None else repo_root_from_here()
        errors = check_root(root)
    if errors:
        for error in errors:
            print(f"fail: {error}")
        return 1
    print("pass")
    return 0


if __name__ == "__main__":
    sys.exit(main())
