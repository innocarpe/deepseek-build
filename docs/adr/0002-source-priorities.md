# ADR 0002 — Design source priorities

- **Status:** Accepted; **amended** 2026-08-06 (layered L1/L2/L3)
- **Date:** 2026-08-06

## Context

Multiple coding agents exist. We want a DeepSeek-native tool that feels extremely fast, without cloning any one project blindly.

A naive global ranking “Grok > Reasonix > Deep Code” biases implementers toward Grok **tool shapes**, which can violate DeepSeek-native harness fit (see Deep Code architecture: tool schemas are not neutral).

## Decision

Use **layered ownership**:

| Layer | Primary sources | Domain |
|-------|-----------------|--------|
| L1 | Deep Code (+ Reasonix on cache layout) | Tool/edit contracts, skills-as-context, permissions, DeepSeek CLI surface |
| L2 | Reasonix | Cache-first invariant, Flash/Pro, tool-call repair |
| L3 | Grok Build | Parallelism, subagents, background execution |

**L3 must not override L1/L2.**

Normative write-up: [HARNESS_PHILOSOPHY.md](../architecture/HARNESS_PHILOSOPHY.md).  
Table form: [SOURCES.md](../product/SOURCES.md).

**Out of scope for v1 feature design:** Gajae-code multi-stage planning / team / mobile harness.

## Consequences

- Specs map to layers (e.g. 45 snippet-edit is L1 before free-form Grok-like edit).  
- Subagents (L3) must obey worker cache law (L2).  
- Reviews reject “speed” changes that break snippet safety or prefix stability.  
