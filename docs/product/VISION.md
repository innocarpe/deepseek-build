# Vision

## One-liner

**DeepSeek Build** is a DeepSeek-native terminal coding agent that feels as fast as Grok Build, as cheap on long sessions as Reasonix, and as correctly tuned to DeepSeek V4 as Deep Code.

## North star

**Wall-clock task completion speed** under real multi-step coding work — not benchmark vanity, not plan-document volume.

Users should feel: *I asked for a change, and the tool made progress immediately*, with parallel exploration and implementation where safe.

## Product pillars

Normative detail: [HARNESS_PHILOSOPHY.md](../architecture/HARNESS_PHILOSOPHY.md).

1. **DeepSeek-first harness (L1 — Deep Code)**  
   Tool shapes (including **snippet-scoped edit**), skills as structured on-demand context, side-effect permissions, thinking/effort UX, and session surface follow DeepSeek-native contracts—not a generic multi-vendor tool zoo.

2. **Cache- and cost-aware loop (L2 — Reasonix + Deep Code)**  
   Byte-stable system/tool/memory **prefix**; dynamics on the turn tail; Flash-first / Pro escalate; tool-call repair. Cache is an invariant, not a late optimization.

3. **Parallel orchestration (L3 — Grok Build)**  
   Parallel tool calls, background shell, subagents, optional worktree isolation—**without** violating L1/L2 (worker cache law).

## Default models

Pinned wire IDs ([ADR 0005](../adr/0005-deepseek-provider-contract.md)):

- **`deepseek-v4-flash`** — default loop (explore, edit, tool churn)
- **`deepseek-v4-pro`** — escalated turns (hard design, review, tough bugs)

Routing: spec `20` / effort: spec `30`. UX: `/model` / `/preset` (Deep Code + Reasonix patterns).

## Success signals (qualitative, v0)

- Same task finishes with fewer serial waits than “single-thread agent” tools
- Long sessions stay affordable via cache hit rates users can see
- Thinking/effort knobs are first-class, not hidden env hacks
- Plan mode helps without trapping the agent in endless planning

## Roadmap board

Execution waves, SemVer bands, and ultragoal chain: **[MASTER_PLAN.md](./MASTER_PLAN.md)**.

## Design

Default UI must be **readable** (DeepSeek blue family), not Grok-style low-contrast black.
