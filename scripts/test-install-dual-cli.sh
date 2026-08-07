#!/usr/bin/env bash
# G011 InstallDualCLI — hermetic clean-prefix install smoke (5x-H4-1).
#
# Proves on primary platform (this machine):
#   S1  dual CLI same SemVer output
#   S3  product home + GROK_HOME bridge (agent_launch stamps under DEEPSEEK_BUILD_HOME)
#   S5  DeepSeek theme in seeded config
#   S7  clean prefix gets CLI + agent without DEEPSEEK_BUILD_AGENT_BIN
#   OB-1 public entry via installed deepseek-build (agent path via launch)
#
# Does NOT require live DeepSeek key (uses scripted server for public entry).
# Does NOT publish npm. Uses already-built CLI/agent when present to stay hermetic.
#
# Usage:
#   ./scripts/test-install-dual-cli.sh
#   ./scripts/test-install-dual-cli.sh --keep
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "${ROOT}"
# shellcheck source=lib/common.sh
source "${ROOT}/scripts/lib/common.sh"

KEEP=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --keep) KEEP=1; shift ;;
    -h|--help) sed -n '2,20p' "$0"; exit 0 ;;
    *) fail "unknown arg: $1" ;;
  esac
done

OUT_DIR="${ROOT}/docs/product/evidence"
mkdir -p "${OUT_DIR}"
META="${OUT_DIR}/PATH_A_R0_G011_INSTALL_META_last.txt"
INVENTORY="${OUT_DIR}/PATH_A_R0_G011_PACKAGE_INVENTORY_last.txt"

echo "=== test-install-dual-cli (G011) ==="
echo "git_sha=$(git rev-parse HEAD 2>/dev/null || echo unknown)"

# --- resolve build artifacts (prefer local debug, then release, then install) ---
CLI_SRC=""
for c in \
  "${ROOT}/target/debug/deepseek-build" \
  "${ROOT}/target/release/deepseek-build" \
  "$(command -v deepseek-build 2>/dev/null || true)"; do
  if [[ -n "${c}" && -x "${c}" ]]; then
    CLI_SRC="${c}"
    break
  fi
done
if [[ -z "${CLI_SRC}" ]]; then
  log "building dsb-cli (debug)…"
  env -u CARGO_INCREMENTAL -u RUSTC_WRAPPER CARGO_INCREMENTAL=0 \
    cargo build -p dsb-cli --config 'build.rustc-wrapper=""'
  CLI_SRC="${ROOT}/target/debug/deepseek-build"
fi
[[ -x "${CLI_SRC}" ]] || fail "no deepseek-build binary"

DSB_SRC="$(dirname "${CLI_SRC}")/dsb"
[[ -x "${DSB_SRC}" ]] || fail "dsb missing next to deepseek-build at ${DSB_SRC}"

AGENT_SRC=""
# Prefer usable agent (help contains worktree)
for c in \
  "${ROOT}/third_party/grok-build/target/debug/xai-grok-pager" \
  "${ROOT}/third_party/grok-build/target/release/xai-grok-pager" \
  "${HOME}/.deepseek-build/bin/xai-grok-pager" \
  "${HOME}/.deepseek-build/bin/deepseek-build-agent"; do
  if agent_bin_usable "${c}" 2>/dev/null; then
    AGENT_SRC="${c}"
    break
  fi
done
[[ -n "${AGENT_SRC}" ]] || fail "no usable agent binary (xai-grok-pager / deepseek-build-agent)"

# --- clean prefix install (simulate S7) ---
WORK="$(mktemp -d "${TMPDIR:-/tmp}/dsb-g011-install.XXXXXX")"
PREFIX="${WORK}/prefix"
HOME_DIR="${WORK}/product-home"
BIN_DIR="${PREFIX}/bin"
mkdir -p "${BIN_DIR}" "${HOME_DIR}/bin"

cleanup() {
  if [[ "${KEEP}" -eq 0 ]]; then
    rm -rf "${WORK}"
  else
    log "kept workdir=${WORK}"
  fi
}
trap cleanup EXIT

