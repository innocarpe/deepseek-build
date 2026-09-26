#!/usr/bin/env python3
"""Hermetic lock for the unit close.

The live tree must pass. A copy with the 2026-09-26 description window, or
with the old "PR is open" checklist line, must fail. Brief fixtures reproduce
the release-lane ban and the withheld merge. Merge fixtures never call gh.
"""

from __future__ import annotations

import json
import shutil
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts" / "lib"))

import pr_merged  # noqa: E402
import session_close  # noqa: E402

SURFACE_FILES = [
    "AGENTS.md",
    "skills/session-unit/SKILL.md",
    "skills/release/SKILL.md",
    "skills/worktree-dispatch/SKILL.md",
    "skills/pr-authoring/SKILL.md",
    "docs/contributing/review-checklist.md",
    "docs/product/ULTRAGOAL_PROMPT_COLD_START_DEEPSEEK_DEPTH_6X.md",
]

OLD_SESSION_DESCRIPTION = (
    "Use when a session starts, drifts off the opening ask, a follow-up unit "
    "appears, or the user says to focus or finish: do that one unit through "
    "the PR, then hand the next to a new Orca tab."
)


def fail(message: str) -> None:
    print(f"bad: {message}", file=sys.stderr)
    raise SystemExit(1)


def ok(message: str) -> None:
    print(f"ok: {message}")


def copy_tree(dest: Path) -> None:
    for rel in SURFACE_FILES:
        target = dest / rel
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy(ROOT / rel, target)


def replace_description(path: Path, description: str) -> None:
    text = path.read_text(encoding="utf-8")
    end = text.find("\n---", 3)
    if end < 0:
        fail(f"no frontmatter in {path}")
    front, rest = text[: end + 4], text[end + 4 :]
    lines = front.splitlines()
    out: list[str] = []
    skipping = False
    replaced = False
    for line in lines:
        if line.startswith("description:"):
            out.append(f'description: "{description}"')
            skipping = True
            replaced = True
            continue
        if skipping:
            if line.startswith("---") or (
                line and not line.startswith((" ", "\t"))
            ):
                skipping = False
                out.append(line)
            continue
        out.append(line)
    if not replaced:
        fail(f"no description in {path}")
    path.write_text("\n".join(out) + rest, encoding="utf-8")


