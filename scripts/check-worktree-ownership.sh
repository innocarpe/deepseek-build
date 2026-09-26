#!/usr/bin/env bash
# Read-only live check: every worktree of this repo that holds dirty work
# must have an agent tab inside it.
#
# The rule is AGENTS.md §Control-tower checkout ("A worktree is created
# together with its owning session") and skills/worktree-dispatch §1. The
# 2026-09-26 shape it catches: `vendor-suite-contract` was created from a
# tower tab, 28 files were modified in it by path, and its only Orca tab was
# a shell `❯` — the tower found that by hand with lsof/pgrep. This script
# reads the same state from `orca terminal list`.
#
# A tab's class is read from the terminal list's title/preview, and the
# evidence line is printed with the verdict:
#   agent    the title carries the TUI marker: " - DeepSeek Build", " - grok",
#            " - codex", " - claude"
#   shell    the title is a path, or the preview ends with a shell prompt `❯`
#   unknown  neither; reported as a non-agent tab, so a dirty worktree there
#            is a defect rather than an assumed session
#
# Exit: 0 every worktree ok, 1 at least one defect, 2 usage / no git / no orca.
#
# Usage: ./scripts/check-worktree-ownership.sh [--json] [--root <path>]
#   --json        one JSON document on stdout (same exit code)
#   --root <path> repo to read; default: the repo this script lives in
#   ORCA_BIN      orca binary to call; default: `orca`
set -euo pipefail

usage() {
  cat <<'EOF'
usage: ./scripts/check-worktree-ownership.sh [--json] [--root <path>]

  --json        one JSON document on stdout (same exit code)
  --root <path> repo to read; default: the repo this script lives in

Environment:
  ORCA_BIN      orca binary to call; default: `orca`

Read-only. Exit 0 all worktrees ok, 1 at least one defect, 2 usage.
EOF
}

