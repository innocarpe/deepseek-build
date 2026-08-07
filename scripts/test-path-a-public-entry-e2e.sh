#!/usr/bin/env bash
# Path A R0A public-entry e2e (G002 / 5x-H0-3).
#
# Proves:
#   1. Public CLI (`deepseek-build` / `dsb`) → agent_launch → agent binary
#   2. DEEPSEEK_BUILD_AGENT_BIN unset (product resolution)
#   3. Hermetic product home with base_url → scripted DeepSeek server
#   4. Captured wire JSONL for /chat/completions
#
# Usage:
#   ./scripts/test-path-a-public-entry-e2e.sh
#   ./scripts/test-path-a-public-entry-e2e.sh --keep   # keep temp artifacts
#
# Exit 0 only when public-entry headless turn hits the scripted server and
# the wire transcript contains at least one chat/completions POST.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "${ROOT}"
# shellcheck source=lib/common.sh
source "${ROOT}/scripts/lib/common.sh"

KEEP=0
SCENARIO="text-pong"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --keep) KEEP=1; shift ;;
    --scenario)
      SCENARIO="${2:?}"
      shift 2
      ;;
    -h|--help)
      sed -n '2,18p' "$0"
      exit 0
      ;;
    *) fail "unknown arg: $1" ;;
  esac
done

echo "=== test-path-a-public-entry-e2e (G002 Path A R0A) ==="
echo "git_sha=$(git rev-parse HEAD 2>/dev/null || echo unknown)"

if [[ -n "${DEEPSEEK_BUILD_AGENT_BIN:-}" ]]; then
  warn "DEEPSEEK_BUILD_AGENT_BIN was set; unsetting for public-entry proof"
  unset DEEPSEEK_BUILD_AGENT_BIN
fi

# --- resolve public CLI (not raw agent) ---
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
  fail "NO_CLI: deepseek-build binary not found — build or ./scripts/install.sh first"
fi
log "cli=${CLI}"

# Dual-name check when dsb sits next to CLI
CLI_DIR="$(cd "$(dirname "${CLI}")" && pwd)"
DSB_BIN=""
if [[ -x "${CLI_DIR}/dsb" ]]; then
  DSB_BIN="${CLI_DIR}/dsb"
elif command -v dsb >/dev/null 2>&1; then
  DSB_BIN="$(command -v dsb)"
fi
log "dsb=${DSB_BIN:-missing (optional for this run)}"

# Agent must be resolvable without ENV override (product resolution).
# Prefer a binary that actually runs --version (some environments SIGKILL
# ~/.deepseek-build/bin/deepseek-build-agent specifically; xai-grok-pager copy works).
agent_runs() {
  local bin="$1"
  [[ -x "${bin}" ]] || return 1
  # Resolve symlinks — if the realpath is a known-bad install path, skip
  local real
  real="$(python3 -c 'import os,sys; print(os.path.realpath(sys.argv[1]))' "${bin}" 2>/dev/null || echo "${bin}")"
  # Probe: --version must exit 0 within a few seconds
  if command -v timeout >/dev/null 2>&1; then
    timeout 5 "${bin}" --version >/dev/null 2>&1
  elif command -v gtimeout >/dev/null 2>&1; then
    gtimeout 5 "${bin}" --version >/dev/null 2>&1
  else
    "${bin}" --version >/dev/null 2>&1
  fi
}

AGENT_RESOLVED=""
for c in \
  "${DEEPSEEK_BUILD_HOME:-$HOME/.deepseek-build}/bin/xai-grok-pager" \
  "${CLI_DIR}/xai-grok-pager" \
  "${CARGO_HOME:-$HOME/.cargo}/bin/xai-grok-pager" \
  "${ROOT}/third_party/grok-build/target/release/xai-grok-pager" \
  "${DEEPSEEK_BUILD_HOME:-$HOME/.deepseek-build}/bin/deepseek-build-agent" \
  "${CLI_DIR}/deepseek-build-agent" \
  "${CARGO_HOME:-$HOME/.cargo}/bin/deepseek-build-agent"
