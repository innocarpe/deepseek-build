#!/usr/bin/env python3
"""Wait for a pull request's checks to finish, then say whether it can merge.

`release.sh` merged the release PR the instant `gh pr create` returned. GitHub
computes a pull request's mergeability asynchronously and the checks had just
started, so the merge answered

    GraphQL: Pull Request is not mergeable (mergePullRequest)

on 6.2.0 (PR #258, published as 6.1.1), 6.7.1 (#311, published as 6.1.7) and
6.8.0 (#323, published as 6.1.10). Each time the release stopped with the PR
open and a person merged it and resumed the script with
`--skip-bump --skip-pr`. Merging ahead of the checks also risks the worse
shape: the tag and the prebuilt build starting from a merge whose CI has not
finished.

This module polls `gh pr view <pr> --json state,mergeable,statusCheckRollup`
and exits 0 only when every check the PR reports is complete with none failed
or cancelled, and GitHub reports the PR `MERGEABLE`. A failed check stops the
wait at that read — before the merge call the caller would otherwise make.
`main` here enforces no required status check, so `gh pr checks --required`
has nothing to report; the PR's rollup (CI's always-on `required` job
included) is what this reads.

Usage:
  scripts/lib/pr_checks.py --pr 123 [--timeout SEC] [--interval SEC]

  --timeout SEC   total wait window        (default 3600)
  --interval SEC  poll interval            (default 10)
                  DSB_PR_CHECKS_TIMEOUT_SEC / DSB_PR_CHECKS_INTERVAL_SEC do
                  the same by environment; a flag wins.

Exit status: 0 ready to merge (or already merged); 1 not ready — a failed
check, a conflicting or closed PR, `gh` failing repeatedly, or the deadline
passing; 2 usage error. Progress goes to stdout and diagnostics to stderr, so
`release.sh` can wrap the failure with what to run next.
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import time
from typing import Any

# A check run that has not reported a conclusion yet (and a status context
# whose state is one of its not-finished values) is pending. Anything else
# that is neither pending nor one of PASSED below counts as a stop: FAILURE,
# CANCELLED, TIMED_OUT, ACTION_REQUIRED, ERROR, STARTUP_FAILURE, STALE.
PENDING = {"PENDING", "QUEUED", "IN_PROGRESS", "EXPECTED", "WAITING", "REQUESTED"}
PASSED = {"SUCCESS", "NEUTRAL", "SKIPPED"}

# `gh` failing this many times in a row is a broken call — a bad PR number, a
# missing credential, no network — not a transient. Stop instead of sitting
# out the whole window and reporting a timeout.
MAX_GH_ERRORS = 5

FIELDS = "state,mergeable,statusCheckRollup"


def fetch(pr: str) -> tuple[dict[str, Any] | None, str]:
    """One `gh pr view`. Returns (document, "") or (None, error text)."""
    proc = subprocess.run(
        ["gh", "pr", "view", pr, "--json", FIELDS],
        text=True,
        capture_output=True,
    )
    if proc.returncode != 0:
        return None, (proc.stderr or proc.stdout or f"exit {proc.returncode}").strip()
    try:
        return json.loads(proc.stdout), ""
    except json.JSONDecodeError as exc:
        return None, f"unreadable gh output: {exc}"


def classification(entry: dict[str, Any]) -> tuple[str, str]:
    """(name, state) for one rollup entry — a CheckRun or a StatusContext."""
    name = str(entry.get("name") or entry.get("context") or "?")
    raw = entry.get("conclusion") or entry.get("state") or "PENDING"
    return name, str(raw).upper()


def survey(doc: dict[str, Any]) -> dict[str, Any]:
    checks = [
        classification(entry)
        for entry in (doc.get("statusCheckRollup") or [])
        if isinstance(entry, dict)
    ]
    return {
        "total": len(checks),
        "pending": [name for name, state in checks if state in PENDING],
        "failed": [
            name for name, state in checks if state not in PENDING and state not in PASSED
        ],
        "mergeable": str(doc.get("mergeable") or "UNKNOWN").upper(),
        "state": str(doc.get("state") or "").upper(),
    }


def names(items: list[str], limit: int = 6) -> str:
    shown = ", ".join(items[:limit])
    if len(items) > limit:
        shown += f" (+{len(items) - limit} more)"
    return shown


def wait(pr: str, timeout: int, interval: int) -> int:
    deadline = time.monotonic() + timeout
    errors = 0
    last_note: tuple[Any, ...] | None = None
    last_print = 0.0
    while True:
        doc, error = fetch(pr)
        if doc is None:
            errors += 1
            if errors >= MAX_GH_ERRORS:
                print(
                    f"error: gh pr view {pr} failed {errors} times in a row: {error}",
                    file=sys.stderr,
                )
                return 1
            print(f"  gh pr view failed ({error}) — retrying", file=sys.stderr)
        else:
            errors = 0
            state = survey(doc)
            if state["failed"]:
                print(
                    f"error: {len(state['failed'])} check(s) did not pass on PR #{pr}: "
                    f"{names(state['failed'])}",
                    file=sys.stderr,
                )
                return 1
            if state["mergeable"] == "CONFLICTING":
                print(
                    f"error: PR #{pr} conflicts with its base (mergeable=CONFLICTING)",
                    file=sys.stderr,
                )
                return 1
            if state["state"] not in {"", "OPEN"}:
                if state["state"] == "MERGED":
                    print(f"PR #{pr} is already merged — nothing to wait for")
                    return 0
                print(f"error: PR #{pr} is {state['state']}, not open", file=sys.stderr)
                return 1
            if state["total"] and not state["pending"] and state["mergeable"] == "MERGEABLE":
                print(f"  checks: {state['total']} passed, 0 pending; mergeable=MERGEABLE")
                print(f"PR #{pr} is ready to merge")
                return 0

            note = (state["total"], len(state["pending"]), state["mergeable"])
            now = time.monotonic()
            if note != last_note or now - last_print >= 60:
                print(
                    f"  checks: {state['total']} reported, {len(state['pending'])} pending; "
                    f"mergeable={state['mergeable']}"
                )
                last_note = note
                last_print = now

        if time.monotonic() >= deadline:
            if doc is None:
                print(f"error: PR #{pr} unreadable for {timeout}s: {error}", file=sys.stderr)
            else:
                print(
                    f"error: PR #{pr} checks did not finish within {timeout}s: "
                    f"{len(state['pending'])} pending of {state['total']} reported, "
                    f"mergeable={state['mergeable']}",
                    file=sys.stderr,
                )
            return 1
        time.sleep(interval)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pr", required=True)
    parser.add_argument(
        "--timeout", type=int, default=int(os.environ.get("DSB_PR_CHECKS_TIMEOUT_SEC", "3600"))
    )
    parser.add_argument(
        "--interval", type=int, default=int(os.environ.get("DSB_PR_CHECKS_INTERVAL_SEC", "10"))
    )
    args = parser.parse_args(argv)
    if args.timeout < 0 or args.interval < 1:
        print("error: --timeout must be >= 0 and --interval >= 1", file=sys.stderr)
        return 2
    return wait(args.pr, args.timeout, args.interval)


if __name__ == "__main__":
    sys.exit(main())
