#!/usr/bin/env bash
# VC011 Path A R0A — public deepseek-build/dsb agent subagent + worker cache.
#
# Proves (Spec 60 / vision-complete Grok L3 Path A bar):
#   1. Public CLI → agent_launch → product agent (no DEEPSEEK_BUILD_AGENT_BIN)
#   2. Hermetic home + scripted DeepSeek wire
#   3. explore-subagent: spawn_subagent explore (read-only child) → explore-subagent-ok
#   4. implement-subagent-mutate: spawn_subagent general-purpose mutates disk
#   5. worker-cache-stamp: path_a_l3 worker_epochs_match=true on public launch
#
# Usage:
#   ./scripts/test-path-a-vc011-r0a.sh
#   ./scripts/test-path-a-vc011-r0a.sh --keep
#   ./scripts/test-path-a-vc011-r0a.sh --scenario explore-subagent
#   ./scripts/test-path-a-vc011-r0a.sh --skip-build
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

echo "=== test-path-a-vc011-r0a (Path A L3 subagent + worker cache) ==="
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
  local MAX_TURNS="${3:-16}"

  local WORK HOME_DIR WS WIRE SERVER_PID=""
  WORK="$(mktemp -d "${TMPDIR:-/tmp}/dsb-vc011-r0a.XXXXXX")"
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
    explore-subagent)
      printf 'FINDME-77\n' >"${WS}/explore-marker.txt"
      ;;
    implement-subagent-mutate)
      printf 'seed-before-worker\n' >"${WS}/seed.txt"
      ;;
    worker-cache-stamp)
      printf 'stamp-ws-ready\n' >"${WS}/marker.txt"
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
    --final-text "vc011-${SCENARIO}-done" \
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
# Hermetic Path A VC011 R0A home — safe to delete with workdir.

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
  set +e
  AGENT_OUT="$(
    run_to 300 "${CLI}" agent \
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

  printf '%s\n' "${AGENT_OUT}" | redact_stream | tail -80 | tee "${WORK}/agent.out" >/dev/null

  local EVIDENCE_WIRE="${OUT_DIR}/PATH_A_R0_VC011_${SCENARIO}_WIRE_last.jsonl"
  local EVIDENCE_META="${OUT_DIR}/PATH_A_R0_VC011_${SCENARIO}_META_last.txt"
  if [[ -f "${WIRE}" ]]; then
    cp "${WIRE}" "${EVIDENCE_WIRE}"
  fi

  local L3_STAMP="${HOME_DIR}/path_a_l3.txt"
  if [[ -f "${L3_STAMP}" ]]; then
    cp "${L3_STAMP}" "${OUT_DIR}/PATH_A_L3_last.txt"
    cp "${L3_STAMP}" "${OUT_DIR}/PATH_A_R0_VC011_L3_last.txt"
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
    if [[ -f "${WS}/explore-marker.txt" ]]; then
      echo "explore_marker=$(python3 -c "print(repr(open('${WS}/explore-marker.txt').read()))")"
    fi
    if [[ -f "${WS}/worker_out.txt" ]]; then
      echo "worker_out=$(python3 -c "print(repr(open('${WS}/worker_out.txt').read()))")"
    else
      echo "worker_out=missing"
    fi
    if [[ -f "${WS}/seed.txt" ]]; then
      echo "seed_txt=$(python3 -c "print(repr(open('${WS}/seed.txt').read()))")"
    fi
    echo "agent_out_tail<<EOF"
    printf '%s\n' "${AGENT_OUT}" | redact_stream | tail -40
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
  elif ! rg -q 'worker_epochs_match=true' "${L3_STAMP}"; then
    warn "${SCENARIO}: path_a_l3 worker_epochs_match!=true"
    FAIL=1
  elif ! rg -q 'worker_kind_explore=explore' "${L3_STAMP}"; then
    warn "${SCENARIO}: path_a_l3 missing worker_kind_explore"
    FAIL=1
  elif ! rg -q 'worker_kind_implement=implement' "${L3_STAMP}"; then
    warn "${SCENARIO}: path_a_l3 missing worker_kind_implement"
    FAIL=1
  elif ! rg -q 'subagents_enabled_in_config=true' "${L3_STAMP}"; then
    warn "${SCENARIO}: path_a_l3 subagents_enabled_in_config!=true"
    FAIL=1
  fi

  case "${SCENARIO}" in
    explore-subagent)
      if ! python3 - "${WIRE}" <<'PY'
import json, sys
path = sys.argv[1]
spawn = 0
explore = 0
child_read = 0
for line in open(path, encoding="utf-8"):
    rec = json.loads(line)
    if rec.get("kind") == "response_tool_calls":
        for tc in rec.get("tool_calls") or []:
            if not isinstance(tc, dict):
                continue
            fn = tc.get("function") or {}
            name = fn.get("name") or ""
            args = str(fn.get("arguments") or "")
            if name in ("spawn_subagent", "Agent", "Task"):
                spawn += 1
                if "explore" in args.lower():
                    explore += 1
            if name in ("read_file", "Read", "read") and "explore-marker" in args:
                child_read += 1
        continue
