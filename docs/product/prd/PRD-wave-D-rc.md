# PRD — Wave D: Release candidate → `1.0.0`

| Field | Value |
|-------|--------|
| SemVer band | **`0.15.0` – `1.0.0`** |
| Plan id | `rc-1.0.0` |
| Status | Planned (after Wave C, or after B if owner defers throughput — document fork) |
| Depends on | Waves A + B required; Wave C **strongly recommended** before `1.0.0` |

## Problem

Features exist but install/docs/CI/limits are not “boring.” Calling anything **`1.0.0`** early burns trust.

## Goal

Ship **`1.0.0`** only when install, docs, defaults (including theme), and sustained dogfood make the product honest.

## Exit criteria for **`1.0.0`**

- [ ] npm and/or binary install works on macOS + Linux  
- [ ] `deepseek-build --version` and `dsb --version` report **`1.0.0`**  
- [ ] user-guide covers install, auth, chat, tools, permissions, theme  
- [ ] CHANGELOG for `1.0.0`  
- [ ] Known limitations published  
- [ ] Product CI: `cargo test --workspace` (+ install smoke if feasible)  
- [ ] Default theme = DeepSeek blue readability profile  
- [ ] Owner confirmation: multi-day dogfood without critical blockers  
- [ ] Waves A+B complete; Wave C complete **or** explicit ADR “throughput deferred post-1.0.0”  

## Non-goals

- Scope freeze forever  
- Process-police CI  

## Suggested minors

| SemVer | Theme |
|--------|--------|
| `0.15.0` | Harden + CI smoke |
| `0.16.0` | user-guide + limits |
| `1.0.0` | Tag only when checklist green |