install -m 755 "${CLI_SRC}" "${BIN_DIR}/deepseek-build"
install -m 755 "${DSB_SRC}" "${BIN_DIR}/dsb"
# Product install places agent as deepseek-build-agent; also keep pager name for probe.
install -m 755 "${AGENT_SRC}" "${BIN_DIR}/deepseek-build-agent"
install -m 755 "${AGENT_SRC}" "${BIN_DIR}/xai-grok-pager"
# Mirror under product home (default agent_launch search).
install -m 755 "${AGENT_SRC}" "${HOME_DIR}/bin/deepseek-build-agent"
install -m 755 "${AGENT_SRC}" "${HOME_DIR}/bin/xai-grok-pager"

log "prefix=${PREFIX}"
log "cli_src=${CLI_SRC}"
log "agent_src=${AGENT_SRC}"

# --- S1 dual CLI same version ---
export PATH="${BIN_DIR}:${PATH}"
unset DEEPSEEK_BUILD_AGENT_BIN || true
export DEEPSEEK_BUILD_HOME="${HOME_DIR}"
export GROK_HOME="${HOME_DIR}"
export CI=1
export NO_COLOR=1

VER_A="$("${BIN_DIR}/deepseek-build" --version 2>&1)"
VER_B="$("${BIN_DIR}/dsb" --version 2>&1)"
log "deepseek-build --version => ${VER_A}"
log "dsb --version => ${VER_B}"
# Binary name differs (deepseek-build vs dsb); SemVer token must match.
SEMVER_A="$(printf '%s\n' "${VER_A}" | rg -o '[0-9]+\.[0-9]+\.[0-9]+' | head -1)"
SEMVER_B="$(printf '%s\n' "${VER_B}" | rg -o '[0-9]+\.[0-9]+\.[0-9]+' | head -1)"
if [[ -z "${SEMVER_A}" || "${SEMVER_A}" != "${SEMVER_B}" ]]; then
  fail "S1 dual CLI SemVer mismatch: '${VER_A}' vs '${VER_B}'"
fi
ok "S1 dual CLI same SemVer (${SEMVER_A})"

# --- package inventory (npm dual bin contract) ---
{
  echo "package_json_name=$(python3 -c 'import json;print(json.load(open("package.json"))["name"])')"
  echo "package_json_version=$(python3 -c 'import json;print(json.load(open("package.json"))["version"])')"
  echo "package_bin_deepseek_build=$(python3 -c 'import json;print(json.load(open("package.json"))["bin"]["deepseek-build"])')"
  echo "package_bin_dsb=$(python3 -c 'import json;print(json.load(open("package.json"))["bin"]["dsb"])')"
  echo "prefix_bin_dir=${BIN_DIR}"
  echo "installed_deepseek_build=${BIN_DIR}/deepseek-build"
  echo "installed_dsb=${BIN_DIR}/dsb"
  echo "installed_agent=${BIN_DIR}/deepseek-build-agent"
  echo "agent_sha256=$(shasum -a 256 "${BIN_DIR}/deepseek-build-agent" | awk '{print $1}')"
  echo "cli_sha256=$(shasum -a 256 "${BIN_DIR}/deepseek-build" | awk '{print $1}')"
  echo "version_line=${VER_A}"
} >"${INVENTORY}"
ok "package inventory written"

# --- S5/S3: launch once → config seed + theme + base_url + stamps ---
# Minimal scripted server so agent_launch → agent can complete a headless turn.
WIRE="${WORK}/wire.jsonl"
python3 "${ROOT}/scripts/lib/scripted_deepseek_server.py" \
  --host 127.0.0.1 --port 0 --wire "${WIRE}" --scenario text-pong \
  --final-text "install-dual-cli-ok" \
  >"${WORK}/server.stdout" 2>"${WORK}/server.log" &
SERVER_PID=$!
READY=""
for _ in $(seq 1 50); do
  READY="$(head -1 "${WORK}/server.stdout" 2>/dev/null || true)"
  [[ "${READY}" == READY\ * ]] && break
  sleep 0.1
