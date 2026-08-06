# DeepSeek Build — agent contract

This file is standing instructions for any coding agent working in this repo.

## Current phase

**Docs-first scaffolding.** Prefer editing `docs/` over inventing implementation code until the stack and MVP specs are locked via ADRs.

## Source priorities (fail-close)

1. **Grok Build** — orchestration, parallelism, runtime layout patterns
2. **Reasonix** — DeepSeek prefix-cache contract, cost loop
3. **Deep Code CLI** — official DeepSeek CLI surface (thinking, effort, skills, MCP, permissions, plan)

**Do not** pull Gajae-code workflow surfaces into v1 design (deep-interview, ralplan, ultragoal, tmux teams).

## Documentation rules

| Write here | Kind of truth |
|------------|----------------|
| `docs/product/` | Why we exist, who for, what we refuse |
| `docs/specs/` | Must-behavior for shipping features |
| `docs/architecture/` | How the system and repo are shaped |
| `docs/adr/` | Irreversible or contested decisions (one ADR per decision) |
| `docs/research/` | Evidence from other tools; not product commitment |
| `docs/user-guide/` | End-user docs only (after behavior exists) |

If product intent and code disagree later, **specs + ADRs win** until intentionally revised.

## Layout

See `docs/architecture/REPO_LAYOUT.md`. Do not invent top-level folders without an ADR.

## Sibling paths

- Grok Build: `../grok-build`
- Reasonix: `../DeepSeek-Reasonix`