def main() -> None:
    live = session_close.check_root(ROOT)
    if live:
        fail("live tree:\n" + "\n".join(live))
    ok("live tree passes")

    window = session_close.description_of(
        (ROOT / "skills/session-unit/SKILL.md").read_text(encoding="utf-8")
    )[: session_close.CODEX_WINDOW]
    if "through the PR, then hand" in window:
        fail("the measured 2026-09-26 window is still the live description")
    ok("live session-unit window is not the 2026-09-26 cut")

    with tempfile.TemporaryDirectory() as tmp:
        dest = Path(tmp)
        copy_tree(dest)
        skill = dest / "skills/session-unit/SKILL.md"
        replace_description(skill, OLD_SESSION_DESCRIPTION)
        errors = session_close.check_root(dest)
        if not any("merge commit" in error and "session-unit" in error for error in errors):
            fail(f"old description did not fail the window: {errors}")
        ok("e914dfb session-unit description fails the 160-char window")

        copy_tree(dest)
        live_desc = session_close.description_of(
            (ROOT / "skills/session-unit/SKILL.md").read_text(encoding="utf-8")
        )
        replace_description(skill, ("note " * 30) + live_desc)
        errors = session_close.check_root(dest)
        if not any("first 160 chars lack" in error for error in errors):
            fail(f"padding the finish line past 160 did not fail: {errors}")
        ok("finish line past the first 160 chars fails")

        copy_tree(dest)
        body = (dest / "skills/session-unit/SKILL.md").read_text(encoding="utf-8")
        body += "\nThe PR is open, or the opening explicitly stopped earlier\n"
        (dest / "skills/session-unit/SKILL.md").write_text(body, encoding="utf-8")
        errors = session_close.check_root(dest)
        if not any("old finish line" in error for error in errors):
            fail(f"restoring the open-PR checklist did not fail: {errors}")
        ok("open-PR checklist line fails")

    briefs = {
        "version ask plus release-lane ban": (
            "user-turn: 새 버전을 올려라\n"
            "Do not touch the 6.0.0 release lane.\n",
            True,
        ),
        "version ask and no ban": (
            "user-turn: ship a version of the depth train\n"
            "Cut it with skills/release.\n",
            False,
        ),
        "docs fix may keep a release-lane ban": (
            "user-turn: fix the prompt fold\n"
            "Do not touch the release lane.\n",
            False,
        ),
        "user said stop before merge": (
            "user-turn: stop before merge, I want to look at the PR first\n"
            "Do not merge.\n",
            False,
        ),
        "withheld merge without a user stop": (
            "user-turn: land the fold fix\n"
            "Stop before the PR.\n",
            True,
        ),
        "missing user-turn": (
            "Do not touch the release lane.\n",
            True,
        ),
    }
    for name, (text, expect_fail) in briefs.items():
        errors = session_close.check_brief(text)
        failed = bool(errors)
        if failed != expect_fail:
            fail(f"brief {name}: errors={errors} expect_fail={expect_fail}")
        ok(f"brief: {name}")

    green = {
        "state": "MERGED",
        "mergedAt": "2026-09-26T00:00:00Z",
        "mergeCommit": {"oid": "abc123"},
        "parents": 2,
        "checks": [
            {"name": "CI / required", "bucket": "pass"},
            {"name": "CI / fmt", "bucket": "skipping"},
        ],
    }
    code, lines = pr_merged.judge(green)
    if code != 0 or "pass" not in lines:
        fail(f"green merge fixture: {code} {lines}")
    ok("merged fixture passes")

    cases = [
        ("open", {**green, "state": "OPEN", "mergedAt": None}, 1),
        ("squash", {**green, "parents": 1}, 1),
        ("no commit", {**green, "mergeCommit": None}, 1),
        ("pending", {**green, "checks": [{"name": "CI / required", "bucket": "pending"}]}, 2),
        ("failed required", {**green, "checks": [{"name": "CI / required", "bucket": "fail"}]}, 1),
        ("no checks", {**green, "checks": []}, 2),
    ]
    for name, doc, expect in cases:
        code, lines = pr_merged.judge(doc)
        if code != expect:
            fail(f"merge fixture {name}: code {code} lines {lines}")
        if "pass" in lines and code != 0:
            fail(f"merge fixture {name} printed pass on a failure")
        ok(f"merge fixture: {name}")

    # The shell wrappers are what sessions run. Exercise them on a fixture.
    wrapper = ROOT / "scripts" / "check-pr-merged.sh"
    fixture = Path(tempfile.mkdtemp()) / "green.json"
    fixture.write_text(json.dumps(green), encoding="utf-8")
    import subprocess

    run = subprocess.run(
        [str(wrapper), "--fixture", str(fixture)],
        text=True,
        capture_output=True,
    )
    if run.returncode != 0 or "pass" not in run.stdout.splitlines():
        fail(f"check-pr-merged.sh wrapper: {run.returncode} {run.stdout} {run.stderr}")
    ok("check-pr-merged.sh wrapper")

    brief_run = subprocess.run(
        [
            str(ROOT / "scripts" / "check-session-close.sh"),
            "brief",
            str(fixture),  # not a brief; replaced below
        ],
        text=True,
        capture_output=True,
    )
    # fixture json has no user-turn, so the brief wrapper must fail closed.
    if brief_run.returncode == 0:
        fail("brief wrapper accepted a file with no user-turn")
    ok("check-session-close.sh brief wrapper fails closed")

    live_run = subprocess.run(
        [str(ROOT / "scripts" / "check-session-close.sh")],
        text=True,
        capture_output=True,
    )
    if live_run.returncode != 0 or live_run.stdout.strip() != "pass":
        fail(f"check-session-close.sh live: {live_run.stdout} {live_run.stderr}")
    ok("check-session-close.sh live")


if __name__ == "__main__":
    main()
