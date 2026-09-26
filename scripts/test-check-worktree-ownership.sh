#!/usr/bin/env bash
# Hermetic test for scripts/check-worktree-ownership.sh and the sentences the
# rule rides on.
#
# Fixtures: a throwaway git repo with four linked worktrees plus a stub
# `orca` that answers `terminal list` from JSON files — an agent tab, a shell
# tab on dirty work (defect), a shell tab on clean work (ok), and no tabs on
# dirty work (defect). Real `git status` decides dirtiness. No Orca daemon,
# no network, no writes outside the temp dir.
#
# The sentence pins follow scripts/test-orca-tab-one-tab.sh: the live
# AGENTS.md and skills/worktree-dispatch/SKILL.md must keep the phrases, and
# a copy with any one removed must fail.
#
# Usage: ./scripts/test-check-worktree-ownership.sh
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CHECK="$ROOT/scripts/check-worktree-ownership.sh"

fail=0
ok() { echo "ok: $*"; }
bad() { echo "not ok: $*" >&2; fail=1; }

if [[ ! -x "$CHECK" ]]; then
  bad "scripts/check-worktree-ownership.sh is missing or not executable"
  exit 1
fi

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

# --- fixture repo: four linked worktrees -----------------------------------
GIT=(git -c user.email=fixture@example.com -c user.name=fixture -c commit.gpgsign=false)

"${GIT[@]}" -c init.defaultBranch=main init -q "$tmp/repo"
printf 'one\n' > "$tmp/repo/file.txt"
"${GIT[@]}" -C "$tmp/repo" add file.txt
"${GIT[@]}" -C "$tmp/repo" commit -qm base

for wt in wt-agent wt-shell-dirty wt-shell-clean wt-no-tabs; do
  "${GIT[@]}" -C "$tmp/repo" worktree add -q "$tmp/$wt" -b "$wt"
done

for wt in wt-agent wt-shell-dirty wt-no-tabs; do
  printf 'edited\n' >> "$tmp/$wt/file.txt"
done

# --- stub orca: terminal list answers from $FIXTURE_DIR/<dir>.json ----------
mkdir -p "$tmp/bin"
cat > "$tmp/bin/orca" <<'STUB'
#!/usr/bin/env bash
# Fixture orca. `terminal list --worktree path:<dir>` reads
# $FIXTURE_DIR/<basename>.json; exit 3 when the fixture is absent, which the
# check must read as an unreadable terminal list.
set -u
wt=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --worktree) wt="${2:-}"; shift 2 ;;
    *) shift ;;
  esac
done
base="${wt##*/}"
if [[ -n "${FIXTURE_DIR:-}" && -f "$FIXTURE_DIR/$base.json" ]]; then
  exec cat "$FIXTURE_DIR/$base.json"
fi
echo '{"ok":false,"error":{"code":"not_found","message":"fixture: no terminals"}}'
exit 3
STUB
chmod +x "$tmp/bin/orca"

mkdir -p "$tmp/fx"
cat > "$tmp/fx/wt-agent.json" <<'JSON'
{"ok":true,"result":{"terminals":[{"handle":"term_fixture_agent","title":"Read /tmp/brief.md and do what it says… - DeepSeek Build","preview":"Responding… 12s"}]}}
JSON
shell_json() {
  cat <<JSON
{"ok":true,"result":{"terminals":[{"handle":"term_fixture_shell","title":"$1","preview":"$1 wt-shell *1 ❯"}]}}
JSON
}
shell_json "$tmp/wt-shell-dirty" > "$tmp/fx/wt-shell-dirty.json"
shell_json "$tmp/wt-shell-clean" > "$tmp/fx/wt-shell-clean.json"
printf '{"ok":true,"result":{"terminals":[]}}' > "$tmp/fx/wt-no-tabs.json"

# --- run 1: the four fixture shapes ----------------------------------------
set +e
ORCA_BIN="$tmp/bin/orca" FIXTURE_DIR="$tmp/fx" \
  "$CHECK" --root "$tmp/repo" --json > "$tmp/run1.json" 2>"$tmp/run1.err"
rc1=$?
set -e

if [[ "$rc1" -eq 1 ]]; then
  ok "run 1 exits 1 with the two defects"
else
  bad "run 1 exit code: expected 1, got $rc1"
  cat "$tmp/run1.err" >&2
fi

if python3 - "$tmp/run1.json" <<'PY'
import json, sys

doc = json.load(open(sys.argv[1]))
wts = {w["path"].rsplit("/", 1)[-1]: w for w in doc["worktrees"]}


def need(cond, msg):
    if not cond:
        sys.stderr.write(msg + "\n")
        sys.exit(1)


need(doc["ok"] is False, "report should be not ok")
need(doc["defects"] == 2, "expected 2 defects, got %r" % doc["defects"])

need(wts["wt-agent"]["verdict"] == "ok", "agent tab + dirty should be ok")
need(wts["wt-agent"]["dirtyFiles"] == 1, "agent worktree should read 1 dirty file")
need(wts["wt-agent"]["tabs"][0]["class"] == "agent", "agent tab should classify as agent")

need(wts["wt-shell-dirty"]["verdict"] == "defect", "shell tab + dirty should be a defect")
need(wts["wt-shell-dirty"]["dirtyFiles"] == 1, "shell-dirty should read 1 dirty file")
need(wts["wt-shell-dirty"]["tabs"][0]["class"] == "shell", "shell tab should classify as shell")
need("no agent tab" in wts["wt-shell-dirty"]["reason"], "defect reason should name the missing agent tab")

need(wts["wt-shell-clean"]["verdict"] == "ok", "shell tab + clean should be ok")
need(wts["wt-shell-clean"]["dirtyFiles"] == 0, "shell-clean should read 0 dirty files")

