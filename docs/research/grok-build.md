# Research: Grok Build

**Local path:** `../../../grok-build` (from this file: `OpenSources/grok-build`)

## Why it matters

Primary answer to: *why does work finish so fast?*

## Observations (scaffolding-era summary)

- **Rust monorepo**: TUI (`xai-grok-pager`), agent shell, tools, workspace as separate crates
- **Parallel tools** + **background shell** + multi-wait
- **Subagents** with own context; optional **worktree** isolation (`xai-fast-worktree`)
- **Workflow engine** (Rhai) with parallel agent panels and budgets
- Hashline-style edit anchors, codebase graph, fast git/fs helpers

## Takeaways for DeepSeek Build

- Optimize for fewer serial waits
- Subagents should not re-pay huge uncached prefixes carelessly (tension with DeepSeek cache — design carefully)
- Modular package boundaries beat one `src/` blob

## Not taking yet

- Full hard-fork
- xAI auth/models catalog as-is
