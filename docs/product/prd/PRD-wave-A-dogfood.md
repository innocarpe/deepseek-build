# PRD — Wave A: Dogfood core

| Field | Value |
|-------|--------|
| SemVer band | **`0.2.0` – `0.7.0`** |
| Plan id | `dogfood-0x` |
| Status | Active |
| Parent | [PRD-v1.md](../PRD-v1.md) · [MASTER_PLAN.md](../MASTER_PLAN.md) |

## Problem

The engine exists (`0.1.0`/`0.2.0`) but the owner cannot treat DeepSeek Build as a **daily coding tool**: install story incomplete relative to full dogfood, tools incomplete for real work, no sessions, no npm.

## Goal

Reach **dogfood-usable**: install → auth → chat → read/edit/write/search/bash under policy on real repos, documented, still on **`0.x.y`**.

## Non-goals

- `1.0.0`
- Parallel tools / subagents
- Full MCP / full skills product
- Theme polish (may land early only if cheap; formal theme is Wave B)

## User stories

1. As a developer, I install so `deepseek-build` and `dsb` are on PATH.  
2. As a developer, I multi-turn chat on Flash and escalate to Pro visibly.  
3. As a developer, I edit this repo safely via snippets and create files.  
4. As a developer, I search the tree and run allowed shell commands.  
5. As a developer, I resume a session after restart.  
6. As a developer, I can `npm i -g` (or documented equivalent) by **`0.7.0`**.

## Exit criteria (all required)

- [ ] All `dogfood-0x` ultragoal stories complete  
- [ ] Dogfood-usable checklist in [RELEASE_TRAIN_0x.md](../RELEASE_TRAIN_0x.md) §3  
- [ ] Version ≥ **`0.7.0`** or documented residual gaps with owner accept  
- [ ] Dual CLI + full SemVer on every release  

## Minors

| SemVer | Capability |
|--------|------------|
| `0.2.0` | Install / PATH (**shipped**) |
| `0.3.0` | Tools daily (**shipped** if on main) |
| `0.4.0` | Dogfood proof |
| `0.5.0` | Sessions |
| `0.6.0` | Surface min |
| `0.7.0` | npm |

## Metrics (qualitative)

- Owner uses agent for a real PR without falling back to manual edit for the happy path  
- No secrets in git; out-of-cwd write still denied by default  
