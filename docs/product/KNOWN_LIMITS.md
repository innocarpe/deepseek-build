# Known limitations

**Product version:** `1.x` scaffold line  
Honest limits. **`1.0.0` was tagged early** — see [REPLAN_2.0.md](./REPLAN_2.0.md).  
**Product target is `2.0.0`** (Grok Build–class agent entry).

## Product-identity gap (1.x)

- **`dsb` does not open a Grok Build–class full-screen coding agent.** 1.x is a thin REPL/scaffold.
- **Not built on the Grok Build open-source runtime** as the execution base (greenfield `dsb-*`).
- Claiming “Grok-class throughput” at 1.x overstates MVP parallel/bg/subagent shims.

## Install / packaging

- **npm postinstall** builds from source when `cargo` is available; there is no multi-arch prebuilt binary CDN yet.
- **Registry publish** of `@innocarpe/deepseek-build` is **owner-gated** (ADR 0007); agents never force publish.
- Linux + macOS are primary; Windows is best-effort (paths/permissions may differ).

## Auth / network

- Requires a valid DeepSeek API key for live chat/run.
- No multi-vendor provider identity in v1.

## Tools / safety

- `write` is create-only; overwrite goes through `read` + `edit`.
- Free-form whole-file edit without snippet is not the primary path.
- MCP **stdio live process manager** is minimal: static/mock catalogs and fingerprint/epoch rules ship; full multi-server lifecycle is still thin.
- Subagents are **in-process** heuristics (explore/implement); not full OS worktree fleets.
- Background bash is process-local (lost on CLI exit).

## Permissions

- Headless `run` maps ask → deny unless `--ask-permissions` on a TTY.
- Out-of-cwd write/delete cannot be elevated via allow-always grants.

## Cache / cost

- Mid-session tool schema or skills index changes start a new cache epoch (expected).
- Flash-first routing; Pro escalate is best-effort (404 falls back to Flash).

## Dogfood

- Automated offline smoke: `./scripts/smoke-dogfood.sh`.
- **Multi-day human dogfood** remains the owner’s judgment for release confidence (PRD-wave-D).

## Not in product identity

- Gajae multi-stage team harness as core loop
- Process-police CI (title/label fashion)
- Claiming **`1.0.0`** while Wave A/B incomplete (chain forbids)
