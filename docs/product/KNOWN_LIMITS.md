# Known limitations

**On-disk SemVer:** read root `Cargo.toml` (do not hardcode).  
**Major line PRDs:** [versions/README.md](./versions/README.md) · current ship **[PRD-v2.md](./PRD-v2.md)** · next **[PRD-v3.md](./PRD-v3.md)**  
**Legacy:** `1.x` scaffold — [PRD-v1.md](./PRD-v1.md)

## What 2.x delivers

- No-args TTY `dsb` / `deepseek-build` → DeepSeek full-screen agent TUI (`deepseek-build-agent`)
- Base runtime vendored at `third_party/grok-build/` (ADR-0008)
- DeepSeek default models / API (`api.deepseek.com`, chat completions)
- Product chrome: DeepSeek Build name, DeepSeekNight (`#4D6BFE`), whale
- npm `postinstall` (2.0.3+) builds wrapper **and** agent when Rust/protoc available

## Honest residual → **3.0.0** (not “done in 2.x”)

| Topic | 2.x reality | Target |
|-------|-------------|--------|
| L1 snippet-safe edit on **Grok tool path** | Partial — strong on thin `dsb chat`/`run`; not full Spec 45 on default agent tools | [PRD-v3.md](./PRD-v3.md) |
| L1 permissions fail-closed on agent path | Partial / Grok capability modes | 3.0.0 |
| L2 prefix/epoch under **agent context stack** | Thin-path `dsb-context` proven; Grok stack not fully fused | 3.0.0 |
| Flash/Pro / repair as controlling loop | Thin path / partial | 3.0.0 |
| L3 worktree/subagent as product identity | Machine present; not fully productized | [PRD-v4.md](./PRD-v4.md) |

## Remaining limits (2.x ops)

### Install / packaging

- **First agent build is large/slow** (vendor tree + protoc/dotslash).
- **npm postinstall** needs **Rust** (+ protoc or dotslash) for TUI agent.
- Linux + macOS primary; Windows best-effort.

### Auth / network

- Requires DeepSeek API key for live turns (`DEEPSEEK_API_KEY` or credentials.json 0600).
- Some upstream strings may remain in deep code paths (product chrome is DeepSeek).
- **Agent routing (fixed 2026-08-07):** each `[model.deepseek-*]` **must** set  
  `base_url = "https://api.deepseek.com"`. Setting only `[endpoints].xai_api_base_url`  
  still sent traffic to **`cli-chat-proxy.grok.com`** (401). Product seed +  
  `ensure_product_agent_config` repair now inject model-level `base_url`.  
  Re-launch `dsb` once so existing homes get repaired.

### Tools / safety

- Thin path (`run` / `chat`) uses `dsb-tools` snippet + permission policy.
- Full-screen agent uses Grok native tools; **L1 fusion is 3.x**.
- Headless fail-closed on thin path: ask → deny unless TTY / explicit flags.
- **Pre-3.x live matrix** verifies agent tools against DeepSeek (read/list/grep/shell/edit)  
  when API key present — see [PRE_3X_TEST_MATRIX.md](./PRE_3X_TEST_MATRIX.md).  
  That is **capability smoke**, not Spec 45 heart fusion.

### Cache / cost

- Thin path: stable prefix epoch via `dsb-context`.
- Agent-stack L2 fusion: **3.x**.

### Dogfood / baseline

- Quick offline: `./scripts/smoke-dogfood.sh`
- **Pre-3.0.0 baseline:** `./scripts/test-pre3x-baseline.sh` (`--vendor` / `--live` / `--all`)
- Matrix SSOT: [PRE_3X_TEST_MATRIX.md](./PRE_3X_TEST_MATRIX.md)
- Evidence: `docs/product/evidence/`

## Not in product identity

- Multi-vendor “works equally on Claude/GPT” as identity  
- Gajae multi-stage planning harness as core loop  
- Claiming 1.x thin REPL is the product DoD  
- Claiming 2.x completed heart fusion (L1+L2 under Grok shell)  
