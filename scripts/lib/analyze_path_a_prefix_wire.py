#!/usr/bin/env python3
"""Analyze Path A multi-turn wire for Spec 10 prefix stability (G008).

Reads a scripted-server JSONL wire (full `body.messages` or slim
`messages` form) and reports:

  - deepseek main-turn count
  - system message byte-equality across turns (L2-10-3)
  - skills index presence + stability (L1-70)
  - volatile growth (tool/assistant tail after fixed head)
  - wall-clock / timestamp tokens in system message (L2-10-4 negative)

Exit 0 when ≥2 deepseek turns share identical system + skills messages
and system has no wall-clock markers. Exit 1 otherwise.

Usage:
  python3 scripts/lib/analyze_path_a_prefix_wire.py docs/product/evidence/PATH_A_R0_LIVENESS_WIRE_last.jsonl
  python3 scripts/lib/analyze_path_a_prefix_wire.py /tmp/wire.jsonl --json
"""
from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from pathlib import Path
from typing import Any


def _sha(s: str) -> str:
    return hashlib.sha256(s.encode("utf-8")).hexdigest()


def _load_rows(path: Path) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    for line in path.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line:
            continue
        rows.append(json.loads(line))
    return rows


def _body(row: dict[str, Any]) -> dict[str, Any]:
    body = row.get("body", row)
    if isinstance(body, str):
        try:
            body = json.loads(body)
        except json.JSONDecodeError:
            return {}
    return body if isinstance(body, dict) else {}


def _messages(row: dict[str, Any]) -> list[dict[str, Any]]:
    body = _body(row)
    msgs = body.get("messages") or row.get("messages") or []
    return msgs if isinstance(msgs, list) else []


def _model(row: dict[str, Any]) -> str:
    body = _body(row)
    m = body.get("model") or row.get("model") or ""
    return m if isinstance(m, str) else ""


def _content(m: dict[str, Any]) -> str:
    c = m.get("content")
    if c is None:
        return ""
    if isinstance(c, str):
        return c
    return json.dumps(c, ensure_ascii=False)


def _find_msg(msgs: list[dict[str, Any]], pred) -> str | None:
    for m in msgs:
        if not isinstance(m, dict):
            continue
        c = _content(m)
        if pred(c):
            return c
    return None


WALL_CLOCK_PATTERNS = [
    re.compile(r"\bUtc::now\b", re.I),
    re.compile(r"\btimestamp\b", re.I),
    re.compile(r"\bSystemTime\b"),
    re.compile(r"\bunix[_-]?time\b", re.I),
    re.compile(r"\b\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}"),  # ISO datetime
]


def analyze(path: Path) -> dict[str, Any]:
    rows = _load_rows(path)
    deepseek = [r for r in rows if "deepseek" in _model(r).lower()]
    # Fall back: non-session-title turns with multi-message payloads
    if len(deepseek) < 2:
        deepseek = []
        for r in rows:
            msgs = _messages(r)
            if not msgs:
                continue
            sys0 = _content(msgs[0]) if msgs else ""
            if "session title" in sys0.lower():
                continue
            if "deepseek" in _model(r).lower() or _model(r).startswith("grok") is False:
                deepseek.append(r)
            elif len(msgs) >= 3:
                deepseek.append(r)

    report: dict[str, Any] = {
        "wire": str(path),
        "total_rows": len(rows),
        "deepseek_turns": len(deepseek),
        "system_stable": False,
        "skills_present": False,
        "skills_stable": False,
        "volatile_grows": False,
        "system_no_wall_clock": False,
        "system_sha256": None,
        "skills_sha256": None,
        "errors": [],
        "pass": False,
    }

    if len(deepseek) < 2:
        report["errors"].append(f"need ≥2 deepseek turns, got {len(deepseek)}")
        return report

    systems = []
    skills = []
    msg_counts = []
    for r in deepseek:
        msgs = _messages(r)
        msg_counts.append(len(msgs))
        if not msgs:
            report["errors"].append("empty messages on a deepseek turn")
            continue
        systems.append(_content(msgs[0]))
        sk = _find_msg(
            msgs,
            lambda c: "skills are available" in c.lower() or "## Skills index" in c,
        )
        skills.append(sk)

    report["system_stable"] = len(set(systems)) == 1 and bool(systems[0])
    report["system_sha256"] = _sha(systems[0]) if systems else None
    present = [s for s in skills if s]
    report["skills_present"] = len(present) == len(skills) and len(present) > 0
    report["skills_stable"] = (
        report["skills_present"] and len(set(present)) == 1
    )
    if present:
        report["skills_sha256"] = _sha(present[0])
    report["volatile_grows"] = msg_counts[-1] > msg_counts[0]

    sys0 = systems[0] if systems else ""
    wall_hits = [p.pattern for p in WALL_CLOCK_PATTERNS if p.search(sys0)]
    report["system_no_wall_clock"] = not wall_hits
    if wall_hits:
        report["errors"].append(f"wall-clock markers in system: {wall_hits}")

    # Note: user_info may contain "Today's date" — treated as volatile head,
    # not Spec 10 system stable-prefix (library assemble_path_a_context has no clock).
    if "Today's date" in sys0:
        report["errors"].append("Today's date found in system message (unexpected)")
        report["system_no_wall_clock"] = False

    ok = (
        report["system_stable"]
        and report["skills_present"]
        and report["skills_stable"]
        and report["system_no_wall_clock"]
        and len(deepseek) >= 2
    )
    report["pass"] = ok
    if not report["system_stable"]:
        report["errors"].append("system message not byte-stable across turns")
    if not report["skills_present"]:
        report["errors"].append("skills index missing on some turn")
    if report["skills_present"] and not report["skills_stable"]:
        report["errors"].append("skills index thrashing across turns")
    return report


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("wire", type=Path, help="Path to wire JSONL")
    ap.add_argument("--json", action="store_true", help="Print full JSON report")
    args = ap.parse_args()
    if not args.wire.is_file():
        print(f"FAIL: wire not found: {args.wire}", file=sys.stderr)
        return 1
    rep = analyze(args.wire)
    if args.json:
        print(json.dumps(rep, indent=2, ensure_ascii=False))
    else:
        status = "PASS" if rep["pass"] else "FAIL"
        print(f"analyze_path_a_prefix_wire: {status}")
        print(f"  wire={rep['wire']}")
        print(f"  deepseek_turns={rep['deepseek_turns']}")
        print(f"  system_stable={rep['system_stable']} sha={rep['system_sha256']}")
        print(
            f"  skills_present={rep['skills_present']} "
            f"skills_stable={rep['skills_stable']} sha={rep['skills_sha256']}"
        )
        print(f"  volatile_grows={rep['volatile_grows']}")
        print(f"  system_no_wall_clock={rep['system_no_wall_clock']}")
        for e in rep["errors"]:
            print(f"  error: {e}")
    return 0 if rep["pass"] else 1


if __name__ == "__main__":
    sys.exit(main())
