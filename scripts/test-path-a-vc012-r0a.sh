#!/usr/bin/env bash
# VC012 Path A R0A — conservative bounded public worktree dogfood + honesty.
#
# Proves (vision V3-WT / L3-WT-1/2 Path A bar) on public deepseek-build/dsb:
#   1. worktree-cli-surface: product/agent help + worktree list --json (dual CLI)
#   2. worktree-flag-forward: product --worktree/--worktree-ref cross process
#      boundary (exec) into agent argv via bounded stub agent (no live model)
#   3. worktree-opt-in-stamp: path_a_l3 worktree_product=opt_in + bare_dsb_session=single
#   4. worktree-headless-no-create: public -p --worktree=NAME creates no git worktree
#      (evidence-backed: git worktree list --porcelain before/after + wire)
#
# Explicit residual (process boundary):
#   Interactive TTY worktree *create* after exec is NOT asserted here — requires
#   a real TTY + vendor worktree machinery. Flag handoff is bounded by stub argv;
#   headless no-create is bounded by git porcelain on a disposable repo.
#
# Usage:
#   ./scripts/test-path-a-vc012-r0a.sh
#   ./scripts/test-path-a-vc012-r0a.sh --keep
#   ./scripts/test-path-a-vc012-r0a.sh --scenario worktree-flag-forward
#   ./scripts/test-path-a-vc012-r0a.sh --skip-build
#
# Exit 0 only when required scenarios pass with public-path evidence.
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
      sed -n '2,24p' "$0"
      exit 0
      ;;
    *) fail "unknown arg: $1" ;;
  esac
done

echo "=== test-path-a-vc012-r0a (Path A L3 worktree dogfood) ==="
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
log "dsb=${DSB_BIN:-missing (optional dual-CLI path)}"

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
  if [[ -x "${CLI_DIR}/dsb" ]]; then
    DSB_BIN="${CLI_DIR}/dsb"
  fi
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

# --- shared: hermetic home config (agent binary installed separately) ---
write_hermetic_config() {
  local HOME_DIR="$1"
  local BASE_URL="${2:-}"
  mkdir -p "${HOME_DIR}/bin"
  if [[ -n "${BASE_URL}" ]]; then
    cat >"${HOME_DIR}/config.toml" <<EOF
# Hermetic Path A VC012 R0A home — safe to delete with workdir.

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
  else
    cat >"${HOME_DIR}/config.toml" <<'EOF'
# Hermetic Path A VC012 R0A home (CLI-only / stub scenarios).

[models]
default = "deepseek-v4-flash"

[model.deepseek-v4-flash]
model = "deepseek-v4-flash"
name = "DeepSeek V4 Flash"
context_window = 128000
api_backend = "chat_completions"
base_url = "https://api.deepseek.com"
api_key = "sk-scripted-path-a-r0"
env_key = "DEEPSEEK_API_KEY"

[ui]
theme = "deepseeknight"
yolo = true
permission_mode = "always-approve"

[subagents]
enabled = true
EOF
  fi
  chmod 600 "${HOME_DIR}/config.toml"
  python3 - <<PY
import json
from pathlib import Path
p = Path("${HOME_DIR}") / "credentials.json"
p.write_text(json.dumps({"api_key": "sk-scripted-path-a-r0"}), encoding="utf-8")
p.chmod(0o600)
PY
}

# Real product agent binary under hermetic home.
setup_hermetic_home() {
  local HOME_DIR="$1"
  local BASE_URL="${2:-}"
  write_hermetic_config "${HOME_DIR}" "${BASE_URL}"
  cp -f "${AGENT_RESOLVED}" "${HOME_DIR}/bin/deepseek-build-agent"
  chmod +x "${HOME_DIR}/bin/deepseek-build-agent"
  if ! agent_runs "${HOME_DIR}/bin/deepseek-build-agent"; then
    fail "hermetic agent copy does not run --version"
  fi
}