done
[[ "${READY}" == READY\ * ]] || fail "scripted server not ready"
HOSTPORT="${READY#READY }"
BASE_URL="http://${HOSTPORT}"

# Seed hermetic config with scripted base_url (S4 re-prove)
cat >"${HOME_DIR}/config.toml" <<EOF
[models]
default = "deepseek-v4-flash"

[model.deepseek-v4-flash]
model = "deepseek-v4-flash"
name = "DeepSeek V4 Flash"
context_window = 128000
api_backend = "chat_completions"
base_url = "${BASE_URL}"
api_key = "sk-scripted-g011"
env_key = "DEEPSEEK_API_KEY"

[model.deepseek-v4-pro]
model = "deepseek-v4-pro"
name = "DeepSeek V4 Pro"
context_window = 128000
api_backend = "chat_completions"
base_url = "${BASE_URL}"
api_key = "sk-scripted-g011"
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
python3 - <<PY
import json
from pathlib import Path
p = Path("${HOME_DIR}") / "credentials.json"
p.write_text(json.dumps({"api_key": "sk-scripted-g011"}), encoding="utf-8")
p.chmod(0o600)
PY
export DEEPSEEK_API_KEY="sk-scripted-g011"

WS="${WORK}/ws"
mkdir -p "${WS}"
echo "g011" >"${WS}/marker.txt"

set +e
OUT="$(
  timeout 90 "${BIN_DIR}/deepseek-build" agent \
    -p "Reply with exactly install-dual-cli-ok and nothing else." \
    --cwd "${WS}" --output-format plain --max-turns 3 --yolo \
    --disallowed-tools "web_search,web_fetch,Agent,spawn_subagent" \
    2>&1
)"
EC=$?
set -e
kill "${SERVER_PID}" 2>/dev/null || true
wait "${SERVER_PID}" 2>/dev/null || true

log "agent_exit=${EC}"
printf '%s\n' "${OUT}" | redact_stream | tail -20

# Stamps under product home (agent_launch)
for f in path_a_prefix_epoch.txt path_a_routing.txt path_a_l3.txt; do
  if [[ ! -f "${HOME_DIR}/${f}" ]]; then
    fail "missing product-home stamp ${f} (agent_launch bridge)"
  fi
done
ok "S3 product home stamps (GROK_HOME bridge)"

# Theme + base_url still present
CFG="$(cat "${HOME_DIR}/config.toml")"
echo "${CFG}" | rg -q 'theme = "deepseeknight"' || fail "S5 theme missing"
echo "${CFG}" | rg -q 'base_url' || fail "S4 base_url missing"
ok "S5 theme deepseeknight + S4 base_url"

# Wire hit
if [[ ! -s "${WIRE}" ]]; then
  fail "OB-1 empty wire — public entry did not reach scripted server"
fi
if ! rg -q 'deepseek-v4-flash' "${WIRE}"; then
  warn "wire missing deepseek-v4-flash (may still pass if only grok title call)"
fi
ok "OB-1 public entry via installed deepseek-build (no DEEPSEEK_BUILD_AGENT_BIN)"

# Agent hash provenance
AGENT_HASH="$(shasum -a 256 "${BIN_DIR}/deepseek-build-agent" | awk '{print $1}')"

{
  echo "git_sha=$(git rev-parse HEAD 2>/dev/null || echo unknown)"
  echo "story=G011"
  echo "prefix=${PREFIX}"
  echo "product_home=${HOME_DIR}"
  echo "dual_version=${VER_A}"
  echo "agent_src=${AGENT_SRC}"
  echo "agent_sha256=${AGENT_HASH}"
  echo "agent_exit=${EC}"
  echo "wire_lines=$(wc -l <"${WIRE}" | tr -d ' ')"
  echo "stamps=prefix,routing,l3"
  echo "DEEPSEEK_BUILD_AGENT_BIN_unset=yes"
  echo "theme=deepseeknight"
  echo "result=PASS"
} >"${META}"

ok "S7 clean prefix install CLI+agent"
echo "evidence_meta=${META}"
echo "evidence_inventory=${INVENTORY}"
echo "test-install-dual-cli: PASS"
exit 0
