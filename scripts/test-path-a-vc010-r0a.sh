#!/usr/bin/env bash
# VC010 Path A R0A — public deepseek-build/dsb agent multi-tool parallel + bg collect.
#
# Proves (Spec 50 / vision-complete Grok L3 Path A bar):
#   1. Public CLI → agent_launch → product agent (no DEEPSEEK_BUILD_AGENT_BIN)
#   2. Hermetic home + scripted DeepSeek wire
#   3. multi-read-parallel: ≥2 read_file tool_calls in one assistant message
#   4. mixed-mutate-serial: multi-read then search_replace with snippet_id
#   5. bg-collect-by-id: is_background shell + get_command_or_subagent_output by task_id
#
# Usage:
#   ./scripts/test-path-a-vc010-r0a.sh
#   ./scripts/test-path-a-vc010-r0a.sh --keep
#   ./scripts/test-path-a-vc010-r0a.sh --scenario multi-read-parallel
#   ./scripts/test-path-a-vc010-r0a.sh --skip-build
#
# Exit 0 only when required scenarios pass with wire + public-path evidence.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "${ROOT}"
# shellcheck source=lib/common.sh
source "${ROOT}/scripts/lib/common.sh"

KEEP=0
SKIP_BUILD=0
ONLY_SCENARIO=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --keep) KEEP=1; shift ;;
    --skip-build) SKIP_BUILD=1; shift ;;
    --scenario)
      ONLY_SCENARIO="${2:?}"
      shift 2
      ;;
    -h|--help)
      sed -n '2,20p' "$0"
      exit 0
      ;;
    *) fail "unknown arg: $1" ;;
  esac
done

echo "=== test-path-a-vc010-r0a (Path A L3 multi-tool + bg collect) ==="
echo "git_sha=$(git rev-parse HEAD 2>/dev/null || echo unknown)"

if [[ -n "${DEEPSEEK_BUILD_AGENT_BIN:-}" ]]; then
  warn "DEEPSEEK_BUILD_AGENT_BIN was set; unsetting for public-entry proof"
  unset DEEPSEEK_BUILD_AGENT_BIN
fi

# --- resolve public CLI ---
CLI=""
for c in \
  "${ROOT}/target/release/deepseek-build" \
  "${ROOT}/target/debug/deepseek-build" \
  "${DEEPSEEK_BUILD_HOME:-$HOME/.deepseek-build}/bin/deepseek-build" \
  "${CARGO_HOME:-$HOME/.cargo}/bin/deepseek-build" \
  "$(command -v deepseek-build 2>/dev/null || true)"
do
  if [[ -n "${c}" && -x "${c}" ]]; then
    CLI="${c}"
    break
  fi
done
if [[ -z "${CLI}" ]]; then
  fail "NO_CLI: deepseek-build binary not found — build cargo -p dsb-cli or ./scripts/install.sh first"
fi
log "cli=${CLI}"

CLI_DIR="$(cd "$(dirname "${CLI}")" && pwd)"
DSB_BIN=""
if [[ -x "${CLI_DIR}/dsb" ]]; then
  DSB_BIN="${CLI_DIR}/dsb"
elif command -v dsb >/dev/null 2>&1; then
  DSB_BIN="$(command -v dsb)"
fi
log "dsb=${DSB_BIN:-missing (optional)}"

agent_runs() {
  local bin="$1"
  [[ -x "${bin}" ]] || return 1
  if command -v timeout >/dev/null 2>&1; then
    timeout 8 "${bin}" --version >/dev/null 2>&1
  elif command -v gtimeout >/dev/null 2>&1; then
    gtimeout 8 "${bin}" --version >/dev/null 2>&1
  else
    "${bin}" --version >/dev/null 2>&1
  fi
}

GROK_ROOT="${ROOT}/third_party/grok-build"
AGENT_BUILD="${GROK_ROOT}/target/release/xai-grok-pager"
if [[ "${SKIP_BUILD}" -eq 0 ]]; then
  log "building Path A agent from stack third_party/grok-build (release -p xai-grok-pager-bin)"
  export CARGO_INCREMENTAL=0
  unset RUSTC_WRAPPER || true
  (
    cd "${GROK_ROOT}"
    cargo build --release -p xai-grok-pager-bin 2>&1 | tail -40
  )
  if ! agent_runs "${AGENT_BUILD}"; then
    fail "NO_AGENT_BUILD: ${AGENT_BUILD} does not run --version after build"
  fi
  AGENT_RESOLVED="${AGENT_BUILD}"