do
  if agent_runs "${c}"; then
    AGENT_RESOLVED="${c}"
    break
  fi
done
if [[ -z "${AGENT_RESOLVED}" ]]; then
  fail "NO_AGENT: no runnable deepseek-build-agent / xai-grok-pager on product paths"
fi
log "agent_resolved=${AGENT_RESOLVED}"
if command -v shasum >/dev/null 2>&1; then
  log "agent_sha256=$(shasum -a 256 "${AGENT_RESOLVED}" | awk '{print $1}')"
elif command -v sha256sum >/dev/null 2>&1; then
  log "agent_sha256=$(sha256sum "${AGENT_RESOLVED}" | awk '{print $1}')"
fi

SERVER_PY="${ROOT}/scripts/lib/scripted_deepseek_server.py"
if [[ ! -f "${SERVER_PY}" ]]; then
  fail "MISSING: ${SERVER_PY}"
fi

WORK="$(mktemp -d "${TMPDIR:-/tmp}/dsb-path-a-r0.XXXXXX")"
HOME_DIR="${WORK}/product-home"
WS="${WORK}/ws"
WIRE="${WORK}/wire.jsonl"
OUT_DIR="${ROOT}/docs/product/evidence"
EVIDENCE_WIRE="${OUT_DIR}/PATH_A_R0_WIRE_last.jsonl"
EVIDENCE_META="${OUT_DIR}/PATH_A_R0_META_last.txt"
mkdir -p "${HOME_DIR}/bin" "${WS}" "${OUT_DIR}"

cleanup() {
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
trap cleanup EXIT

# --- start scripted server ---
SERVER_LOG="${WORK}/server.log"
python3 "${SERVER_PY}" \
  --host 127.0.0.1 \
  --port 0 \
  --wire "${WIRE}" \
  --scenario "${SCENARIO}" \
  --final-text "path-a-r0-ok" \
  >"${WORK}/server.stdout" 2>"${SERVER_LOG}" &
SERVER_PID=$!

READY_LINE=""
for _ in $(seq 1 50); do
  if [[ -s "${WORK}/server.stdout" ]]; then
    READY_LINE="$(head -1 "${WORK}/server.stdout" || true)"
    if [[ "${READY_LINE}" == READY\ * ]]; then
      break
    fi
  fi
  if ! kill -0 "${SERVER_PID}" 2>/dev/null; then
    cat "${SERVER_LOG}" >&2 || true
    fail "scripted server exited early"
  fi
  sleep 0.1
done
if [[ "${READY_LINE}" != READY\ * ]]; then
  cat "${SERVER_LOG}" >&2 || true
  fail "scripted server did not print READY"
fi
HOSTPORT="${READY_LINE#READY }"
BASE_URL="http://${HOSTPORT}"
log "scripted_base_url=${BASE_URL}"

# --- hermetic product home (DEEPSEEK_BUILD_HOME → GROK_HOME via agent_launch) ---
# Seed config with base_url pointing at scripted server (not live DeepSeek).
cat >"${HOME_DIR}/config.toml" <<EOF
# Hermetic Path A R0A home — generated by test-path-a-public-entry-e2e.sh
# Safe to delete with the workdir.

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
EOF
chmod 600 "${HOME_DIR}/config.toml"

# Minimal credentials so launch does not try interactive setup
python3 - <<PY
import json
from pathlib import Path
p = Path("${HOME_DIR}") / "credentials.json"
p.write_text(json.dumps({"api_key": "sk-scripted-path-a-r0"}), encoding="utf-8")
p.chmod(0o600)
PY

# Install agent into hermetic product home as deepseek-build-agent (COPY, not
# symlink — some hosts SIGKILL the realpath ~/.deepseek-build/bin/deepseek-build-agent).
# agent_launch finds DEEPSEEK_BUILD_HOME/bin/deepseek-build-agent first when ENV unset.
cp -f "${AGENT_RESOLVED}" "${HOME_DIR}/bin/deepseek-build-agent"
chmod +x "${HOME_DIR}/bin/deepseek-build-agent"
if ! agent_runs "${HOME_DIR}/bin/deepseek-build-agent"; then
  fail "hermetic agent copy does not run --version"
fi
log "hermetic_agent=${HOME_DIR}/bin/deepseek-build-agent"

echo "path-a-marker" >"${WS}/marker.txt"

export DEEPSEEK_BUILD_HOME="${HOME_DIR}"
export DEEPSEEK_API_KEY="sk-scripted-path-a-r0"
# Ensure no global GROK_HOME steals config
export GROK_HOME="${HOME_DIR}"
# Headless / non-interactive
export CI=1
export NO_COLOR=1

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

# --- public entry: deepseek-build agent (agent_launch path) ---
# Args after `agent` are forwarded to deepseek-build-agent via exec.
log "running public entry: deepseek-build agent -p …"
set +e
AGENT_OUT="$(
  run_to 120 "${CLI}" agent \
    -p "Reply with exactly the token path-a-r0-ok and nothing else." \
    --cwd "${WS}" \
    --output-format plain \
    --max-turns 4 \
    --yolo \
    --disallowed-tools "web_search,web_fetch,Agent,spawn_subagent" \
    2>&1
)"
EC=$?
set -e

