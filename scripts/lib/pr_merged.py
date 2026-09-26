#!/usr/bin/env python3
"""A merge is a fact only when gh pr view says so.

Exit 0 prints `pass` when state is MERGED, mergedAt and mergeCommit.oid are
non-empty, the merge commit has two parents, and CI's required check passed.
Exit 2 means CI has not finished. Exit 1 is any other miss. The session
report pastes this output; a sentence that says "merged" without it is not
the fact.
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path
from typing import Any


def _parents(doc: dict[str, Any]) -> int | None:
    raw = doc.get("parents")
    if isinstance(raw, int):
        return raw
    if isinstance(raw, list):
        return len(raw)
    return None


def judge(doc: dict[str, Any]) -> tuple[int, list[str]]:
    """Return (exit code, output lines). Lines always start with pass or fail."""
    lines: list[str] = []
    state = doc.get("state")
    merged_at = doc.get("mergedAt")
    commit = doc.get("mergeCommit") or {}
    oid = commit.get("oid") if isinstance(commit, dict) else None
    parents = _parents(doc)
    checks = doc.get("checks") or []
    lines.append(f"state={state}")
    lines.append(f"mergedAt={merged_at or ''}")
    lines.append(f"mergeCommit={oid or ''}")
    lines.append(f"parents={parents if parents is not None else ''}")
    if isinstance(checks, list):
        for check in checks:
            if isinstance(check, dict):
                lines.append(
                    f"check {check.get('name')}={check.get('bucket') or check.get('state')}"
                )

    failures: list[str] = []
    if state != "MERGED":
        failures.append(f"state is {state!r}, need MERGED")
    if not merged_at:
        failures.append("mergedAt is empty")
    if not oid:
        failures.append("mergeCommit.oid is empty")
    if parents != 2:
        failures.append(
            f"merge commit parents={parents}, need 2 (a merge commit, not a squash)"
        )

    pending = False
    if not isinstance(checks, list) or not checks:
        pending = True
        failures.append("no CI checks yet")
    else:
        buckets = []
        for check in checks:
            if not isinstance(check, dict):
                continue
            bucket = (check.get("bucket") or "").lower()
            buckets.append((check.get("name") or "", bucket))
        if any(bucket in {"fail", "cancel"} for _, bucket in buckets):
            failures.append("CI did not pass")
        elif any(bucket == "pending" for _, bucket in buckets):
            pending = True
            failures.append("CI still pending")
        else:
            required = [bucket for name, bucket in buckets if "required" in name.lower()]
            if "pass" not in required:
                failures.append("CI / required did not pass")

    if failures:
        for failure in failures:
            lines.append(f"fail: {failure}")
        # Pending with an otherwise merged PR is "not yet", not a red PR.
        hard = [item for item in failures if item not in {"CI still pending", "no CI checks yet"}]
        if pending and not hard:
            return 2, lines
        return 1, lines
    lines.append("pass")
    return 0, lines


def fetch_live(pr: str, repo: str) -> dict[str, Any]:
    if not os.environ.get("GH_TOKEN"):
        token = subprocess.check_output(
            ["gh", "auth", "token", "--user", "innocarpe"], text=True
        ).strip()
        os.environ["GH_TOKEN"] = token
    view = subprocess.check_output(
        [
            "gh",
            "pr",
            "view",
            pr,
            "--repo",
            repo,
            "--json",
            "state,mergedAt,mergeCommit,url",
        ],
        text=True,
    )
    doc = json.loads(view)
    commit = doc.get("mergeCommit") or {}
    oid = commit.get("oid") if isinstance(commit, dict) else None
    if oid:
        parents = subprocess.check_output(
            [
                "gh",
                "api",
                f"repos/{repo}/commits/{oid}",
                "--jq",
                ".parents | length",
            ],
            text=True,
        ).strip()
        doc["parents"] = int(parents)
    else:
        doc["parents"] = None
    checks = subprocess.run(
        [
            "gh",
            "pr",
            "checks",
            pr,
            "--repo",
            repo,
            "--json",
            "name,bucket,state",
        ],
        text=True,
        capture_output=True,
    )
    if checks.stdout.strip():
        doc["checks"] = json.loads(checks.stdout)
    else:
        doc["checks"] = []
    return doc


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--fixture", type=str, default=None)
    parser.add_argument("--pr", default=None)
    parser.add_argument("--repo", default="innocarpe/deepseek-build")
    args = parser.parse_args(argv)
    if args.fixture:
        doc = json.loads(Path(args.fixture).read_text(encoding="utf-8"))
    elif args.pr:
        doc = fetch_live(args.pr, args.repo)
    else:
        print("fail: pass --fixture or --pr", file=sys.stderr)
        return 1
    code, lines = judge(doc)
    print("\n".join(lines))
    return code


if __name__ == "__main__":
    sys.exit(main())