json=0
root=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --json) json=1; shift ;;
    --root)
      [[ $# -ge 2 && -n "$2" ]] || { echo "check-worktree-ownership: --root needs a path" >&2; exit 2; }
      root="$2"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "check-worktree-ownership: unknown argument: $1" >&2; usage >&2; exit 2 ;;
  esac
done

if [[ -z "$root" ]]; then
  root="$(cd "$(dirname "$0")/.." && pwd)"
fi
orca="${ORCA_BIN:-orca}"

CWO_JSON="$json" CWO_ROOT="$root" CWO_ORCA="$orca" exec python3 - <<'PY'
import json
import os
import shutil
import subprocess
import sys

MODE_JSON = os.environ["CWO_JSON"] == "1"
ROOT = os.environ["CWO_ROOT"]
ORCA = os.environ["CWO_ORCA"]

# TUI title suffixes, lowercased. Matched as substrings of the title so a
# title truncated in the middle still classifies.
AGENT_MARKERS = (" - deepseek build", " - grok", " - codex", " - claude")
# Evidence keeps the tail: a shell prompt sits at the end of the screen.
PREVIEW_KEEP = 160


def run(args):
    return subprocess.run(args, capture_output=True, text=True, errors="replace")


def classify(title, preview):
    lower = (title or "").lower()
    if any(m in lower for m in AGENT_MARKERS):
        return "agent"
    if (title or "").lstrip().startswith(("~", "/")):
        return "shell"
    if (preview or "").rstrip().endswith("❯"):
        return "shell"
    return "unknown"


def read_tabs(path):
    """Terminal list for one worktree. None = unreadable."""
    r = run([ORCA, "terminal", "list", "--worktree", "path:" + path, "--json"])
    if r.returncode != 0:
        return None
    try:
        data = json.loads(r.stdout)
        if data.get("ok") is not True:
            return None
        terminals = data["result"]["terminals"]
        if not isinstance(terminals, list):
            return None
    except (ValueError, KeyError, TypeError):
        return None
    tabs = []
    for t in terminals:
        title = str(t.get("title") or "")
        preview = str(t.get("preview") or "")
        tabs.append({
            "handle": str(t.get("handle") or ""),
            "title": title,
            "preview": preview[-PREVIEW_KEEP:],
            "class": classify(title, preview),
        })
    return tabs


def list_worktrees():
    r = run(["git", "-C", ROOT, "worktree", "list", "--porcelain"])
    if r.returncode != 0:
        sys.stderr.write("check-worktree-ownership: not a git checkout: %s\n" % ROOT)
        sys.exit(2)
    entries = []
    cur = None
    for line in r.stdout.splitlines():
        if line.startswith("worktree "):
            cur = {"path": line[len("worktree "):], "branch": "(detached)"}
            entries.append(cur)
        elif cur is not None and line.startswith("branch refs/heads/"):
            cur["branch"] = line[len("branch refs/heads/"):]
    return entries


def dirty_count(path):
    r = run(["git", "-C", path, "status", "--porcelain"])
    if r.returncode != 0:
        return None
    return len([ln for ln in r.stdout.splitlines() if ln.strip()])


def main():
    if shutil.which(ORCA) is None:
        sys.stderr.write("check-worktree-ownership: orca not found (ORCA_BIN=%s)\n" % ORCA)
        return 2

    out = []
    defects = 0
    for wt in list_worktrees():
        path = wt["path"]
        rec = {
            "path": path,
            "branch": wt["branch"],
            "dirtyFiles": None,
            "tabsRead": False,
            "tabs": [],
            "verdict": "ok",
            "reason": "",
        }
        out.append(rec)

        if not os.path.isdir(path):
            rec["verdict"] = "skipped"
            rec["reason"] = "path missing (prunable registration)"
            continue

        dirty = dirty_count(path)
        rec["dirtyFiles"] = dirty
        if dirty is None:
            rec["verdict"] = "defect"
            rec["reason"] = "git status failed; dirty state unreadable"
            defects += 1
            continue

        tabs = read_tabs(path)
        if tabs is not None:
            rec["tabsRead"] = True
            rec["tabs"] = tabs
        agents = [t for t in (tabs or []) if t["class"] == "agent"]

        if dirty == 0:
            rec["reason"] = "clean"
        elif agents:
            rec["reason"] = "dirty (%d files), agent tab present" % dirty
        else:
            if not rec["tabsRead"]:
                tab_note = "terminal list unreadable"
            elif not rec["tabs"]:
                tab_note = "no tabs"
            else:
                kinds = {}
                for t in rec["tabs"]:
                    kinds[t["class"]] = kinds.get(t["class"], 0) + 1
                tab_note = "tabs: " + ", ".join(
                    "%d %s" % (n, k) for k, n in sorted(kinds.items()))
            rec["verdict"] = "defect"
            rec["reason"] = "dirty (%d files), no agent tab (%s)" % (dirty, tab_note)
            defects += 1

    doc = {"root": ROOT, "ok": defects == 0, "defects": defects, "worktrees": out}
    if MODE_JSON:
        print(json.dumps(doc, ensure_ascii=False, indent=2))
    else:
        print("check-worktree-ownership: %s" % ROOT)
        for rec in out:
            print("%-7s %s  [%s]  dirty=%s  %s" % (
                rec["verdict"], rec["path"], rec["branch"],
                "?" if rec["dirtyFiles"] is None else rec["dirtyFiles"],
                rec["reason"]))
            for t in rec["tabs"]:
                print("        tab %s  %s  title=%r  preview=%r" % (
                    t["handle"], t["class"], t["title"], t["preview"]))
        n_ok = sum(1 for r in out if r["verdict"] == "ok")
        n_skip = sum(1 for r in out if r["verdict"] == "skipped")
        line = "%d ok, %d defect" % (n_ok, defects)
        if n_skip:
            line += ", %d skipped" % n_skip
        print("check-worktree-ownership: %s (%d worktrees)" % (line, len(out)))
    return 1 if defects else 0


sys.exit(main())
PY