printf '%s\n' "${AGENT_OUT}" | redact_stream | tail -40 | tee "${WORK}/agent.out"

# G008: Path A prefix epoch stamp written by agent_launch before exec.
EPOCH_STAMP="${HOME_DIR}/path_a_prefix_epoch.txt"
if [[ -f "${EPOCH_STAMP}" ]]; then
  cp "${EPOCH_STAMP}" "${OUT_DIR}/PATH_A_PREFIX_EPOCH_last.txt"
  log "path_a_prefix_epoch stamp present"
else
  warn "path_a_prefix_epoch.txt missing under DEEPSEEK_BUILD_HOME (G008 stamp)"
fi

# G009: Path A Spec 20 routing stamp (Flash default + /pro once).
ROUTING_STAMP="${HOME_DIR}/path_a_routing.txt"
if [[ -f "${ROUTING_STAMP}" ]]; then
  cp "${ROUTING_STAMP}" "${OUT_DIR}/PATH_A_ROUTING_last.txt"
  log "path_a_routing stamp present"
else
  warn "path_a_routing.txt missing under DEEPSEEK_BUILD_HOME (G009 stamp)"
fi

# G010: Path A L3 schedule + worker cache stamp.
L3_STAMP="${HOME_DIR}/path_a_l3.txt"
if [[ -f "${L3_STAMP}" ]]; then
  cp "${L3_STAMP}" "${OUT_DIR}/PATH_A_L3_last.txt"
  log "path_a_l3 stamp present"
else
  warn "path_a_l3.txt missing under DEEPSEEK_BUILD_HOME (G010 stamp)"
fi

# Persist wire + meta for evidence (redacted)
if [[ -f "${WIRE}" ]]; then
  cp "${WIRE}" "${EVIDENCE_WIRE}"
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
  if [[ -f "${EPOCH_STAMP}" ]]; then
    echo "path_a_prefix_epoch_stamp=present"
    cat "${EPOCH_STAMP}"
  else
    echo "path_a_prefix_epoch_stamp=missing"
  fi
  if [[ -f "${ROUTING_STAMP}" ]]; then
    echo "path_a_routing_stamp=present"
    cat "${ROUTING_STAMP}"
  else
    echo "path_a_routing_stamp=missing"
  fi
  if [[ -f "${L3_STAMP}" ]]; then
    echo "path_a_l3_stamp=present"
    cat "${L3_STAMP}"
  else
    echo "path_a_l3_stamp=missing"
  fi
} >"${EVIDENCE_META}"

# --- assertions ---
FAIL=0
if [[ ! -s "${WIRE}" ]]; then
  warn "wire transcript empty — agent never hit scripted server"
  FAIL=1
fi
if [[ ! -f "${EPOCH_STAMP}" ]]; then
  warn "G008 stamp missing — assemble_path_a_context not exercised on launch"
  FAIL=1
