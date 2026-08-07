#!/usr/bin/env bash
# L3 capability smoke against the installed DeepSeek agent (prep for 4.0.0).
#
# Does NOT change product defaults. Does NOT require vendor cargo test target/.
# Uses hermetic GROK_HOME + temp workspace. Safe to run while heart-3x develops.
#
# Usage:
#   ./scripts/test-l3-smoke.sh           # core probes
#   ./scripts/test-l3-smoke.sh --extended  # + subagent (costly / slower)
#
# See: docs/product/PARALLEL_3X_4X_PLAN.md (Lane B)
set -euo pipefail
# shellcheck source=lib/common.sh
source "$(cd "$(dirname "$0")" && pwd)/lib/common.sh"

EXTENDED=0
OFFLINE_ONLY=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --extended) EXTENDED=1; shift ;;
    --offline-only) OFFLINE_ONLY=1; shift ;;
    -h|--help)
      sed -n '2,16p' "$0"
      exit 0
      ;;
    *) fail "unknown arg: $1" ;;
  esac
done

RESULTS="${L3_RESULTS:-$ROOT/docs/product/evidence/_last_l3_smoke.tsv}"
mkdir -p "$(dirname "$RESULTS")"
: >"$RESULTS"
record() { record_result "$RESULTS" "$1" "$2" "${3:-}"; }

HAVE_KEY=0
if load_deepseek_key; then
  HAVE_KEY=1
elif [[ "$OFFLINE_ONLY" -eq 1 ]]; then
  warn "no API key; offline CLI probes only"
else
  warn "no DEEPSEEK_API_KEY/credentials.json — running offline CLI probes; pass key for live L3.1+"
  OFFLINE_ONLY=1
fi

TIMEOUT_BIN=""
if command -v timeout >/dev/null 2>&1; then
  TIMEOUT_BIN="timeout"
elif command -v gtimeout >/dev/null 2>&1; then
  TIMEOUT_BIN="gtimeout"
fi
run_to() {
  local secs="$1"
  shift
  if [[ -n "$TIMEOUT_BIN" ]]; then
    "$TIMEOUT_BIN" "$secs" "$@"
  else
    "$@"
  fi
}

AGENT="$(find_agent_bin)" || fail "agent binary missing (install deepseek-build-agent)"
log "agent=$AGENT"

GROK_TMP="$(make_hermetic_grok_home)"
WS="$(mktemp -d "${TMPDIR:-/tmp}/dsb-l3-ws.XXXXXX")"
cleanup() { rm -rf "$GROK_TMP" "$WS"; }
trap cleanup EXIT
export GROK_HOME="$GROK_TMP"

echo "l3-marker-ready" >"$WS/marker.txt"
mkdir -p "$WS/sub"

FAILED=0
mark_fail() { FAILED=1; warn "$*"; }

# --- L3.0 CLI surface (offline) ---
log "L3.0 agent --help lists worktree / no-subagents"
HELP_OUT="$("$AGENT" --help 2>&1 || true)"
if echo "$HELP_OUT" | rg -q 'worktree' && echo "$HELP_OUT" | rg -q 'no-subagents'; then
  record L3.0 PASS "help flags present"
  ok "L3.0 CLI flags"
else
  record L3.0 FAIL "missing worktree or no-subagents in help"
  mark_fail "L3.0"
fi

# --- L3.4 worktree CLI help (offline; always) ---
log "L3.4 worktree subcommand help"
set +e
WT_HELP="$("$AGENT" worktree --help 2>&1)"
EC=$?
set -e
if [[ $EC -eq 0 ]] || echo "$WT_HELP" | rg -qi 'worktree|Usage'; then
  record L3.4 PASS
  ok "L3.4 worktree help"
else
  record L3.4 FAIL
  mark_fail "L3.4"
fi

if [[ "$OFFLINE_ONLY" -eq 1 ]]; then
  for id in L3.1 L3.2 L3.3 L3.5; do
    record "$id" SKIP "no API key / --offline-only"
  done
  skip "live L3.1–L3.3/L3.5 skipped (restore credentials.json or DEEPSEEK_API_KEY)"