if spawn < 1:
    print("FAIL: fixture did not emit spawn_subagent", file=sys.stderr)
    sys.exit(1)
if explore < 1:
    print("FAIL: spawn_subagent missing explore type", file=sys.stderr)
    sys.exit(1)
print(f"spawn_emits={spawn} explore_typed={explore} child_read_emits={child_read}")
sys.exit(0)
PY
      then
        warn "${SCENARIO}: wire explore spawn proof failed"
        FAIL=1
      fi
      if ! printf '%s\n' "${AGENT_OUT}" | rg -q 'explore-subagent-ok|FINDME-77|explore-child'; then
        warn "${SCENARIO}: missing explore-subagent-ok / FINDME-77 in agent output"
        FAIL=1
      fi
      ;;
    implement-subagent-mutate)
      WORKER_CONTENT="$(cat "${WS}/worker_out.txt" 2>/dev/null || true)"
      if [[ "${WORKER_CONTENT}" != $'worker-mutated-ok\n' && "${WORKER_CONTENT}" != "worker-mutated-ok" ]]; then
        warn "${SCENARIO}: worker_out.txt expected worker-mutated-ok, got $(printf '%q' "${WORKER_CONTENT}")"
        FAIL=1
      fi
      if ! python3 - "${WIRE}" <<'PY'
import json, sys
path = sys.argv[1]
spawn = 0
gp = 0
for line in open(path, encoding="utf-8"):
    rec = json.loads(line)
    if rec.get("kind") != "response_tool_calls":
        continue
    for tc in rec.get("tool_calls") or []:
        if not isinstance(tc, dict):
            continue
        fn = tc.get("function") or {}
        name = fn.get("name") or ""
        args = str(fn.get("arguments") or "")
        if name in ("spawn_subagent", "Agent", "Task"):
            spawn += 1
            if "general-purpose" in args.lower() or "implement" in args.lower():
                gp += 1
if spawn < 1:
    print("FAIL: fixture did not emit spawn_subagent", file=sys.stderr)
    sys.exit(1)
if gp < 1:
    print("FAIL: spawn missing general-purpose / implement-class type", file=sys.stderr)
    sys.exit(1)
print(f"spawn_emits={spawn} implement_class={gp}")
sys.exit(0)
PY
      then
        warn "${SCENARIO}: wire implement spawn proof failed"
        FAIL=1
      fi
      if ! printf '%s\n' "${AGENT_OUT}" | rg -q 'implement-subagent-ok|worker-mutated|implement-child'; then
        warn "${SCENARIO}: missing implement-subagent-ok / worker-mutated in agent output"
        FAIL=1
      fi
      ;;
    worker-cache-stamp)
      if ! printf '%s\n' "${AGENT_OUT}" | rg -q 'worker-cache-stamp-ok'; then
        warn "${SCENARIO}: missing worker-cache-stamp-ok in agent output"
        FAIL=1
      fi
      # Stamp checks already applied above for all scenarios.
      ;;
  esac

  if [[ "${FAIL}" -ne 0 ]]; then
    warn "${SCENARIO}: FAIL (meta=${EVIDENCE_META})"
    if [[ -f "${SERVER_LOG}" ]]; then
      warn "${SCENARIO}: server log tail:"
      tail -40 "${SERVER_LOG}" >&2 || true
    fi
    return 1
  fi
  ok "${SCENARIO}: PASS"
  return 0
}

FAILED=0
SCENARIOS=(explore-subagent implement-subagent-mutate worker-cache-stamp)
if [[ -n "${ONLY_SCENARIO}" ]]; then
  SCENARIOS=("${ONLY_SCENARIO}")
fi

for sc in "${SCENARIOS[@]}"; do
  case "${sc}" in
    explore-subagent)
      if ! run_scenario explore-subagent \
        "Follow the scripted explore subagent tools then stop. Reply with the final token when done." \
        16; then
        FAILED=1
      fi
      ;;
    implement-subagent-mutate)
      if ! run_scenario implement-subagent-mutate \
        "Follow the scripted implement-class subagent tools then stop." \
        16; then
        FAILED=1
      fi
      ;;
    worker-cache-stamp)
      if ! run_scenario worker-cache-stamp \
        "Reply with the scripted final token and stop." \
        6; then
        FAILED=1
      fi
      ;;
    *)
      fail "unknown scenario ${sc}"
      ;;
  esac
done

if [[ "${FAILED}" -ne 0 ]]; then
  fail "VC011 Path A R0A FAILED one or more scenarios"
fi
ok "VC011 Path A R0A all scenarios PASS"
exit 0
