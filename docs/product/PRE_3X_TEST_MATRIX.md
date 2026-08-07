# Pre-3.0.0 test matrix (SSOT)

**Status:** Normative for “are we safe to start heart fusion (3.0.0)?”  
**Audience:** maintainers / agents  
**Related:** [PRD-v3.md](./PRD-v3.md) · [KNOWN_LIMITS.md](./KNOWN_LIMITS.md) · [GATES.md](../GATES.md) · ADR-0005 · ADR-0008

---

## 1. Why this exists

2.x shipped a **Grok-derived agent machine** plus a **DeepSeek product shell**. Before 3.0.0 fuses L1/L2 hearts into that shell we must answer **honestly**:

1. Do **product overlay crates** still pass offline?  
2. Does the **vendored Grok machine** still pass its offline/mock regression (curated, not “hope”)?  
3. Do **original Grok agent capabilities** work when inference is **DeepSeek API** (`api.deepseek.com`, chat completions) — not Grok proxy, not thin-path only?

Until this matrix has a dated evidence file under `docs/product/evidence/`, claims like “Grok features work on DeepSeek” are **unverified**.

---

## 2. Tiers (run order)

| Tier | Name | Network | Default gate | Script |
|------|------|---------|--------------|--------|
| **T0** | Product offline | none | **required** before any 3.x PR | `scripts/test-product-offline.sh` |
| **T1** | Grok vendor offline (tiered) | none | **light** before vendor edits; full only when needed | `scripts/test-grok-vendor-offline.sh` |
| **T2** | Product entry / install surface | none | **required** | included in T0 + agent bin checks |
| **T3** | Thin-path DeepSeek live | DeepSeek API | **required** when key present | `scripts/test-deepseek-live.sh` (phase thin) |
| **T4** | **Agent-path DeepSeek live** (Grok features) | DeepSeek API | **required** when key present | `scripts/test-deepseek-live.sh` (phase agent) |
| **T5** | Extended agent features | DeepSeek API | optional / scheduled | same script `--extended` |

Orchestrator:

```bash
# Everyday (recommended)
./scripts/test-pre3x-baseline.sh --live       # T0 + T2 + T3 + T4

# Offline product only
./scripts/test-pre3x-baseline.sh

# Vendor offline — tiered (disk-aware)
./scripts/test-pre3x-baseline.sh --vendor           # T1 light
./scripts/test-pre3x-baseline.sh --vendor-medium
./scripts/test-pre3x-baseline.sh --vendor-full      # HEAVY; clean target after
./scripts/test-pre3x-baseline.sh --all              # light vendor + live (not full)
```

**Disk:** vendor build artifacts live in `third_party/grok-build/target/` (gitignored).  
Cold `--vendor-full` can grow to **tens of GB**. After heavy runs:

```bash
rm -rf third_party/grok-build/target
```

**Not a process-police GHA suite.** Local/agent dogfood gate with written evidence.

---

## 3. T0 — Product offline (dsb-* workspace)

| ID | Check | Pass criteria |
|----|-------|---------------|
| T0.1 | SemVer | `./scripts/check-semver.sh` |
| T0.2 | Workspace tests | `cargo test --workspace` exit 0 |
| T0.3 | Dual bins | `deepseek-build` + `dsb` `--version` match workspace version |
| T0.4 | Help | both bins `--help` exit 0 |
| T0.5 | Agent config seed unit tests | `cargo test -p dsb-cli product_config` / repair tests |
| T0.6 | npm version match | `package.json` ↔ Cargo when present |

Coverage intent: provider client, snippet edit (Spec 45 thin), permissions, context epoch, agent loop thin, CLI dual names.

---

## 4. T1 — Grok vendor offline (tiered)

Full `cargo test` on the entire vendor workspace is **not** the everyday gate.

| Level | Crates | When | Disk |
|-------|--------|------|------|
| **light** (default) | `xai-grok-sampler`, `sampling-types`, `config`, `config-types` | everyday vendor signal; DeepSeek uses chat_completions sampler | small–medium |
| **medium** | + `xai-grok-tools`, `xai-grok-test-support` | tool impl regression | medium |
| **full** | + `xai-grok-shell --lib`, `test_sampling_client` | before deep vendor/runtime surgery | **large** (cold tens of GB) |

```bash
./scripts/test-grok-vendor-offline.sh           # light
./scripts/test-grok-vendor-offline.sh --medium
./scripts/test-grok-vendor-offline.sh --full    # then: rm -rf third_party/grok-build/target
```

| Explicitly **out of all T1 levels by default** | Reason |
|------------------------------------------------|--------|
| PTY fullscreen UI soak | Host-dependent |
| Leader death / soak / version-skew | Env-specific |
| Live xAI network | Wrong provider |
| Full workspace `cargo test` | Hours + huge disk |

Vendor **release build** remains `./scripts/build-grok-pager.sh check|release` (product agent binary), separate from T1.

---

## 5. T2 — Entry surface

| ID | Check | Pass criteria |
|----|-------|---------------|
| T2.1 | Agent binary present | `deepseek-build-agent` or `xai-grok-pager` on product path / `DEEPSEEK_BUILD_AGENT_BIN` |
| T2.2 | Product config seed | New home → `config.toml` has flash/pro, `chat_completions`, **`base_url = https://api.deepseek.com`**, theme deepseeknight |
| T2.3 | Repair path | Existing config missing `base_url` gets repair on next `ensure_product_agent_config` |
| T2.4 | models list | Agent `models` lists deepseek-v4-flash / pro as available when GROK_HOME=product home |

