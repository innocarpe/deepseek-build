# Pre-3.0.0 baseline — 2026-08-07

**Matrix:** [PRE_3X_TEST_MATRIX.md](../PRE_3X_TEST_MATRIX.md)  
**Product SemVer on disk:** `2.0.3`  
**Vendor SOURCE_REV:** `4d6d11372ab8f73026a78c45a7b7e7b1310eb39f`  
**Host:** macOS (local maintainer)

## Executive finding

Before this baseline, **agent-path Grok features had not been verified against DeepSeek API**.

Critical config bug found and fixed:

| Before | After |
|--------|--------|
| Seed set only `[endpoints].xai_api_base_url` | Each `[model.deepseek-*]` has **`base_url = "https://api.deepseek.com"`** |
| Headless agent → `cli-chat-proxy.grok.com` → **401** | Headless agent → `api.deepseek.com` → **OK** |

Code: `crates/dsb-cli/src/agent_launch.rs` (seed + repair). Unit tests for seed + repair.

## T0 — Product offline

| ID | Status | Notes |
|----|--------|-------|
| T0.1 SemVer | **PASS** | |
| T0.2 `cargo test --workspace` | **PASS** | all dsb-* unit/integration offline |
| T0.3 dual bins version | **PASS** | after rebuild to 2.0.3 (stale install was 2.0.2) |
| T0.4 help | **PASS** | |
| T0.5 config seed/repair tests | **PASS** | `base_url` asserted |
| T0.6 npm version match | **PASS** | |

Command: `./scripts/test-product-offline.sh`

## T2 — Entry

| ID | Status | Notes |
|----|--------|-------|
| T2.1 agent binary | **PASS** | `~/.deepseek-build/bin/deepseek-build-agent` |
| T2.2 seed | **PASS** | via unit tests |
| T2.3 repair | **PASS** | injects missing `base_url` |

## T3 — Thin-path DeepSeek live

| ID | Status | Notes |
|----|--------|-------|
| T3.1 `dsb run` pong | **PASS** | flash, thinking on |
| T3.2 model line | **PASS** | `deepseek-v4-flash` |

## T4 — Agent-path DeepSeek live (Grok tools)

Hermetic `GROK_HOME` + temp workspace. Binary: installed agent. Auth: product credentials (redacted).

| ID | Capability | Status | Notes |
|----|------------|--------|-------|
| T4.0 | Route not Grok proxy | **PASS** | no `cli-chat-proxy.grok.com` |
| T4.1 | Headless text | **PASS** | `pong` |
| T4.2 | `read_file` | **PASS** | fixture content |
| T4.3 | `list_dir` | **PASS** | entries listed |
| T4.4 | `grep` | **PASS** | token found |
| T4.5 | `run_terminal_cmd` | **PASS** | `shell-ok-99` |
| T4.6 | `search_replace` | **PASS** | disk verified `alpha-marker-99` |
| T4.7 | Multi-step read | **PASS** | nested file |
| T4.8 | `deepseek-v4-pro` | **PASS** | short pong |

Command: `./scripts/test-deepseek-live.sh`  
Duration: ~2.5 min for full T3+T4.

## T1 — Vendor offline (curated)

Running / partial at doc write time; re-check `_last_pre3x_vendor.tsv` and script log.

| ID | Crate | Status (this run) |
|----|-------|-------------------|
| T1.0 | pager-bin check | SKIP (already green earlier) / PASS when run |
| T1.1 | `xai-grok-sampler` | **PASS** (172+ tests) |
| T1.2 | `xai-grok-sampling-types` | **PASS** (301) |
| T1.3 | `xai-grok-tools` | **PASS** (~2900 lib) |
| T1.4 | `xai-grok-config` | **PASS** |
| T1.5 | `xai-grok-config-types` | **PASS** |
| T1.6 | `xai-grok-test-support` | **PASS** |
| T1.7 | `xai-grok-shell --lib` | *see follow-up* |
| T1.8 | sampling_client integration | *see follow-up* |

Command: `./scripts/test-grok-vendor-offline.sh`  
Note: first compile of vendor test graph is long (cold).

## Not proven (expected → 3.0.0)

- Spec 45 snippet contract on Grok `search_replace` path  
- Spec 90 permissions product matrix on agent  
- L2 prefix epoch under agent context assembly  
- Spec 15 repair as controlling loop  
- T5 remaining: sessions/subagent/MCP/skills/plan/worktree/quality (T5.1–T5.7, T5.10) not automated yet  

### T5 follow-up run (same day, `--extended`)

**Disk:** no vendor `target` rebuild (agent bin only, ~160MB). free still ~56GB.

| ID | Status | Notes |
|----|--------|-------|
| T5.8 permissions deny | **PASS** | no unauthorized write file |
| T5.9 streaming-json | **PASS** | NDJSON thought/text/end on DeepSeek |
| T5.1–T5.7, T5.10 | **SKIP** | not automated in harness v1 |

T3+T4 re-confirmed green in same invocation (~80s total live).

## Harness landed

| Path | Role |
|------|------|
| `docs/product/PRE_3X_TEST_MATRIX.md` | SSOT matrix |
| `scripts/test-pre3x-baseline.sh` | orchestrator |
| `scripts/test-product-offline.sh` | T0/T2 |
| `scripts/test-grok-vendor-offline.sh` | T1 |
| `scripts/test-deepseek-live.sh` | T3/T4/T5 |
| `scripts/lib/common.sh` | hermetic GROK_HOME, key load, redact |

## Honest product status (2026-08-07)

| Claim | Reality |
|-------|---------|
| Grok agent **core tools** work via DeepSeek API | **Yes** after `base_url` fix (T4.0–T4.8) |
| **All** Grok Build features work on DeepSeek | **Not proven** — sessions/subagent/MCP/skills/worktree (T5) not run |
| Zero bugs found | **No** — critical routing bug found (below); fixed in tree |
| L1/L2 hearts fused | **No** — still 3.0.0 work (KNOWN_LIMITS) |

### Bug found this baseline

| Bug | Impact | Status |
|-----|--------|--------|
| Product `config.toml` seed omitted model-level `base_url` | Agent defaulted to `cli-chat-proxy.grok.com` → **401** despite DeepSeek models listed | **Fixed** in `agent_launch.rs` seed + repair; unit-tested |

Existing homes without `base_url`: next `dsb` launch runs repair. Or re-run ensure / add `base_url` manually.

## Recommendation

| Gate | Verdict |
|------|---------|
| Everyday regression | `./scripts/test-pre3x-baseline.sh --live` (not vendor-full) |
| Start 3.0.0 heart fusion design | **OK** with T0+T4 green + base_url fix merged |
| Claim “all Grok features work on DeepSeek” | **No** — core tools yes; T5 not done |
| Ship without model `base_url` repair | **No** |

## Operator notes

1. Rebuild product bins after version bump (`cargo build -p dsb-cli --release`).  
2. Existing `~/.deepseek-build/config.toml` without `base_url`: launch `dsb` once after this fix so repair runs.  
3. Never commit credentials or raw API keys; evidence must redact.