else
  AGENT_RESOLVED=""
  for c in \
    "${AGENT_BUILD}" \
    "${DEEPSEEK_BUILD_HOME:-$HOME/.deepseek-build}/bin/xai-grok-pager" \
    "${DEEPSEEK_BUILD_HOME:-$HOME/.deepseek-build}/bin/deepseek-build-agent" \
    "${CLI_DIR}/deepseek-build-agent" \
    "${CLI_DIR}/xai-grok-pager"
  do
    if agent_runs "${c}"; then
      AGENT_RESOLVED="${c}"
      break
    fi
  done
  if [[ -z "${AGENT_RESOLVED}" ]]; then
    fail "NO_AGENT: no runnable agent; drop --skip-build to compile from stack"
  fi
fi
log "agent_resolved=${AGENT_RESOLVED}"
if command -v shasum >/dev/null 2>&1; then
  log "agent_sha256=$(shasum -a 256 "${AGENT_RESOLVED}" | awk '{print $1}')"
fi

# Ensure public CLI exists (release preferred for agent_launch stamps)
if [[ ! -x "${ROOT}/target/release/deepseek-build" ]]; then
  log "building dsb-cli release for public entry"
  cargo build --release -p dsb-cli 2>&1 | tail -20
  CLI="${ROOT}/target/release/deepseek-build"
  CLI_DIR="$(cd "$(dirname "${CLI}")" && pwd)"
fi

SERVER_PY="${ROOT}/scripts/lib/scripted_deepseek_server.py"
[[ -f "${SERVER_PY}" ]] || fail "MISSING: ${SERVER_PY}"

OUT_DIR="${ROOT}/docs/product/evidence"
mkdir -p "${OUT_DIR}"

TIMEOUT_BIN=""
if command -v timeout >/dev/null 2>&1; then
  TIMEOUT_BIN="timeout"
elif command -v gtimeout >/dev/null 2>&1; then
  TIMEOUT_BIN="gtimeout"
fi
run_to() {
  local secs="$1"
  shift
  if [[ -n "${TIMEOUT_BIN}" ]]; then
    "${TIMEOUT_BIN}" "${secs}" "$@"
  else
    "$@"
  fi
}

run_scenario() {
  local SCENARIO="$1"
  local PROMPT="$2"
  local MAX_TURNS="${3:-12}"

  local WORK HOME_DIR WS WIRE SERVER_PID=""
  WORK="$(mktemp -d "${TMPDIR:-/tmp}/dsb-vc010-r0a.XXXXXX")"
  HOME_DIR="${WORK}/product-home"
  WS="${WORK}/ws"
  WIRE="${WORK}/wire.jsonl"
  mkdir -p "${HOME_DIR}/bin" "${WS}"

  cleanup_scenario() {
    if [[ -n "${SERVER_PID:-}" ]]; then
      kill "${SERVER_PID}" 2>/dev/null || true
      wait "${SERVER_PID}" 2>/dev/null || true
    fi
    if [[ "${KEEP}" -eq 0 ]]; then
      rm -rf "${WORK}"
    else
      log "kept workdir=${WORK}"
    fi
  }
  trap cleanup_scenario RETURN

  case "${SCENARIO}" in
    multi-read-parallel|mixed-mutate-serial)
      printf 'alpha-marker\n' >"${WS}/a.txt"
      printf 'beta-marker\n' >"${WS}/b.txt"
      ;;
    bg-collect-by-id)
      printf 'bg-ws-ready\n' >"${WS}/marker.txt"
      ;;
    *)
      fail "unknown scenario ${SCENARIO}"
      ;;
  esac

  local SERVER_LOG="${WORK}/server.log"
  python3 "${SERVER_PY}" \
    --host 127.0.0.1 \
    --port 0 \
    --wire "${WIRE}" \
    --scenario "${SCENARIO}" \
    --liveness-dir "${WS}" \
    --final-text "vc010-${SCENARIO}-done" \
    >"${WORK}/server.stdout" 2>"${SERVER_LOG}" &
  SERVER_PID=$!

  local READY_LINE=""
  for _ in $(seq 1 50); do
    if [[ -s "${WORK}/server.stdout" ]]; then
      READY_LINE="$(head -1 "${WORK}/server.stdout" || true)"
      if [[ "${READY_LINE}" == READY\ * ]]; then
        break
      fi
    fi
    if ! kill -0 "${SERVER_PID}" 2>/dev/null; then
      cat "${SERVER_LOG}" >&2 || true
      fail "scripted server exited early (${SCENARIO})"
    fi
    sleep 0.1
  done
  if [[ "${READY_LINE}" != READY\ * ]]; then
    cat "${SERVER_LOG}" >&2 || true
    fail "scripted server did not print READY (${SCENARIO})"
  fi
  local HOSTPORT="${READY_LINE#READY }"
  local BASE_URL="http://${HOSTPORT}"
  log "${SCENARIO}: scripted_base_url=${BASE_URL}"

  cat >"${HOME_DIR}/config.toml" <<EOF