# Bounded stub agent: records argv after product CLI exec (process-boundary proof).
# Does not create git worktrees; does not call models.
install_stub_agent() {
  local HOME_DIR="$1"
  local ARGV_FILE="${HOME_DIR}/agent_argv.txt"
  cat >"${HOME_DIR}/bin/deepseek-build-agent" <<STUB
#!/usr/bin/env bash
# VC012 bounded stub agent — argv capture only (no worktree create, no model).
set -euo pipefail
ARGV_OUT="${ARGV_FILE}"
if [[ "\${1:-}" == "--version" ]]; then
  echo "vc012-stub-agent 0.0.0"
  exit 0
fi
if [[ "\${1:-}" == "--help" || "\${1:-}" == "-h" ]]; then
  cat <<'HELP'
vc012 stub agent (records argv; does not create worktrees)
  -w, --worktree [<NAME>]
  --worktree-ref <REF>
  worktree     Manage git worktrees (stub: no-op)
HELP
  exit 0
fi
{
  printf 'argc=%s\n' "\$#"
  i=0
  for a in "\$@"; do
    i=\$((i + 1))
    printf 'arg[%s]=%s\n' "\$i" "\$a"
  done
  printf 'raw:'
  for a in "\$@"; do
    printf ' %q' "\$a"
  done
  printf '\n'
} >"\${ARGV_OUT}"
echo STUB_AGENT_OK
exit 0
STUB
  chmod +x "${HOME_DIR}/bin/deepseek-build-agent"
}

start_scripted_server() {
  local WORK="$1"
  local SCENARIO="$2"
  local WS="$3"
  local WIRE="${WORK}/wire.jsonl"
  local SERVER_LOG="${WORK}/server.log"
  python3 "${SERVER_PY}" \
    --host 127.0.0.1 \
    --port 0 \
    --wire "${WIRE}" \
    --scenario "${SCENARIO}" \
    --liveness-dir "${WS}" \
    --final-text "vc012-${SCENARIO}-done" \
    >"${WORK}/server.stdout" 2>"${SERVER_LOG}" &
  echo $!
}

wait_server_ready() {
  local WORK="$1"
  local SERVER_PID="$2"
  local SCENARIO="$3"
  local READY_LINE=""
  for _ in $(seq 1 50); do
    if [[ -s "${WORK}/server.stdout" ]]; then
      READY_LINE="$(head -1 "${WORK}/server.stdout" || true)"
      if [[ "${READY_LINE}" == READY\ * ]]; then
        break
      fi
    fi
    if ! kill -0 "${SERVER_PID}" 2>/dev/null; then
      cat "${WORK}/server.log" >&2 || true
      fail "scripted server exited early (${SCENARIO})"
    fi
    sleep 0.1
  done
  if [[ "${READY_LINE}" != READY\ * ]]; then
    cat "${WORK}/server.log" >&2 || true
    fail "scripted server did not print READY (${SCENARIO})"
  fi
  echo "${READY_LINE#READY }"
}

write_meta_header() {
  local META="$1"
  local HOME_DIR="$2"
  local SCENARIO="$3"
  local EXTRA="${4:-}"
  {
    echo "git_sha=$(git rev-parse HEAD 2>/dev/null || echo unknown)"
    echo "cli=${CLI}"
    echo "dsb=${DSB_BIN:-}"
    echo "agent_resolved=${AGENT_RESOLVED}"
    echo "DEEPSEEK_BUILD_AGENT_BIN_unset=yes"
    echo "DEEPSEEK_BUILD_HOME=${HOME_DIR}"
    echo "scenario=${SCENARIO}"
    if [[ -n "${EXTRA}" ]]; then
      printf '%s\n' "${EXTRA}"
    fi
  } >"${META}"
}

