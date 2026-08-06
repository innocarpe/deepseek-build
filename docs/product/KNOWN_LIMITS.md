# Known limitations

**Product version:** `2.0.0` (Grok Build base)  
**Legacy:** `1.x` scaffold remains installable; see [REPLAN_2.0.md](./REPLAN_2.0.md).

## What 2.0.0 delivers

- No-args TTY `dsb` / `deepseek-build` → Grok-class full-screen agent (`deepseek-build-agent`)
- Base runtime vendored at `third_party/grok-build/` (ADR-0008)
- DeepSeek default models / API (`api.deepseek.com`, chat completions)
- L1/L2 minimums tested on overlay tools + prefix epoch; mapped under Grok capability modes

## Remaining limits

### Install / packaging

- **First agent release build is large/slow** (vendor tree + protoc/dotslash host tools).
- **npm postinstall** builds product wrapper from source when `cargo` is available; agent binary requires `./scripts/install.sh` (or pre-built agent) for full TUI.
- **Registry publish** remains **owner-gated** (ADR 0007) when 2FA/OTP required.
- Linux + macOS primary; Windows best-effort.

### Auth / network

- Requires DeepSeek API key for live turns (`DEEPSEEK_API_KEY` or credentials.json 0600).
- Upstream Grok UI strings may still appear inside the pager chrome (product wrapper/docs say DeepSeek Build).

### Tools / safety

- Thin path (`run` / `repl-legacy`) uses `dsb-tools` snippet + permission policy.
- Full-screen agent uses Grok native tools (SearchReplace/hashline/bash); capability modes apply.
- Headless fail-closed: ask → deny unless TTY / explicit allow flags.
- Subagents/worktrees: prefer Grok real mechanisms under agent; 1.x in-process shims are legacy.

### Cache / cost

- Thin path: stable prefix epoch via `dsb-context` (live `prefix_epoch=` line).
- Mid-session tool/skills changes start a new epoch (expected).

### Dogfood

- Offline: `./scripts/smoke-dogfood.sh`
- Evidence: `docs/product/evidence/`

## Not in product identity

- Multi-vendor “works equally on Claude/GPT” as identity  
- Gajae multi-stage planning harness as core loop  
- Claiming 1.x thin REPL is the product DoD  