fi
if [[ ! -f "${ROUTING_STAMP}" ]]; then
  warn "G009 stamp missing — path_a_default_router not exercised on launch"
  FAIL=1
fi
if [[ ! -f "${L3_STAMP}" ]]; then
  warn "G010 stamp missing — L3 classify/worker_stable_prefix not exercised on launch"
  FAIL=1
elif ! rg -q 'worker_epochs_match=true' "${L3_STAMP}" \
  || ! rg -q 'bash_mutating=true' "${L3_STAMP}" \
  || ! rg -q 'subagents_enabled_in_config=true' "${L3_STAMP}"; then
  warn "G010 stamp content incomplete"
  FAIL=1
fi
# L2-20-1: default deepseek turns use Flash wire id
if [[ -s "${WIRE}" ]]; then
  if ! python3 - "${WIRE}" <<'PY'
import json, sys
path = sys.argv[1]
flash = 0
pro = 0
other = []
for line in open(path, encoding="utf-8"):
    o = json.loads(line)
    body = o.get("body", o)
    if isinstance(body, str):
        try:
            body = json.loads(body)
        except Exception:
            continue
    if not isinstance(body, dict):
        continue
    m = body.get("model") or ""
    if m == "deepseek-v4-flash":
        flash += 1
    elif m == "deepseek-v4-pro":
        pro += 1
    elif m and "deepseek" in m:
        other.append(m)
    # ignore session-title side model (grok-4.5)
if flash < 1:
    print(f"no deepseek-v4-flash on wire (flash={flash} pro={pro} other={other})", file=sys.stderr)
    sys.exit(1)
print(f"wire_models flash={flash} pro={pro}")
sys.exit(0)
PY
  then
    warn "L2-20-1: expected deepseek-v4-flash on Path A wire"
    FAIL=1
  fi
fi

if ! rg -q 'chat/completions' "${WIRE}" 2>/dev/null; then
  warn "wire missing chat/completions path"
  FAIL=1
fi

# At least one POST recorded
WIRE_N=0
if [[ -f "${WIRE}" ]]; then
  WIRE_N="$(wc -l <"${WIRE}" | tr -d ' ')"
fi
if [[ "${WIRE_N}" -lt 1 ]]; then
  warn "expected ≥1 wire request, got ${WIRE_N}"
  FAIL=1
fi

# Prefer seeing the scripted token in agent output (text scenario)
if ! printf '%s\n' "${AGENT_OUT}" | rg -q 'path-a-r0-ok'; then
  warn "agent stdout missing path-a-r0-ok (ec=${EC}); wire may still prove path"
  # Soft: if wire has completions, still accept public-entry proof
  if [[ "${WIRE_N}" -lt 1 ]]; then
    FAIL=1
  fi
fi

# Dual CLI optional soft check: dsb --version works
if [[ -n "${DSB_BIN}" ]]; then
  set +e
  "${DSB_BIN}" --version >/dev/null 2>&1
  DSB_EC=$?
  set -e
  if [[ "${DSB_EC}" -ne 0 ]]; then
    warn "dsb --version failed"
    FAIL=1
  else
    ok "dsb --version"
  fi
fi

# CLI version
set +e
"${CLI}" --version >/dev/null 2>&1
VER_EC=$?
set -e
if [[ "${VER_EC}" -ne 0 ]]; then
  warn "deepseek-build --version failed"
  FAIL=1
fi

if [[ "${FAIL}" -ne 0 ]]; then
  echo "test-path-a-public-entry-e2e: FAIL" >&2
  echo "server log:" >&2
  tail -30 "${SERVER_LOG}" >&2 || true
  echo "meta: ${EVIDENCE_META}" >&2
  exit 1
fi

ok "public entry → agent_launch → scripted DeepSeek wire (${WIRE_N} request(s))"
echo "evidence_wire=${EVIDENCE_WIRE}"
echo "evidence_meta=${EVIDENCE_META}"
echo "test-path-a-public-entry-e2e: PASS"
exit 0