# --- scenario: worktree-cli-surface (no model) ---
run_worktree_cli_surface() {
  local SCENARIO="worktree-cli-surface"
  local WORK HOME_DIR REPO
  WORK="$(mktemp -d "${TMPDIR:-/tmp}/dsb-vc012-r0a.XXXXXX")"
  HOME_DIR="${WORK}/product-home"
  REPO="${WORK}/repo"
  mkdir -p "${HOME_DIR}" "${REPO}"

  cleanup_scenario() {
    if [[ "${KEEP}" -eq 0 ]]; then
      rm -rf "${WORK}"
    else
      log "kept workdir=${WORK}"
    fi
  }
  trap cleanup_scenario RETURN

  # Disposable git repo for --repo list
  git -C "${REPO}" init -q
  git -C "${REPO}" config user.email "vc012@example.com"
  git -C "${REPO}" config user.name "vc012"
  printf 'marker\n' >"${REPO}/marker.txt"
  git -C "${REPO}" add marker.txt
  git -C "${REPO}" commit -qm "vc012 init"

  setup_hermetic_home "${HOME_DIR}"
  export DEEPSEEK_BUILD_HOME="${HOME_DIR}"
  export GROK_HOME="${HOME_DIR}"
  export CI=1
  export NO_COLOR=1
  unset DEEPSEEK_BUILD_AGENT_BIN || true

  local PRODUCT_HELP HELP_OUT WT_HELP LIST_OUT DSB_LIST="" HELP_EC=0 WT_EC=0 LIST_EC=0 DSB_EC=0 PRODUCT_HELP_EC=0
  set +e
  # Product CLI should document top-level --worktree (VC012 forward).
  PRODUCT_HELP="$(run_to 15 "${CLI}" --help 2>&1)"
  PRODUCT_HELP_EC=$?
  # Product clap intercepts `agent --help`; forward help to the agent binary with `--`.
  HELP_OUT="$(run_to 30 "${CLI}" agent -- --help 2>&1)"
  HELP_EC=$?
  WT_HELP="$(run_to 30 "${CLI}" agent worktree --help 2>&1)"
  WT_EC=$?
  LIST_OUT="$(run_to 60 "${CLI}" agent worktree list --json --repo "${REPO}" 2>&1)"
  LIST_EC=$?
  if [[ -n "${DSB_BIN}" && -x "${DSB_BIN}" ]]; then
    DSB_LIST="$(run_to 60 "${DSB_BIN}" agent worktree list --json --repo "${REPO}" 2>&1)"
    DSB_EC=$?
  fi
  set -e

  local EVIDENCE_META="${OUT_DIR}/PATH_A_R0_VC012_${SCENARIO}_META_last.txt"
  write_meta_header "${EVIDENCE_META}" "${HOME_DIR}" "${SCENARIO}" \
    "product_help_exit=${PRODUCT_HELP_EC}
help_exit=${HELP_EC}
help_cmd=deepseek-build agent -- --help
worktree_help_exit=${WT_EC}
list_exit=${LIST_EC}
dsb_list_exit=${DSB_EC}
repo=${REPO}"
  {
    echo "product_help_worktree_lines<<EOF"
    printf '%s\n' "${PRODUCT_HELP}" | rg -n 'worktree|Worktree' || true
    echo "EOF"
    echo "agent_help_worktree_lines<<EOF"
    printf '%s\n' "${HELP_OUT}" | rg -n 'worktree|Worktree' || true
    echo "EOF"
    echo "worktree_help_tail<<EOF"
    printf '%s\n' "${WT_HELP}" | redact_stream | tail -40
    echo "EOF"
    echo "list_out<<EOF"
    printf '%s\n' "${LIST_OUT}" | redact_stream | tail -20
    echo "EOF"
    echo "dsb_list_out<<EOF"
    printf '%s\n' "${DSB_LIST}" | redact_stream | tail -20
    echo "EOF"
  } >>"${EVIDENCE_META}"

  local FAIL=0
  if ! printf '%s\n' "${PRODUCT_HELP}" | rg -qi 'worktree'; then
    warn "${SCENARIO}: product deepseek-build --help missing --worktree"
    FAIL=1
  fi
  if ! printf '%s\n' "${HELP_OUT}" | rg -qi 'worktree'; then
    warn "${SCENARIO}: forwarded agent -- --help missing worktree"
    FAIL=1
  fi
  if [[ ${WT_EC} -ne 0 ]] && ! printf '%s\n' "${WT_HELP}" | rg -qi 'worktree|Usage|list'; then
    warn "${SCENARIO}: worktree --help failed (ec=${WT_EC})"
    FAIL=1
  fi
  if ! printf '%s\n' "${WT_HELP}" | rg -qi 'list|Manage git worktrees|worktree'; then
    warn "${SCENARIO}: worktree --help body unexpected"
    FAIL=1
  fi
  if [[ ${LIST_EC} -ne 0 ]]; then
    warn "${SCENARIO}: worktree list --json failed ec=${LIST_EC}"
    FAIL=1
  fi
  # JSON array (empty OK) or empty-ish product message; reject hard crashes.
  if ! printf '%s\n' "${LIST_OUT}" | rg -q '\[|\]|worktree|No worktrees'; then
    # Accept pure "[]"
    if [[ "$(printf '%s' "${LIST_OUT}" | tr -d '[:space:]')" != "[]" ]]; then
      warn "${SCENARIO}: list output not JSON/empty-worktrees shaped"
      FAIL=1
    fi
  fi
  if [[ -n "${DSB_BIN}" ]]; then
    if [[ ${DSB_EC} -ne 0 ]]; then
      warn "${SCENARIO}: dsb agent worktree list failed ec=${DSB_EC}"
      FAIL=1
    fi
  else
    log "${SCENARIO}: dsb binary missing — dual-CLI path SKIP (deepseek-build still required)"
  fi

  if [[ "${FAIL}" -ne 0 ]]; then
    warn "${SCENARIO}: FAIL (meta=${EVIDENCE_META})"
    return 1
  fi
  ok "${SCENARIO}: PASS"
  return 0
}

