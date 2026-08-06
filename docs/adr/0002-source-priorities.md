# ADR 0002 — Design source priorities

- **Status:** Accepted
- **Date:** 2026-08-06

## Context

Multiple coding agents exist. We want a DeepSeek-native tool that feels extremely fast, without cloning any one project blindly.

## Decision

**In scope for design (priority order):**

1. Grok Build — speed & orchestration
2. Reasonix — cache-first DeepSeek loop & cost
3. Deep Code CLI (`lessweb/deepcode-cli`) — official DeepSeek-oriented CLI surface

**Out of scope for v1 feature design:**

- Gajae-code multi-stage planning / team / mobile harness

## Consequences

- Feature proposals citing only Gajae patterns are rejected unless re-justified under speed north star
- Deep Code is not optional “inspiration”; thinking, effort, skills, MCP, permissions, plan are design inputs
- Research notes may still mention Gajae under `docs/research/` without committing product features