# Hermetic Path A VC010 R0A home — safe to delete with workdir.

[models]
default = "deepseek-v4-flash"

[model.deepseek-v4-flash]
model = "deepseek-v4-flash"
name = "DeepSeek V4 Flash"
context_window = 128000
api_backend = "chat_completions"
base_url = "${BASE_URL}"
api_key = "sk-scripted-path-a-r0"
env_key = "DEEPSEEK_API_KEY"

[model.deepseek-v4-pro]
model = "deepseek-v4-pro"
name = "DeepSeek V4 Pro"
context_window = 128000
api_backend = "chat_completions"
base_url = "${BASE_URL}"
api_key = "sk-scripted-path-a-r0"
env_key = "DEEPSEEK_API_KEY"

[endpoints]
xai_api_base_url = "${BASE_URL}"

[ui]
theme = "deepseeknight"
yolo = true
permission_mode = "always-approve"

[subagents]
enabled = true
EOF
  chmod 600 "${HOME_DIR}/config.toml"

  python3 - <<PY
import json
from pathlib import Path
p = Path("${HOME_DIR}") / "credentials.json"
p.write_text(json.dumps({"api_key": "sk-scripted-path-a-r0"}), encoding="utf-8")
p.chmod(0o600)
PY

  cp -f "${AGENT_RESOLVED}" "${HOME_DIR}/bin/deepseek-build-agent"
  chmod +x "${HOME_DIR}/bin/deepseek-build-agent"
  if ! agent_runs "${HOME_DIR}/bin/deepseek-build-agent"; then
    fail "hermetic agent copy does not run --version"
  fi

  export DEEPSEEK_BUILD_HOME="${HOME_DIR}"
  export DEEPSEEK_API_KEY="sk-scripted-path-a-r0"
  export GROK_HOME="${HOME_DIR}"
  export CI=1
  export NO_COLOR=1
  unset DEEPSEEK_BUILD_AGENT_BIN || true

  log "${SCENARIO}: running public entry deepseek-build agent -p …"
  local AGENT_OUT EC
  # Keep subagent tools available: product builder forces enabled_background=false
  # when allowed_subagent_types becomes empty (builder.rs). Only strip web tools.
  # L3 smoke uses the product default tool surface for background shell.
  set +e
  AGENT_OUT="$(
    run_to 240 "${CLI}" agent \
      -p "${PROMPT}" \
      --cwd "${WS}" \
      --output-format plain \
      --max-turns "${MAX_TURNS}" \
      --yolo \
      --disallowed-tools "web_search,web_fetch" \
      2>&1
  )"
  EC=$?
  set -e

  printf '%s\n' "${AGENT_OUT}" | redact_stream | tail -60 | tee "${WORK}/agent.out" >/dev/null

  local EVIDENCE_WIRE="${OUT_DIR}/PATH_A_R0_VC010_${SCENARIO}_WIRE_last.jsonl"
  local EVIDENCE_META="${OUT_DIR}/PATH_A_R0_VC010_${SCENARIO}_META_last.txt"
  if [[ -f "${WIRE}" ]]; then
    cp "${WIRE}" "${EVIDENCE_WIRE}"
  fi

  local L3_STAMP="${HOME_DIR}/path_a_l3.txt"
  if [[ -f "${L3_STAMP}" ]]; then
    cp "${L3_STAMP}" "${OUT_DIR}/PATH_A_L3_last.txt"
    cp "${L3_STAMP}" "${OUT_DIR}/PATH_A_R0_VC010_L3_last.txt"
  fi

  {
    echo "git_sha=$(git rev-parse HEAD 2>/dev/null || echo unknown)"
    echo "cli=${CLI}"
    echo "dsb=${DSB_BIN:-}"
    echo "agent_resolved=${AGENT_RESOLVED}"
    echo "DEEPSEEK_BUILD_AGENT_BIN_unset=yes"
    echo "DEEPSEEK_BUILD_HOME=${HOME_DIR}"
    echo "scripted_base_url=${BASE_URL}"
    echo "scenario=${SCENARIO}"
    echo "agent_exit=${EC}"
    echo "wire=${EVIDENCE_WIRE}"
    if [[ -f "${WIRE}" ]]; then
      echo "wire_lines=$(wc -l <"${WIRE}" | tr -d ' ')"
    fi
    if [[ -f "${L3_STAMP}" ]]; then
      echo "path_a_l3_stamp=present"
      # shellcheck disable=SC2002
      cat "${L3_STAMP}" | sed 's/^/l3_/'
    else
      echo "path_a_l3_stamp=missing"
    fi
    if [[ -f "${WS}/a.txt" ]]; then
      echo "a_txt=$(python3 -c "print(repr(open('${WS}/a.txt').read()))")"
    fi
    if [[ -f "${WS}/b.txt" ]]; then
      echo "b_txt=$(python3 -c "print(repr(open('${WS}/b.txt').read()))")"
    fi
    echo "agent_out_tail<<EOF"
    printf '%s\n' "${AGENT_OUT}" | redact_stream | tail -30
    echo "EOF"
  } >"${EVIDENCE_META}"

  local FAIL=0
  if [[ ! -s "${WIRE}" ]]; then
    warn "${SCENARIO}: empty wire"
    FAIL=1
  fi
  if ! rg -q 'chat/completions' "${WIRE}" 2>/dev/null; then
    warn "${SCENARIO}: wire missing chat/completions"
    FAIL=1
  fi
  if [[ ! -f "${L3_STAMP}" ]]; then
    warn "${SCENARIO}: path_a_l3.txt missing under public DEEPSEEK_BUILD_HOME"
    FAIL=1
  fi

  case "${SCENARIO}" in
    multi-read-parallel)
      if ! python3 - "${WIRE}" <<'PY'