# --- scenario: worktree-opt-in-stamp (scripted short turn) ---
run_worktree_opt_in_stamp() {
  local SCENARIO="worktree-opt-in-stamp"
  local FIXTURE_SCENARIO="worker-cache-stamp"
  local WORK HOME_DIR WS WIRE SERVER_PID=""
  WORK="$(mktemp -d "${TMPDIR:-/tmp}/dsb-vc012-r0a.XXXXXX")"
  HOME_DIR="${WORK}/product-home"
  WS="${WORK}/ws"
  WIRE="${WORK}/wire.jsonl"
  mkdir -p "${HOME_DIR}" "${WS}"
  printf 'stamp-ws-ready\n' >"${WS}/marker.txt"

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

  SERVER_PID="$(start_scripted_server "${WORK}" "${FIXTURE_SCENARIO}" "${WS}")"
  local HOSTPORT BASE_URL
  HOSTPORT="$(wait_server_ready "${WORK}" "${SERVER_PID}" "${SCENARIO}")"
  BASE_URL="http://${HOSTPORT}"
  log "${SCENARIO}: scripted_base_url=${BASE_URL}"

  setup_hermetic_home "${HOME_DIR}" "${BASE_URL}"
  export DEEPSEEK_BUILD_HOME="${HOME_DIR}"
  export DEEPSEEK_API_KEY="sk-scripted-path-a-r0"
  export GROK_HOME="${HOME_DIR}"
  export CI=1
  export NO_COLOR=1
  unset DEEPSEEK_BUILD_AGENT_BIN || true

  local AGENT_OUT EC
  set +e
  AGENT_OUT="$(
    run_to 180 "${CLI}" agent \
      -p "Reply with the scripted final token and stop." \
      --cwd "${WS}" \
      --output-format plain \
      --max-turns 6 \
      --yolo \
      --disallowed-tools "web_search,web_fetch" \
      2>&1
  )"
  EC=$?
  set -e

  local EVIDENCE_WIRE="${OUT_DIR}/PATH_A_R0_VC012_${SCENARIO}_WIRE_last.jsonl"
  local EVIDENCE_META="${OUT_DIR}/PATH_A_R0_VC012_${SCENARIO}_META_last.txt"
  if [[ -f "${WIRE}" ]]; then
    cp "${WIRE}" "${EVIDENCE_WIRE}"
  fi
  local L3_STAMP="${HOME_DIR}/path_a_l3.txt"
  if [[ -f "${L3_STAMP}" ]]; then
    cp "${L3_STAMP}" "${OUT_DIR}/PATH_A_L3_last.txt"
    cp "${L3_STAMP}" "${OUT_DIR}/PATH_A_R0_VC012_L3_last.txt"
  fi

  write_meta_header "${EVIDENCE_META}" "${HOME_DIR}" "${SCENARIO}" \
    "scripted_base_url=${BASE_URL}
