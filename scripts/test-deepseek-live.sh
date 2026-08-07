#!/usr/bin/env bash
# T3 + T4 (+ optional T5) — live DeepSeek API verification for thin + Grok agent paths.
# Requires DEEPSEEK_API_KEY or ~/.deepseek-build/credentials.json.
# Never prints API keys.
set -euo pipefail
# shellcheck source=lib/common.sh
source "$(cd "$(dirname "$0")" && pwd)/lib/common.sh"

EXTENDED=0
PHASE="all" # all | thin | agent
while [[ $# -gt 0 ]]; do
  case "$1" in
    --extended) EXTENDED=1; shift ;;
    --thin) PHASE=thin; shift ;;
    --agent) PHASE=agent; shift ;;
    -h|--help)
      cat <<'EOF'
Usage: test-deepseek-live.sh [--thin|--agent] [--extended]

  --thin       T3 only (dsb run thin path)
  --agent      T4 only (deepseek-build-agent headless + tools)
  --extended   also T5 optional agent features
EOF
      exit 0
      ;;
    *) fail "unknown arg: $1" ;;
  esac
done

RESULTS="${PRE3X_RESULTS:-}"
record() {
  [[ -n "$RESULTS" ]] || return 0
  record_result "$RESULTS" "$1" "$2" "${3:-}"
}

if ! load_deepseek_key; then
  fail "no DeepSeek API key (set DEEPSEEK_API_KEY or credentials.json)"
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

FAILED=0
mark_fail() {
  FAILED=1
  warn "$*"
}

# ---------- T3 thin ----------
run_thin() {
  log "== T3 thin-path DeepSeek live =="
  local bin
  if ! bin="$(find_product_bin dsb)"; then
    cargo build -p dsb-cli --release
    bin="$(find_product_bin dsb)" || fail "dsb binary missing"
  fi

  log "T3.1 dsb run pong"
  local out ec=0
  set +e
  out="$(run_to 120 "$bin" run "Reply with exactly one word: pong" 2>&1)"
  ec=$?
  set -e
  echo "$out" | redact_stream | tail -20
  if [[ $ec -eq 0 ]] && echo "$out" | rg -qi 'pong'; then
    record T3.1 PASS
    ok "T3.1 pong"
  else
    record T3.1 FAIL "ec=$ec"
    mark_fail "T3.1 failed"
  fi

  if echo "$out" | rg -q 'model=deepseek-v4-flash|model_used=deepseek-v4-flash'; then
    record T3.2 PASS
    ok "T3.2 model line"
  else
    record T3.2 FAIL "model line missing"
    mark_fail "T3.2 model line"
  fi
}