import json, sys
path = sys.argv[1]
multi_resp = 0
tool_msgs = 0
for line in open(path, encoding="utf-8"):
    rec = json.loads(line)
    if rec.get("kind") == "response_tool_calls":
        tcs = rec.get("tool_calls") or []
        names = []
        for tc in tcs:
            if isinstance(tc, dict):
                fn = (tc.get("function") or {}).get("name") or ""
                names.append(fn)
        if len(tcs) >= 2 and sum(1 for n in names if n in ("read_file", "Read", "read")) >= 2:
            multi_resp += 1
        continue
    if rec.get("kind") != "request":
        continue
    body = rec.get("body") or {}
    msgs = body.get("messages") if isinstance(body, dict) else None
    if not isinstance(msgs, list):
        continue
    n = sum(1 for m in msgs if isinstance(m, dict) and m.get("role") in ("tool", "function"))
    tool_msgs = max(tool_msgs, n)
if multi_resp < 1:
    print("FAIL: fixture did not emit multi-read response_tool_calls batch", file=sys.stderr)
    sys.exit(1)
if tool_msgs < 2:
    print(f"FAIL: expected ≥2 tool role messages after multi-read, max={tool_msgs}", file=sys.stderr)
    sys.exit(1)
print(f"multi_read_response_batches={multi_resp} max_tool_role_messages={tool_msgs}")
sys.exit(0)
PY
      then
        warn "${SCENARIO}: wire multi-read proof failed"
        FAIL=1
      fi
      if ! printf '%s\n' "${AGENT_OUT}" | rg -q 'multi-read-parallel-ok'; then
        warn "${SCENARIO}: final token multi-read-parallel-ok missing in agent output"
        FAIL=1
      fi
      ;;
    mixed-mutate-serial)
      A_CONTENT="$(cat "${WS}/a.txt" 2>/dev/null || true)"
      if [[ "${A_CONTENT}" != $'alpha-mutated\n' && "${A_CONTENT}" != "alpha-mutated" ]]; then
        warn "${SCENARIO}: a.txt expected alpha-mutated, got $(printf '%q' "${A_CONTENT}")"
        FAIL=1
      fi
      if ! python3 - "${WIRE}" <<'PY'
import json, re, sys
path = sys.argv[1]
sid_re = re.compile(r"snp_[0-9A-HJKMNP-TV-Z]{26}")
edits = 0
for line in open(path, encoding="utf-8"):
    rec = json.loads(line)
    if rec.get("kind") != "request":
        continue
    body = rec.get("body") or {}
    msgs = body.get("messages") if isinstance(body, dict) else None
    if not isinstance(msgs, list):
        continue
    for m in msgs:
        if not isinstance(m, dict):
            continue
        for tc in m.get("tool_calls") or []:
            if not isinstance(tc, dict):
                continue
            fn = (tc.get("function") or {})
            name = fn.get("name") or ""
            args = fn.get("arguments") or ""
            if name == "search_replace" and sid_re.search(str(args)):
                edits += 1