agent_exit=${EC}
wire=${EVIDENCE_WIRE}
fixture_scenario=${FIXTURE_SCENARIO}"
  {
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
    echo "agent_out_tail<<EOF"
    printf '%s\n' "${AGENT_OUT}" | redact_stream | tail -40
    echo "EOF"
  } >>"${EVIDENCE_META}"

  local FAIL=0
  if [[ ! -s "${WIRE}" ]]; then
    warn "${SCENARIO}: empty wire"
    FAIL=1
  fi
  if [[ ! -f "${L3_STAMP}" ]]; then
    warn "${SCENARIO}: path_a_l3.txt missing under public DEEPSEEK_BUILD_HOME"
    FAIL=1
  else
    if ! rg -q 'worktree_product=opt_in' "${L3_STAMP}"; then
      warn "${SCENARIO}: missing worktree_product=opt_in"
      FAIL=1
    fi
    if ! rg -q 'bare_dsb_session=single' "${L3_STAMP}"; then
      warn "${SCENARIO}: missing bare_dsb_session=single"
      FAIL=1
    fi
    if ! rg -q 'worker_epochs_match=true' "${L3_STAMP}"; then
      warn "${SCENARIO}: worker_epochs_match!=true"
      FAIL=1
    fi
    if ! rg -q 'subagents_enabled_in_config=true' "${L3_STAMP}"; then
      warn "${SCENARIO}: subagents_enabled_in_config!=true"
      FAIL=1
    fi
  fi
  if ! printf '%s\n' "${AGENT_OUT}" | rg -q 'worker-cache-stamp-ok'; then
    warn "${SCENARIO}: missing worker-cache-stamp-ok in agent output (scripted turn)"
    FAIL=1
  fi

  if [[ "${FAIL}" -ne 0 ]]; then
    warn "${SCENARIO}: FAIL (meta=${EVIDENCE_META})"
    return 1
  fi
  ok "${SCENARIO}: PASS"
  return 0
}

# --- scenario: worktree-headless-no-create ---
run_worktree_headless_no_create() {
  local SCENARIO="worktree-headless-no-create"
  local FIXTURE_SCENARIO="worker-cache-stamp"
  local WT_NAME="vc012-headless-dogfood"
  local WORK HOME_DIR REPO WS WIRE SERVER_PID=""
  WORK="$(mktemp -d "${TMPDIR:-/tmp}/dsb-vc012-r0a.XXXXXX")"
  HOME_DIR="${WORK}/product-home"
  REPO="${WORK}/repo"
  WS="${REPO}"
  WIRE="${WORK}/wire.jsonl"
  mkdir -p "${HOME_DIR}" "${REPO}"

  cleanup_scenario() {
    if [[ -n "${SERVER_PID:-}" ]]; then
      kill "${SERVER_PID}" 2>/dev/null || true
      wait "${SERVER_PID}" 2>/dev/null || true
    fi
    # Best-effort prune any accidental worktrees under this repo
    if [[ -d "${REPO}/.git" ]]; then
      while IFS= read -r line; do
        case "${line}" in
          worktree\ *)
            local p="${line#worktree }"
            if [[ "${p}" != "${REPO}" && -d "${p}" ]]; then
              git -C "${REPO}" worktree remove --force "${p}" 2>/dev/null || true
            fi
            ;;
        esac
      done < <(git -C "${REPO}" worktree list --porcelain 2>/dev/null || true)
    fi
    if [[ "${KEEP}" -eq 0 ]]; then
      rm -rf "${WORK}"
    else
      log "kept workdir=${WORK}"
    fi
  }
  trap cleanup_scenario RETURN

  git -C "${REPO}" init -q
  git -C "${REPO}" config user.email "vc012@example.com"
  git -C "${REPO}" config user.name "vc012"
  printf 'headless-ws-ready\n' >"${REPO}/marker.txt"
  git -C "${REPO}" add marker.txt
  git -C "${REPO}" commit -qm "vc012 headless init"

  local BEFORE_LIST AFTER_LIST
  BEFORE_LIST="$(git -C "${REPO}" worktree list --porcelain 2>/dev/null || true)"

  SERVER_PID="$(start_scripted_server "${WORK}" "${FIXTURE_SCENARIO}" "${WS}")"
  local HOSTPORT BASE_URL
  HOSTPORT="$(wait_server_ready "${WORK}" "${SERVER_PID}" "${SCENARIO}")"
  BASE_URL="http://${HOSTPORT}"
  log "${SCENARIO}: scripted_base_url=${BASE_URL}"

  setup_hermetic_home "${HOME_DIR}" "${BASE_URL}"
  export DEEPSEEK_BUILD_HOME="${HOME_DIR}"
  export DEEPSEEK_API_KEY="sk-scripted-path-a-r0"
  export GROK_HOME="${HOME_DIR}"
  export CI=1
  export NO_COLOR=1
  unset DEEPSEEK_BUILD_AGENT_BIN || true

  local AGENT_OUT EC
  set +e
  # Product top-level --worktree (forwarded) + headless -p (vendor ignores create).
  AGENT_OUT="$(
    run_to 180 "${CLI}" \
      --worktree "${WT_NAME}" \
      agent \
      -p "Reply with the scripted final token and stop." \
      --cwd "${REPO}" \
      --output-format plain \
      --max-turns 6 \
      --yolo \
      --disallowed-tools "web_search,web_fetch" \
      2>&1
  )"
  EC=$?
  set -e

  AFTER_LIST="$(git -C "${REPO}" worktree list --porcelain 2>/dev/null || true)"

  local EVIDENCE_WIRE="${OUT_DIR}/PATH_A_R0_VC012_${SCENARIO}_WIRE_last.jsonl"
  local EVIDENCE_META="${OUT_DIR}/PATH_A_R0_VC012_${SCENARIO}_META_last.txt"
  if [[ -f "${WIRE}" ]]; then
    cp "${WIRE}" "${EVIDENCE_WIRE}"
  fi
  local L3_STAMP="${HOME_DIR}/path_a_l3.txt"
  if [[ -f "${L3_STAMP}" ]]; then
    cp "${L3_STAMP}" "${OUT_DIR}/PATH_A_L3_last.txt"
    cp "${L3_STAMP}" "${OUT_DIR}/PATH_A_R0_VC012_L3_last.txt"
  fi

  write_meta_header "${EVIDENCE_META}" "${HOME_DIR}" "${SCENARIO}" \
    "scripted_base_url=${BASE_URL}