**Critical finding (2026-08-07):** seed used to set only `[endpoints].xai_api_base_url`. Agent still called **`cli-chat-proxy.grok.com`**. Model-level **`base_url`** is mandatory. Fixed in `agent_launch.rs` + T4 probes.

---

## 6. T3 — Thin-path DeepSeek live

Uses overlay crates (`dsb run` / `dsb chat`), **not** Grok agent loop.

| ID | Prompt / action | Pass |
|----|-----------------|------|
| T3.1 | `dsb run "Reply with exactly one word: pong"` | exit 0, body contains `pong` |
| T3.2 | Usage / model line | `model=deepseek-v4-flash` (or configured default) |
| T3.3 | Optional Pro | `dsb run --pro …` or model override if supported |

Auth: `DEEPSEEK_API_KEY` or `~/.deepseek-build/credentials.json` (0600). **Never log secrets.**

---

## 7. T4 — Agent-path DeepSeek live (Grok feature core)

Hermetic **workspace** + **GROK_HOME** with product DeepSeek models (`base_url` set).  
Binary: `deepseek-build-agent` (or installed alias). Headless: `-p` / `--single`.

| ID | Grok capability | How | Pass criteria |
|----|-----------------|-----|---------------|
| T4.0 | Config routes to DeepSeek | headless chat; stderr must **not** contain `cli-chat-proxy.grok.com` | exit 0; answer matches |
| T4.1 | Headless single-turn text | `-p "…pong…"` `--max-turns 2` | stdout contains `pong` |
| T4.2 | `read_file` | allowlist `--tools read_file` + fixture | content echoed / asserted |
| T4.3 | `list_dir` | `--tools list_dir` | fixture names appear |
| T4.4 | `grep` | `--tools grep` | marker line found |
| T4.5 | `run_terminal_cmd` | `--tools run_terminal_cmd` `--yolo` | known stdout token |
| T4.6 | `search_replace` | edit fixture + optional re-read | file on disk changed |
| T4.7 | Multi-turn tools | read then answer (reasoning_content path) | exit 0; correct content |
| T4.8 | Model switch | `-m deepseek-v4-pro` short prompt | exit 0 (or documented skip if quota) |

Safety defaults for automation:

- Hermetic temp cwd (never product repo write unless intentional)  
- `--yolo` only inside temp workspace  
- `--disallowed-tools` for web / Agent when not under test  
- Timeouts per case (script default 180–300s)

---

## 8. T5 — Extended agent features (optional)

These are **Grok product surface** items that may work but are **not** 3.0.0 blockers. Record pass/fail/skip.

| ID | Feature | Headless / notes |
|----|---------|------------------|
| T5.1 | Sessions resume (`-c` / `-r`) | multi-call |
| T5.2 | Subagent spawn | may need Agent tool + cost |
| T5.3 | Background shell / task output | kill_task / get_task_output |
| T5.4 | MCP list / call | only if MCP configured |
| T5.5 | Skills load | SKILL.md in fixture |
| T5.6 | Plan mode | if exposed headless |
| T5.7 | Worktree flag | `--worktree` smoke |
| T5.8 | Permissions deny | without yolo, deny rule blocks write |
| T5.9 | Streaming JSON output | `--output-format streaming-json` |
| T5.10 | Flash vs Pro quality | subjective / token budget |

---

## 9. Mapping to 3.0.0 (what this matrix does **not** prove)

| 3.0.0 P0 (PRD-v3) | Covered by pre-3.x matrix? |
|-------------------|----------------------------|
| L1 snippet-safe on **Grok tool path** | **No** — only that Grok `search_replace` works; Spec 45 fusion is 3.x tests |
| L1 permissions fail-closed on agent | Partial T5.8 only |
| L2 prefix/epoch under agent stack | **No** |
| Tool-call repair Spec 15 under agent | Partial via multi-turn tools only |
| Flash/Pro controlling loop | T4.8 smoke only |

This matrix is the **2.x reality baseline**. 3.0.0 adds **heart contract** tests that flip red → green.

---

## 10. Evidence protocol

After a full or partial run:

1. Write `docs/product/evidence/PRE3X_BASELINE_YYYY-MM-DD.md`  
2. Table of tier/ID → pass/fail/skip + short notes  
3. **Redact** API keys, full auth headers, home paths with secrets  
4. Link from KNOWN_LIMITS / PRD-v3 Notes when status changes  
5. If T4.0 fails (proxy routing), **do not** start 3.0.0 fusion work

### Failure classes

| Class | Meaning | Action |
|-------|---------|--------|
| **CONFIG** | wrong base_url / auth | fix seed/repair; re-run T4 |
| **PROVIDER** | DeepSeek 4xx/5xx / model id | ADR-0005 re-check |
| **AGENT** | tool loop / headless bug | vendor or product patch |
| **ENV** | missing binary / protoc / timeout | fix machine; skip with reason |
| **FLAKE** | intermittent | re-run 2×; quarantine ID |

---

## 11. Quick reference commands

```bash
# Offline product
./scripts/test-product-offline.sh

# Offline Grok curated
./scripts/test-grok-vendor-offline.sh

# Live DeepSeek (thin + agent core)
export DEEPSEEK_API_KEY=…   # or rely on credentials.json via script loader
./scripts/test-deepseek-live.sh

# Full pre-3.x
./scripts/test-pre3x-baseline.sh --all
```