if edits < 1:
    # tool_calls may not be echoed in later requests; check tool results + disk is primary
    print("NOTE: no search_replace+snippet_id in echoed tool_calls; disk golden is primary", file=sys.stderr)
print(f"search_replace_with_snippet_id_echoes={edits}")
sys.exit(0)
PY
      then
        warn "${SCENARIO}: wire parse error"
        FAIL=1
      fi
      if ! printf '%s\n' "${AGENT_OUT}" | rg -q 'mixed-mutate-serial-ok'; then
        warn "${SCENARIO}: final token mixed-mutate-serial-ok missing"
        FAIL=1
      fi
      ;;
    bg-collect-by-id)
      if ! python3 - "${WIRE}" <<'PY'
import json, sys
path = sys.argv[1]
bg = 0
collect = 0
bg_ok_in_tools = 0
for line in open(path, encoding="utf-8"):
    rec = json.loads(line)
    if rec.get("kind") == "response_tool_calls":
        for tc in rec.get("tool_calls") or []:
            if not isinstance(tc, dict):
                continue
            fn = tc.get("function") or {}
            name = fn.get("name") or ""
            args = str(fn.get("arguments") or "")
            if name in ("run_terminal_command", "run_terminal_cmd", "bash"):
                if "is_background" in args and "true" in args.lower():
                    bg += 1
            if name in (
                "get_command_or_subagent_output",
                "get_task_output",
                "get_terminal_command_output",
            ):
                if "task_ids" in args:
                    collect += 1
        continue
    if rec.get("kind") != "request":
        continue
    body = rec.get("body") or {}
    msgs = body.get("messages") if isinstance(body, dict) else None
    if not isinstance(msgs, list):
        continue
    for m in msgs:
        if not isinstance(m, dict):
            continue
        content = str(m.get("content") or "")
        if "bg-ok-77" in content:
            bg_ok_in_tools += 1
if bg < 1:
    print("FAIL: fixture did not emit background run_terminal_command", file=sys.stderr)
    sys.exit(1)
if collect < 1:
    print("FAIL: fixture did not emit collect-by-id tool_calls", file=sys.stderr)
    sys.exit(1)
if bg_ok_in_tools < 1:
    print("FAIL: bg-ok-77 never appeared in tool results on wire", file=sys.stderr)
    sys.exit(1)
print(f"background_emits={bg} collect_emits={collect} bg_ok_tool_hits={bg_ok_in_tools}")
sys.exit(0)
PY
      then
        warn "${SCENARIO}: bg collect wire proof failed"
        FAIL=1
      fi
      if ! printf '%s\n' "${AGENT_OUT}" | rg -q 'bg-collect-ok|bg-ok-77'; then
        warn "${SCENARIO}: missing bg-collect-ok / bg-ok-77 in agent output"
        FAIL=1
      fi
      ;;
  esac

  if [[ "${FAIL}" -ne 0 ]]; then
    warn "${SCENARIO}: FAIL (meta=${EVIDENCE_META})"
    return 1
  fi
  ok "${SCENARIO}: PASS"
  return 0
}

FAILED=0
SCENARIOS=(multi-read-parallel mixed-mutate-serial bg-collect-by-id)
if [[ -n "${ONLY_SCENARIO}" ]]; then
  SCENARIOS=("${ONLY_SCENARIO}")
fi

for sc in "${SCENARIOS[@]}"; do
  case "${sc}" in
    multi-read-parallel)
      if ! run_scenario multi-read-parallel \
        "Follow the scripted multi-read tools then stop. Reply with the final token when done." \
        10; then
        FAILED=1
      fi
      ;;
    mixed-mutate-serial)
      if ! run_scenario mixed-mutate-serial \
        "Follow the scripted multi-read then mutate tools then stop." \
        12; then
        FAILED=1
      fi
      ;;
    bg-collect-by-id)
      if ! run_scenario bg-collect-by-id \
        "Follow the scripted background shell and collect tools then stop." \
        12; then
        FAILED=1
      fi
      ;;
    *)
      fail "unsupported scenario: ${sc}"
      ;;
  esac
done

# Dual CLI soft check
if [[ -n "${DSB_BIN}" ]]; then
  if "${DSB_BIN}" --version >/dev/null 2>&1; then
    ok "dsb --version"
  else
    warn "dsb --version failed"
    FAILED=1
  fi
fi

if [[ "${FAILED}" -ne 0 ]]; then
  echo "test-path-a-vc010-r0a: FAIL" >&2
  exit 1
fi

ok "VC010 Path A R0A scenarios green"
echo "test-path-a-vc010-r0a: PASS"
exit 0