agent_exit=${EC}
wire=${EVIDENCE_WIRE}
worktree_flag_name=${WT_NAME}
worktree_flag_source=product_top_level
fixture_scenario=${FIXTURE_SCENARIO}
repo=${REPO}
claim_scope=headless_p_plus_product_worktree_no_git_worktree_create
process_boundary_residual=interactive_tty_worktree_create_not_asserted"
  {
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
    echo "worktree_list_before<<EOF"
    printf '%s\n' "${BEFORE_LIST}"
    echo "EOF"
    echo "worktree_list_after<<EOF"
    printf '%s\n' "${AFTER_LIST}"
    echo "EOF"
    echo "agent_out_tail<<EOF"
    printf '%s\n' "${AGENT_OUT}" | redact_stream | tail -40
    echo "EOF"
  } >>"${EVIDENCE_META}"

  local FAIL=0
  if [[ ! -s "${WIRE}" ]]; then
    warn "${SCENARIO}: empty wire (headless turn did not reach model)"
    FAIL=1
  fi
  if [[ ! -f "${L3_STAMP}" ]]; then
    warn "${SCENARIO}: path_a_l3.txt missing"
    FAIL=1
  elif ! rg -q 'worktree_product=opt_in' "${L3_STAMP}"; then
    warn "${SCENARIO}: stamp missing worktree_product=opt_in"
    FAIL=1
  fi
  if ! printf '%s\n' "${AGENT_OUT}" | rg -q 'worker-cache-stamp-ok'; then
    warn "${SCENARIO}: missing worker-cache-stamp-ok (turn may have failed)"
    FAIL=1
  fi
  # Headless honesty: no new worktree path and no name leak (evidence-backed).
  local BEFORE_COUNT AFTER_COUNT
  BEFORE_COUNT="$(printf '%s\n' "${BEFORE_LIST}" | rg -c '^worktree ' || true)"
  AFTER_COUNT="$(printf '%s\n' "${AFTER_LIST}" | rg -c '^worktree ' || true)"
  BEFORE_COUNT="${BEFORE_COUNT:-0}"
  AFTER_COUNT="${AFTER_COUNT:-0}"
  {
    echo "worktree_count_before=${BEFORE_COUNT}"
    echo "worktree_count_after=${AFTER_COUNT}"
    echo "headless_no_create_assert=git_porcelain_count_and_name"
  } >>"${EVIDENCE_META}"
  if [[ "${AFTER_COUNT}" -gt "${BEFORE_COUNT}" ]]; then
    warn "${SCENARIO}: git worktree count increased (${BEFORE_COUNT} -> ${AFTER_COUNT}) under headless -p --worktree"
    FAIL=1
  fi
  if printf '%s\n' "${AFTER_LIST}" | rg -q "${WT_NAME}"; then
    warn "${SCENARIO}: worktree name ${WT_NAME} appears in git worktree list after headless run"
    FAIL=1
  fi
  # Also scan common sibling paths for the named worktree.
  if find "${WORK}" -maxdepth 3 -type d -name "*${WT_NAME}*" 2>/dev/null | rg -q .; then
    warn "${SCENARIO}: found directory matching ${WT_NAME} under workdir"
    FAIL=1
  fi
  # Before/after porcelain must be identical for a tight no-create claim.
  if [[ "${BEFORE_LIST}" != "${AFTER_LIST}" ]]; then
    warn "${SCENARIO}: git worktree porcelain changed under headless -p --worktree"
    FAIL=1
  fi

  if [[ "${FAIL}" -ne 0 ]]; then
    warn "${SCENARIO}: FAIL (meta=${EVIDENCE_META})"
    return 1
  fi
  ok "${SCENARIO}: PASS"
  return 0
}

