# Vision

## One-liner

**DeepSeek Build** is a DeepSeek-native terminal coding agent that feels as fast as Grok Build, as cheap on long sessions as Reasonix, and as correctly tuned to DeepSeek V4 as Deep Code.

## North star

**Wall-clock task completion speed** under real multi-step coding work — not benchmark vanity, not plan-document volume.

Users should feel: *I asked for a change, and the tool made progress immediately*, with parallel exploration and implementation where safe.

## Product pillars

1. **DeepSeek-first harness**  
   Thinking mode, reasoning effort, Skills, MCP, permissions, and plan mode follow the official DeepSeek-oriented CLI surface ([Deep Code](https://github.com/lessweb/deepcode-cli)) and DeepSeek API contracts.

2. **Cache- and cost-aware loop**  
   Stable system/tool/memory prefix so DeepSeek automatic prefix/KV cache stays warm (Reasonix lesson). Flash-first; Pro on demand.

3. **Parallel orchestration**  
   Parallel tool calls, background shell, subagents, optional worktree isolation (Grok Build lesson). Progress over ceremony.

## Default models

- **DeepSeek-V4-Flash** — default loop (explore, edit, tool churn)
- **DeepSeek-V4-Pro** — escalated turns (hard design, review, tough bugs)

Exact routing rules: `docs/specs/` (to be filled) and future `/model` / `/preset` UX (Deep Code + Reasonix patterns).

## Success signals (qualitative, v0)

- Same task finishes with fewer serial waits than “single-thread agent” tools
- Long sessions stay affordable via cache hit rates users can see
- Thinking/effort knobs are first-class, not hidden env hacks
- Plan mode helps without trapping the agent in endless planning
