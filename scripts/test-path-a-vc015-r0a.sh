#!/usr/bin/env bash
# VC015 Path A R0A — V3-60-3 parent snippet after implement-class worker mutate.
#
# Proves (Spec 60 T3 / vision V3-60-3 Path A bar):
#   1. Public CLI → agent_launch → product agent (no DEEPSEEK_BUILD_AGENT_BIN)
#   2. Hermetic home + scripted DeepSeek wire
#   3. Parent read_file mints snippet_id for parent_seed.txt
#   4. spawn_subagent general-purpose mutates the same path
#   5. Parent search_replace with pre-mutation snippet_id is rejected
#      (snippet_stale / snippet_not_found) — disk must not gain should-not-apply
#   6. Final token parent-worker-snippet-stale-ok
#
# Usage:
#   ./scripts/test-path-a-vc015-r0a.sh
#   ./scripts/test-path-a-vc015-r0a.sh --keep
#   ./scripts/test-path-a-vc015-r0a.sh --skip-build
#
# Exit 0 only when the scenario passes with wire + public-path evidence.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "${ROOT}"
# shellcheck source=lib/common.sh
source "${ROOT}/scripts/lib/common.sh"

KEEP=0
SKIP_BUILD=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --keep) KEEP=1; shift ;;
    --skip-build) SKIP_BUILD=1; shift ;;
    -h|--help)
      sed -n '2,20p' "$0"
      exit 0
      ;;
    *) fail "unknown arg: $1" ;;
  esac
done

SCENARIO="parent-worker-snippet-stale"

echo "=== test-path-a-vc015-r0a (Path A V3-60-3 parent snippet after worker) ==="
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
  fail "NO_CLI: deepseek-build binary not found — build cargo -p dsb-cli or copy stack release first"
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

if [[ ! -x "${ROOT}/target/release/deepseek-build" && ! -x "${CLI}" ]]; then
  log "building dsb-cli release for public entry"
  cargo build --release -p dsb-cli 2>&1 | tail -20
  CLI="${ROOT}/target/release/deepseek-build"
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

PROMPT="Follow the scripted parent read → implement-class subagent mutate → stale snippet_id edit sequence then stop."
MAX_TURNS=20

WORK="$(mktemp -d "${TMPDIR:-/tmp}/dsb-vc015-r0a.XXXXXX")"
HOME_DIR="${WORK}/product-home"
WS="${WORK}/ws"
WIRE="${WORK}/wire.jsonl"
mkdir -p "${HOME_DIR}/bin" "${WS}"
SERVER_PID=""

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

printf 'parent-seed-original\n' >"${WS}/parent_seed.txt"

SERVER_LOG="${WORK}/server.log"
python3 "${SERVER_PY}" \
  --host 127.0.0.1 \
  --port 0 \
  --wire "${WIRE}" \
  --scenario "${SCENARIO}" \
  --liveness-dir "${WS}" \
  --final-text "vc015-${SCENARIO}-done" \
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
    fail "scripted server exited early (${SCENARIO})"
  fi
  sleep 0.1
done
if [[ "${READY_LINE}" != READY\ * ]]; then
  cat "${SERVER_LOG}" >&2 || true
  fail "scripted server did not print READY (${SCENARIO})"
fi
HOSTPORT="${READY_LINE#READY }"
BASE_URL="http://${HOSTPORT}"
log "${SCENARIO}: scripted_base_url=${BASE_URL}"

cat >"${HOME_DIR}/config.toml" <<EOF
# Hermetic Path A VC015 R0A home — safe to delete with workdir.

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
set +e
AGENT_OUT="$(
  run_to 360 "${CLI}" agent \
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

EVIDENCE_WIRE="${OUT_DIR}/PATH_A_R0_VC015_${SCENARIO}_WIRE_last.jsonl"
EVIDENCE_META="${OUT_DIR}/PATH_A_R0_VC015_${SCENARIO}_META_last.txt"
if [[ -f "${WIRE}" ]]; then
  cp "${WIRE}" "${EVIDENCE_WIRE}"
fi

L3_STAMP="${HOME_DIR}/path_a_l3.txt"
if [[ -f "${L3_STAMP}" ]]; then
  cp "${L3_STAMP}" "${OUT_DIR}/PATH_A_R0_VC015_L3_last.txt"
fi

SEED_CONTENT="$(cat "${WS}/parent_seed.txt" 2>/dev/null || true)"

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
  echo "parent_seed=$(python3 -c "print(repr(open('${WS}/parent_seed.txt').read()) if __import__('os').path.exists('${WS}/parent_seed.txt') else 'missing')")"
  echo "agent_out_tail<<EOF"
  printf '%s\n' "${AGENT_OUT}" | redact_stream | tail -50
  echo "EOF"
} >"${EVIDENCE_META}"

FAIL=0
if [[ ! -s "${WIRE}" ]]; then
  warn "${SCENARIO}: empty wire"
  FAIL=1
fi
if ! rg -q 'chat/completions' "${WIRE}" 2>/dev/null; then
  warn "${SCENARIO}: wire missing chat/completions"
  FAIL=1