# --- scenario: worktree-flag-forward (bounded stub; process-boundary argv) ---
run_worktree_flag_forward() {
  local SCENARIO="worktree-flag-forward"
  local WT_NAME="vc012-bound-forward"
  local WT_REF="HEAD"
  local WORK HOME_DIR
  WORK="$(mktemp -d "${TMPDIR:-/tmp}/dsb-vc012-r0a.XXXXXX")"
  HOME_DIR="${WORK}/product-home"
  mkdir -p "${HOME_DIR}"

  cleanup_scenario() {
    if [[ "${KEEP}" -eq 0 ]]; then
      rm -rf "${WORK}"
    else
      log "kept workdir=${WORK}"
    fi
  }
  trap cleanup_scenario RETURN

  write_hermetic_config "${HOME_DIR}"
  install_stub_agent "${HOME_DIR}"

  export DEEPSEEK_BUILD_HOME="${HOME_DIR}"
  export GROK_HOME="${HOME_DIR}"
  export CI=1
  export NO_COLOR=1
  unset DEEPSEEK_BUILD_AGENT_BIN || true

  local AGENT_OUT EC
  set +e
  # Product top-level flags must appear in stub argv after exec_agent process boundary.
  AGENT_OUT="$(
    run_to 30 "${CLI}" \
      --worktree "${WT_NAME}" \
      --worktree-ref "${WT_REF}" \
      agent \
      -p "stub-only-no-model" \
      --max-turns 1 \
      2>&1
  )"
  EC=$?
  set -e

  local ARGV_FILE="${HOME_DIR}/agent_argv.txt"
  local L3_STAMP="${HOME_DIR}/path_a_l3.txt"
  local EVIDENCE_META="${OUT_DIR}/PATH_A_R0_VC012_${SCENARIO}_META_last.txt"
  if [[ -f "${L3_STAMP}" ]]; then
    cp "${L3_STAMP}" "${OUT_DIR}/PATH_A_L3_last.txt"
    cp "${L3_STAMP}" "${OUT_DIR}/PATH_A_R0_VC012_L3_last.txt"
  fi
  if [[ -f "${ARGV_FILE}" ]]; then
    cp "${ARGV_FILE}" "${OUT_DIR}/PATH_A_R0_VC012_${SCENARIO}_ARGV_last.txt"
  fi

  write_meta_header "${EVIDENCE_META}" "${HOME_DIR}" "${SCENARIO}" \
    "agent_exit=${EC}