need(wts["wt-no-tabs"]["verdict"] == "defect", "no tabs + dirty should be a defect")
need(wts["wt-no-tabs"]["tabs"] == [], "no-tabs fixture should have no tabs")

need(wts["repo"]["verdict"] == "ok", "clean primary with no tabs should be ok")
PY
then
  ok "run 1 verdicts: agent ok, shell+dirty defect, shell+clean ok, no tabs defect"
else
  bad "run 1 verdicts did not match the fixture shapes"
fi

# --- run 2: every dirty worktree has an agent tab -> exit 0 ----------------
mkdir -p "$tmp/fx-agents"
for wt in wt-agent wt-shell-dirty wt-shell-clean wt-no-tabs; do
  cp "$tmp/fx/wt-agent.json" "$tmp/fx-agents/$wt.json"
done

set +e
ORCA_BIN="$tmp/bin/orca" FIXTURE_DIR="$tmp/fx-agents" \
  "$CHECK" --root "$tmp/repo" --json > "$tmp/run2.json" 2>"$tmp/run2.err"
rc2=$?
set -e

if [[ "$rc2" -eq 0 ]]; then
  ok "run 2 exits 0 when every dirty worktree has an agent tab"
else
  bad "run 2 exit code: expected 0, got $rc2"
  cat "$tmp/run2.err" >&2
fi

if python3 - "$tmp/run2.json" <<'PY'
import json, sys

doc = json.load(open(sys.argv[1]))
sys.exit(0 if doc["ok"] and doc["defects"] == 0 else 1)
PY
then
  ok "run 2 report is ok with no defects"
else
  bad "run 2 report still names defects"
fi

# --- run 3: dirty worktree whose terminal list is unreadable = defect ------
mkdir -p "$tmp/fx-partial"
for wt in wt-agent wt-shell-dirty wt-shell-clean; do
  cp "$tmp/fx/wt-agent.json" "$tmp/fx-partial/$wt.json"
done

set +e
ORCA_BIN="$tmp/bin/orca" FIXTURE_DIR="$tmp/fx-partial" \
  "$CHECK" --root "$tmp/repo" --json > "$tmp/run3.json" 2>"$tmp/run3.err"
rc3=$?
set -e

if python3 - "$tmp/run3.json" <<'PY'
import json, sys

doc = json.load(open(sys.argv[1]))
wts = {w["path"].rsplit("/", 1)[-1]: w for w in doc["worktrees"]}
no_tabs = wts["wt-no-tabs"]
sys.exit(0 if (no_tabs["verdict"] == "defect"
               and no_tabs["tabsRead"] is False
               and "unreadable" in no_tabs["reason"]) else 1)
PY
then
  ok "run 3: an unreadable terminal list on dirty work is a defect"
else
  bad "run 3: unreadable terminal list did not read as a defect"
fi
if [[ "$rc3" -eq 1 ]]; then
  ok "run 3 exits 1"
else
  bad "run 3 exit code: expected 1, got $rc3"
fi

# --- sentence pins ---------------------------------------------------------
AGENTS_REL="AGENTS.md"
DISPATCH_REL="skills/worktree-dispatch/SKILL.md"

AGENTS_NEEDLES=(
  'created together with its owning session'
  'carries the unit end to end'
  'the empty-card defect'
)
DISPATCH_NEEDLES=(
  'Handoff when the session that creates the worktree already runs elsewhere'
  'Hand over the in-flight state in the brief'
  'Stop driving that path'
)

collapsed() { tr '\n' ' ' < "$1" | tr -s '[:space:]' ' '; }

# One line per missing needle in <root>/<rel>; empty output means it holds.
check_file() {
  local root="$1" rel="$2"; shift 2
  local file="$root/$rel" text needle
  if [[ ! -f "$file" ]]; then
    echo "$rel is missing"
    return 0
  fi
  text="$(collapsed "$file")"
  for needle in "$@"; do
    [[ "$text" == *"$needle"* ]] || echo "$rel no longer says: $needle"
  done
}

pin_live() {
  local rel="$1"; shift
  local out
  out="$(check_file "$ROOT" "$rel" "$@")"
  if [[ -z "$out" ]]; then
    ok "live $rel keeps all $# pinned phrases"
  else
    bad "live $rel"
    printf '%s\n' "$out" >&2
  fi
}

# Each needle must be load-bearing: a copy with it removed has to fail.
# The strip is python, not `${text/"$needle"/}`: macOS bash 3.2 spends ~6.5 s
# per replacement on a file this size (measured 2026-09-26).
pin_stripped() {
  local rel="$1"; shift
  local live="$ROOT/$rel" needle out
  [[ -f "$live" ]] || return 0
  mkdir -p "$tmp/stripped/$(dirname "$rel")"
  collapsed "$live" > "$tmp/collapsed.txt"
  for needle in "$@"; do
    python3 - "$tmp/collapsed.txt" "$tmp/stripped/$rel" "$needle" <<'PY'
import sys

src, dst, needle = sys.argv[1], sys.argv[2], sys.argv[3]
open(dst, "w").write(open(src).read().replace(needle, ""))
PY
    out="$(check_file "$tmp/stripped" "$rel" "$needle")"
    if [[ -n "$out" ]]; then
      ok "a $rel copy without '$needle' fails"
    else
      bad "a $rel copy without '$needle' still passed"
    fi
  done
}

pin_live "$AGENTS_REL" "${AGENTS_NEEDLES[@]}"
pin_live "$DISPATCH_REL" "${DISPATCH_NEEDLES[@]}"
pin_stripped "$AGENTS_REL" "${AGENTS_NEEDLES[@]}"
pin_stripped "$DISPATCH_REL" "${DISPATCH_NEEDLES[@]}"

exit "$fail"
