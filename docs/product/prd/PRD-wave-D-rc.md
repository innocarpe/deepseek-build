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

- [x] npm and/or binary install works on macOS + Linux *(scripts + smoke; prebuilt CDN optional)*  
- [ ] `deepseek-build --version` and `dsb --version` report **`1.0.0`** *(set on release PR)*  
- [x] user-guide covers install, auth, chat, tools, permissions, theme (**0.16.0**)  
- [ ] CHANGELOG for `1.0.0` *(added on release PR)*  
- [x] Known limitations published (`docs/product/KNOWN_LIMITS.md`)  
- [x] Product CI: `cargo test --workspace` (+ offline smoke) — **0.15.0**  
- [x] Default theme = DeepSeek blue readability profile (**0.9.0+**)  
- [ ] Owner confirmation: multi-day dogfood without critical blockers  
- [x] Waves A+B+C complete  

## Non-goals

- Scope freeze forever  
- Process-police CI  

## Suggested minors

| SemVer | Theme |
|--------|--------|
| `0.15.0` | Harden + CI smoke |
| `0.16.0` | user-guide + limits |
| `1.0.0` | Tag only when checklist green |