worktree_flag_name=${WT_NAME}
worktree_ref=${WT_REF}
worktree_flag_source=product_top_level
claim_scope=product_flag_forward_stub_argv_after_exec
process_boundary_residual=interactive_tty_worktree_create_not_asserted
stub_agent=yes"
  {
    if [[ -f "${L3_STAMP}" ]]; then
      echo "path_a_l3_stamp=present"
      # shellcheck disable=SC2002
      cat "${L3_STAMP}" | sed 's/^/l3_/'
    else
      echo "path_a_l3_stamp=missing"
    fi
    echo "stub_argv<<EOF"
    if [[ -f "${ARGV_FILE}" ]]; then
      cat "${ARGV_FILE}"
    else
      echo "MISSING"
    fi
    echo "EOF"
    echo "agent_out_tail<<EOF"
    printf '%s\n' "${AGENT_OUT}" | redact_stream | tail -20
    echo "EOF"
  } >>"${EVIDENCE_META}"

  local FAIL=0
  if [[ ! -f "${ARGV_FILE}" ]]; then
    warn "${SCENARIO}: stub did not write agent_argv.txt (exec may not have reached agent)"
    FAIL=1
  else
    if ! rg -q -- "--worktree" "${ARGV_FILE}"; then
      warn "${SCENARIO}: product --worktree missing from agent argv after exec"
      FAIL=1
    fi
    if ! rg -q -- "${WT_NAME}" "${ARGV_FILE}"; then
      warn "${SCENARIO}: worktree name ${WT_NAME} missing from agent argv"
      FAIL=1
    fi
    if ! rg -q -- "--worktree-ref" "${ARGV_FILE}"; then
      warn "${SCENARIO}: product --worktree-ref missing from agent argv after exec"
      FAIL=1
    fi
    if ! rg -q -- "${WT_REF}" "${ARGV_FILE}"; then
      warn "${SCENARIO}: worktree-ref value ${WT_REF} missing from agent argv"
      FAIL=1
    fi
  fi
  if ! printf '%s\n' "${AGENT_OUT}" | rg -q 'STUB_AGENT_OK'; then
    warn "${SCENARIO}: missing STUB_AGENT_OK (stub not executed)"
    FAIL=1
  fi
  if [[ ! -f "${L3_STAMP}" ]]; then
    warn "${SCENARIO}: path_a_l3.txt missing (stamps should run before exec)"
    FAIL=1
  elif ! rg -q 'worktree_product=opt_in' "${L3_STAMP}"; then
    warn "${SCENARIO}: stamp missing worktree_product=opt_in"
    FAIL=1
  elif ! rg -q 'bare_dsb_session=single' "${L3_STAMP}"; then
    warn "${SCENARIO}: stamp missing bare_dsb_session=single"
    FAIL=1
  fi

  if [[ "${FAIL}" -ne 0 ]]; then
    warn "${SCENARIO}: FAIL (meta=${EVIDENCE_META})"
    return 1
  fi
  ok "${SCENARIO}: PASS"
  return 0
}

FAILED=0
SCENARIOS=(worktree-cli-surface worktree-flag-forward worktree-opt-in-stamp worktree-headless-no-create)
if [[ -n "${ONLY_SCENARIO}" ]]; then
  SCENARIOS=("${ONLY_SCENARIO}")
fi

for sc in "${SCENARIOS[@]}"; do
  case "${sc}" in
    worktree-cli-surface)
      if ! run_worktree_cli_surface; then FAILED=1; fi
      ;;
    worktree-flag-forward)
      if ! run_worktree_flag_forward; then FAILED=1; fi
      ;;
    worktree-opt-in-stamp)
      if ! run_worktree_opt_in_stamp; then FAILED=1; fi
      ;;
    worktree-headless-no-create)
      if ! run_worktree_headless_no_create; then FAILED=1; fi
      ;;
    *)
      fail "unknown scenario ${sc}"
      ;;
  esac
done

if [[ "${FAILED}" -ne 0 ]]; then
  fail "VC012 Path A R0A FAILED one or more scenarios"
fi
ok "VC012 Path A R0A all scenarios PASS"
exit 0