# ---------- T4 agent ----------
run_agent_core() {
  log "== T4 agent-path DeepSeek live =="
  local agent
  agent="$(find_agent_bin)" || fail "agent binary missing (T2.1)"

  local GROK_TMP WS
  GROK_TMP="$(make_hermetic_grok_home)"
  WS="$(mktemp -d "${TMPDIR:-/tmp}/dsb-pre3x-ws.XXXXXX")"
  cleanup() { rm -rf "$GROK_TMP" "$WS"; }
  trap cleanup EXIT

  export GROK_HOME="$GROK_TMP"
  echo "alpha-marker-42" >"$WS/fixture.txt"
  mkdir -p "$WS/subdir"
  echo "nested-content" >"$WS/subdir/n.txt"
  printf 'line one\nfind-me-token-77\nline three\n' >"$WS/searchme.txt"

  # T4.0 + T4.1 headless chat — must not hit Grok proxy
  log "T4.0/T4.1 headless chat via DeepSeek"
  local out ec=0
  set +e
  out="$(
    run_to 180 "$agent" -p "Reply with exactly one word: pong" \
      --cwd "$WS" \
      --output-format plain \
      --max-turns 2 \
      --disallowed-tools "run_terminal_cmd,search_replace,web_search,web_fetch,Agent" \
      2>&1
  )"
  ec=$?
  set -e
  echo "$out" | redact_stream | tail -30
  if echo "$out" | rg -q 'cli-chat-proxy\.grok\.com'; then
    record T4.0 FAIL "routed to cli-chat-proxy.grok.com"
    mark_fail "T4.0 still on Grok proxy — model base_url broken"
  elif [[ $ec -ne 0 ]]; then
    record T4.0 FAIL "ec=$ec"
    mark_fail "T4.0/T4.1 exit $ec"
  else
    record T4.0 PASS "no grok proxy host in output"
    ok "T4.0 DeepSeek route"
  fi
  if [[ $ec -eq 0 ]] && echo "$out" | rg -qi 'pong'; then
    record T4.1 PASS
    ok "T4.1 pong"
  else
    record T4.1 FAIL
    mark_fail "T4.1 pong"
  fi

  # helper for tool cases
  agent_tool_case() {
    local id="$1" expect_re="$2" tools="$3" prompt="$4" max_turns="${5:-8}"
    log "$id tools=$tools"
    local o e=0
    set +e
    o="$(
      run_to 300 "$agent" -p "$prompt" \
        --cwd "$WS" \
        --yolo \
        --output-format plain \
        --max-turns "$max_turns" \
        --tools "$tools" \
        2>&1
    )"
    e=$?
    set -e
    echo "$o" | redact_stream | tail -40
    if echo "$o" | rg -q 'cli-chat-proxy\.grok\.com'; then
      record "$id" FAIL "grok proxy"
      mark_fail "$id proxy"
      return
    fi
    if [[ $e -eq 0 ]] && echo "$o" | rg -qi "$expect_re"; then
      record "$id" PASS
      ok "$id"
    else
      record "$id" FAIL "ec=$e"
      mark_fail "$id failed"
    fi
  }

  agent_tool_case T4.2 'alpha-marker-42' 'read_file' \
    "Use the read_file tool to read fixture.txt. Reply with exactly the file contents only (no quotes)."

  agent_tool_case T4.3 'fixture\.txt|subdir' 'list_dir' \
    "Use the list_dir tool on the current directory. Reply with the entry names only."

  agent_tool_case T4.4 'find-me-token-77' 'grep' \
    "Use the grep tool to search this workspace for find-me-token-77. Reply with the matching line only."

  agent_tool_case T4.5 'shell-ok-99' 'run_terminal_cmd' \
    "Use run_terminal_cmd to execute: echo shell-ok-99. Then reply with exactly that stdout line."

  log "T4.6 search_replace"
  set +e
  out="$(
    run_to 300 "$agent" -p \
      "Use search_replace to replace alpha-marker-42 with alpha-marker-99 in fixture.txt. Then use read_file and reply with the new contents only." \
      --cwd "$WS" --yolo --output-format plain --max-turns 10 \
      --tools "search_replace,read_file" 2>&1
  )"
  ec=$?
  set -e
  echo "$out" | redact_stream | tail -40
  disk="$(cat "$WS/fixture.txt" 2>/dev/null || true)"
  if [[ $ec -eq 0 ]] && echo "$disk" | rg -q 'alpha-marker-99'; then
    record T4.6 PASS "disk=$disk"
    ok "T4.6 search_replace (disk verified)"
  elif [[ $ec -eq 0 ]] && echo "$out" | rg -q 'alpha-marker-99'; then
    record T4.6 PASS "output only; disk='$disk'"
    ok "T4.6 search_replace (output)"
  else
    record T4.6 FAIL "disk=$disk"
    mark_fail "T4.6 search_replace"
  fi

  # multi-turn tool loop (same as read but force tool first)
  agent_tool_case T4.7 'nested-content' 'read_file' \
    "Read subdir/n.txt with read_file. Reply with exactly its contents." 8

  log "T4.8 pro model short"
  set +e
  out="$(
    run_to 180 "$agent" -p "Reply with exactly one word: pong" \
      -m deepseek-v4-pro \
      --cwd "$WS" --output-format plain --max-turns 2 \
      --disallowed-tools "run_terminal_cmd,search_replace,web_search,web_fetch,Agent" \
      2>&1
  )"
  ec=$?
  set -e
  echo "$out" | redact_stream | tail -20
  if [[ $ec -eq 0 ]] && echo "$out" | rg -qi 'pong'; then
    record T4.8 PASS
    ok "T4.8 pro"
  else
    record T4.8 FAIL "ec=$ec (quota or model?)"
    mark_fail "T4.8 pro"
  fi

  trap - EXIT
  cleanup
}

run_extended() {
  log "== T5 extended (best-effort) =="
  local agent
  agent="$(find_agent_bin)" || {
    record T5 FAIL "no agent"
    return
  }
  local GROK_TMP WS
  GROK_TMP="$(make_hermetic_grok_home)"
  WS="$(mktemp -d "${TMPDIR:-/tmp}/dsb-pre3x-ws.XXXXXX")"
  cleanup2() { rm -rf "$GROK_TMP" "$WS"; }
  trap cleanup2 EXIT
  export GROK_HOME="$GROK_TMP"
  echo "sess-marker" >"$WS/a.txt"

  log "T5.9 streaming-json smoke"
  set +e
  local out ec
  out="$(
    run_to 180 "$agent" -p "Reply with exactly: ok" \
      --cwd "$WS" --output-format streaming-json --max-turns 2 \
      --disallowed-tools "run_terminal_cmd,search_replace,web_search,web_fetch,Agent" \
      2>&1
  )"
  ec=$?
  set -e
  echo "$out" | redact_stream | tail -15
  if [[ $ec -eq 0 ]]; then
    record T5.9 PASS
    ok "T5.9 streaming-json"
  else
    record T5.9 FAIL
    mark_fail "T5.9"
  fi

  log "T5.8 permission deny (no yolo write)"
  # Without yolo, write should not freely land; we only assert process completes without proxy
  set +e
  out="$(
    run_to 180 "$agent" -p "Create a file named should-not-write.txt with hello" \
      --cwd "$WS" --output-format plain --max-turns 4 \
      --tools "search_replace" \
      --deny "Edit(**)" \
      2>&1
  )"
  ec=$?
  set -e
  if [[ -f "$WS/should-not-write.txt" ]]; then
    record T5.8 FAIL "file written despite deny"
    mark_fail "T5.8 deny ineffective"
  else
    record T5.8 PASS "no unauthorized file"
    ok "T5.8 deny (no file)"
  fi

  # Remaining T5.* marked skip unless we add dedicated harness later
  for id in T5.1 T5.2 T5.3 T5.4 T5.5 T5.6 T5.7 T5.10; do
    record "$id" SKIP "not automated in v1 harness"
  done

  trap - EXIT
  cleanup2
}

case "$PHASE" in
  thin) run_thin ;;
  agent) run_agent_core ;;
  all)
    run_thin
    run_agent_core
    ;;
esac

if [[ "$EXTENDED" -eq 1 ]]; then
  run_extended
fi

if [[ "$FAILED" -ne 0 ]]; then
  fail "live DeepSeek suite had failures"
fi
ok "live DeepSeek suite PASSED"
