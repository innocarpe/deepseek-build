# Design sources

Ordered. Higher rank wins on conflict unless an ADR says otherwise.

## 1. Grok Build (primary structural + orchestration reference)

**Local:** `../../../grok-build` (sibling under `OpenSources/`)

| Take | Leave (for now) |
|------|------------------|
| Parallel tools, subagents, background tasks | Full monorepo clone / xAI branding |
| Workflow-style multi-agent fan-out patterns | xAI-only auth and telemetry |
| Native-speed tool runtime mindset | Every product feature in the pager |
| Repo modularity (crate / package boundaries) | Forced hard-fork day one |

## 2. Reasonix (DeepSeek cache + cost)

**Local:** `../../../DeepSeek-Reasonix`

| Take | Leave (for now) |
|------|------------------|
| Cache-stable system/tool/memory prefix | Desktop / full plugin surface day one |
| Flash-first, Pro escalate UX | Every TOML config dial |
| Tool-call repair | Multi-provider as primary identity |

## 3. Deep Code CLI (official DeepSeek-oriented CLI)

**Upstream:** https://github.com/lessweb/deepcode-cli  
**DeepSeek docs:** agent integration “Deep Code”

| Take | Leave (for now) |
|------|------------------|
| Thinking + reasoning effort controls | Node stack requirement |
| Agent Skills discovery (`.agents/skills`, project/user paths) | VS Code parity day one |
| MCP, permissions, `/plan`, session commands | Every slash command identical naming forever |
| Context caching awareness | Multivendor Coding Plan as core identity |

Star count is irrelevant; **official listing + V4-tuned harness philosophy** is the reason it is first-class.

## Explicitly deferred

### Gajae-code

**Local:** `../../../gajae-code`

Useful ideas may exist later (minimal verify-before-done). **v1 product design does not import:**

- deep-interview → ralplan → ultragoal
- tmux team workers
- mobile / Telegram answer loops as core surface

Reason: observed wall-clock progress is too poor for our north star; multi-stage harness risk of “planning forever.”

## Conflict rule

```text
product/VISION + NON_GOALS
    > adr/
    > specs/
    > research/ notes
    > “looks cool in another tool”
```
