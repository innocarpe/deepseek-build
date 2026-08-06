# Ultragoal brief — after G0–G2 green

Use this as the **mission text** for ultragoal / long autonomous runs.

## Mission (M1 only)

Implement **M1 — Provider + cache + routing** for DeepSeek Build:

1. Cargo workspace + crates per [ADR 0004](../adr/0004-toolchain.md) (`dsb` binary).  
2. DeepSeek Chat Completions client per [ADR 0005](../adr/0005-deepseek-provider-contract.md).  
3. Stable prefix builder + golden tests per [spec 10](../specs/10-cache-contract.md).  
4. Tool-call repair + pairing per [spec 15](../specs/15-tool-call-repair.md).  
5. Flash/Pro routing per [spec 20](../specs/20-model-routing.md).  
6. Thinking/effort request shape per [spec 30](../specs/30-thinking-effort.md).  
7. Headless or thin CLI: user message → model → (optional **read-only** tools later) → response.  
8. Smoke: multi-turn + golden prefix + cache evidence protocol.

## Hard constraints

- Follow [HARNESS_PHILOSOPHY](../architecture/HARNESS_PHILOSOPHY.md) L1/L2/L3.  
- PRs only; [skills/pr-authoring](../../skills/pr-authoring/SKILL.md); Orca-level bodies.  
- **Do not** implement M2+ (snippet edit 45, shell, parallel 50, subagents 60) until those specs exist and G3+ green.  
- **Do not** add process-police CI.  
- **Do not** import Gajae multi-stage planning.  
- Update [GATES.md](../GATES.md) only when flipping gates with evidence.

## Success

- `dsb` (or `cargo run -p dsb-cli`) can complete a multi-turn chat against DeepSeek with key from env.  
- Automated tests for spec 10/15 (/20 routing table) pass.  
- Default model `deepseek-v4-flash`; Pro escalate works and is visible.  