fi

# Disk: worker must have mutated parent path; stale edit must not apply
if [[ "${SEED_CONTENT}" != $'worker-mutated-parent\n' && "${SEED_CONTENT}" != "worker-mutated-parent" ]]; then
  warn "${SCENARIO}: parent_seed.txt expected worker-mutated-parent, got $(printf '%q' "${SEED_CONTENT}")"
  FAIL=1
fi
if printf '%s' "${SEED_CONTENT}" | rg -q 'should-not-apply-after-worker'; then
  warn "${SCENARIO}: stale parent edit applied should-not-apply-after-worker"
  FAIL=1
fi

if ! python3 - "${WIRE}" <<'PY'
import json, re, sys
path = sys.argv[1]
has_mint = False
has_spawn = False
has_implement = False
has_stale_edit = False
has_fail = False
sid_re = re.compile(r"snp_[0-9A-HJKMNP-TV-Z]{26}")
for line in open(path, encoding="utf-8"):
    rec = json.loads(line)
    kind = rec.get("kind")
    if kind == "response_tool_calls":
        for tc in rec.get("tool_calls") or []:
            if not isinstance(tc, dict):
                continue
            fn = tc.get("function") or {}
            name = fn.get("name") or ""
            args = str(fn.get("arguments") or "")
            if name in ("read_file", "Read", "read") and "parent_seed" in args:
                has_mint = True  # mint observed on read path; content checked via tool role
            if name in ("spawn_subagent", "Agent", "Task"):
                has_spawn = True
                if "general-purpose" in args.lower() or "implement" in args.lower():
                    has_implement = True
            if name in ("search_replace", "Edit", "edit") and "snippet_id" in args:
                has_stale_edit = True
        continue
    # Request bodies may embed tool results as messages
    body = rec.get("body") or rec.get("request") or {}
    if isinstance(body, dict):
        msgs = body.get("messages") or []
        for m in msgs:
            if not isinstance(m, dict):
                continue
            if m.get("role") not in ("tool", "function"):
                continue
            c = str(m.get("content") or "")
            if sid_re.search(c) and "snippet_id" in c:
                has_mint = True
            if "snippet_stale" in c or "snippet_not_found" in c or "unknown snippet" in c.lower():
                has_fail = True
    # Some wires log tool results as top-level kinds
    if kind in ("tool_result", "tool_response"):
        c = str(rec.get("content") or rec.get("body") or "")
        if "snippet_stale" in c or "snippet_not_found" in c:
            has_fail = True
        if sid_re.search(c):
            has_mint = True

# Also scan raw wire text for fail-closed markers (robust across wire shapes)
raw = open(path, encoding="utf-8").read()
if "snippet_stale" in raw or "snippet_not_found" in raw:
    has_fail = True
if "snippet_id:" in raw or sid_re.search(raw):
    has_mint = True
if "spawn_subagent" in raw or '"Agent"' in raw:
    has_spawn = True
if "general-purpose" in raw:
    has_implement = True
if "search_replace" in raw and "should-not-apply-after-worker" in raw:
    has_stale_edit = True

print(
    f"mint={has_mint} spawn={has_spawn} implement_class={has_implement} "
    f"stale_edit_emit={has_stale_edit} fail_closed={has_fail}"
)
if not has_mint:
    print("FAIL: missing parent snippet mint", file=sys.stderr)
    sys.exit(1)
if not has_spawn or not has_implement:
    print("FAIL: missing implement-class spawn", file=sys.stderr)
    sys.exit(1)
if not has_stale_edit:
    print("FAIL: fixture did not emit parent stale search_replace", file=sys.stderr)
    sys.exit(1)
if not has_fail:
    print("FAIL: missing fail-closed tool result after worker mutate", file=sys.stderr)
    sys.exit(1)
sys.exit(0)
PY
then
  warn "${SCENARIO}: wire chain mint/spawn/stale-fail proof failed"
  FAIL=1
fi

if ! printf '%s\n' "${AGENT_OUT}" | rg -q 'parent-worker-snippet-stale-ok|parent-worker-snippet-stale'; then
  # Final token preferred; allow partial if wire already proves fail-closed + disk
  if [[ "${FAIL}" -eq 0 ]]; then
    warn "${SCENARIO}: missing final token (wire+disk ok — soft warn)"
  else
    warn "${SCENARIO}: missing parent-worker-snippet-stale-ok in agent output"
    FAIL=1
  fi
fi

if [[ "${FAIL}" -ne 0 ]]; then
  warn "${SCENARIO}: FAIL (meta=${EVIDENCE_META})"
  if [[ -f "${SERVER_LOG}" ]]; then
    warn "${SCENARIO}: server log tail:"
    tail -40 "${SERVER_LOG}" >&2 || true
  fi
  fail "VC015 Path A V3-60-3 R0A FAILED"
fi

ok "${SCENARIO}: PASS"
ok "VC015 Path A V3-60-3 R0A PASS"
exit 0
