# H10 Path A prefix/epoch — G006 evidence

**Date:** 2026-08-07  
**Band:** `3.0.0-beta.1`  
**Story:** G006 L2-Prefix · WAVE `3x-H2-1`

## What shipped

| Layer | Change |
|-------|--------|
| Product API | `dsb_context::assemble_path_a_context` for Path A agent message assembly |
| Tests | H10.1–H10.3 + skills thrash + prefix/message layout |

## Commands

```bash
cargo test -p dsb-context path_a
cargo test -p dsb-context   # H10.4 thin regression
```

## Honest limits

Grok compaction still owns some Path A history shaping; product contract requires stable-prefix discipline for DeepSeek turns via this assembly API (or equivalent byte-stable inputs). Full shell wiring of every Grok prompt path may continue in G007/G008 polish if gaps remain — P0 is hash-stable assembly under the agent stack contract.

## SemVer

**`3.0.0-beta.1`**