else
  # --- L3.1 headless + DeepSeek route ---
  log "L3.1 headless text via DeepSeek"
  set +e
  OUT="$(
    run_to 180 "$AGENT" -p "Reply with exactly one word: pong" \
      --cwd "$WS" --output-format plain --max-turns 2 \
      --disallowed-tools "run_terminal_cmd,search_replace,web_search,web_fetch,Agent,spawn_subagent" \
      2>&1
  )"
  EC=$?
  set -e
  echo "$OUT" | redact_stream | tail -15
  if echo "$OUT" | rg -q 'cli-chat-proxy\.grok\.com'; then
    record L3.1 FAIL "routed to grok proxy"
    mark_fail "L3.1 proxy"
  elif [[ $EC -eq 0 ]] && echo "$OUT" | rg -qi 'pong'; then
    record L3.1 PASS
    ok "L3.1"
  else
    record L3.1 FAIL "ec=$EC"
    mark_fail "L3.1"
  fi

  # --- L3.2 background shell ---
  log "L3.2 background run_terminal_cmd + get output"
  set +e
  OUT="$(
    run_to 300 "$AGENT" -p \
      "Use run_terminal_command with background true to run: sleep 1; echo bg-ok-77. Then use get_command_or_subagent_output (or get_task_output) to wait for it. Reply with exactly the stdout line bg-ok-77 when done." \
      --cwd "$WS" --yolo --output-format plain --max-turns 12 \
      --tools "run_terminal_cmd,get_command_or_subagent_output,get_task_output,kill_command_or_subagent" \
      2>&1
  )"
  EC=$?
  set -e
  echo "$OUT" | redact_stream | tail -40
  if [[ $EC -eq 0 ]] && echo "$OUT" | rg -q 'bg-ok-77'; then
    record L3.2 PASS
    ok "L3.2 background shell"
  else
    record L3.2 FAIL "ec=$EC (model may not have used background=true)"
    mark_fail "L3.2"
  fi

  # --- L3.3 --no-subagents ---
  log "L3.3 --no-subagents headless"
  set +e
  OUT="$(
    run_to 180 "$AGENT" -p "Reply with exactly: nosub" \
      --cwd "$WS" --output-format plain --max-turns 2 \
      --no-subagents \
      --disallowed-tools "run_terminal_cmd,search_replace,web_search,web_fetch" \
      2>&1
  )"
  EC=$?
  set -e
  echo "$OUT" | redact_stream | tail -15
  if [[ $EC -eq 0 ]] && echo "$OUT" | rg -qi 'nosub'; then
    record L3.3 PASS
    ok "L3.3 no-subagents"
  else
    record L3.3 FAIL "ec=$EC"
    mark_fail "L3.3"
  fi

  if [[ "$EXTENDED" -eq 1 ]]; then
    log "L3.5 spawn_subagent explore (extended)"
    set +e
    OUT="$(
      run_to 420 "$AGENT" -p \
        "Spawn an explore subagent (spawn_subagent) to list files in the current directory using list_dir only. When it returns, reply with the word: delegated." \
        --cwd "$WS" --yolo --output-format plain --max-turns 16 \
        --tools "spawn_subagent,list_dir,read_file,grep" \
        2>&1
    )"
    EC=$?
    set -e
    echo "$OUT" | redact_stream | tail -50
    if [[ $EC -eq 0 ]] && echo "$OUT" | rg -qi 'delegated'; then
      record L3.5 PASS
      ok "L3.5 subagent"
    else
      record L3.5 FAIL "ec=$EC"
      mark_fail "L3.5"
    fi
  else
    record L3.5 SKIP "pass --extended"
    skip "L3.5 subagent (use --extended)"
  fi
fi

log "== L3 smoke summary ($RESULTS) =="
if [[ -s "$RESULTS" ]]; then
  column -t -s $'\t' "$RESULTS" 2>/dev/null || cat "$RESULTS"
fi

if [[ "$FAILED" -ne 0 ]]; then
  fail "L3 smoke had failures (see $RESULTS)"
fi
ok "L3 smoke ALL PASSED"
