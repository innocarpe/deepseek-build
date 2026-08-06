# Research: Gajae-code (deferred)

**Local path:** `OpenSources/gajae-code`

## Status

**Out of v1 feature design** (ADR 0002).

## Why recorded

Avoid re-litigating without evidence. User experience: planning and task execution progress extremely poorly / too slowly relative to Grok Build.

## Structural risk notes (non-exhaustive)

- Multi-stage workflow layers on top of the agent loop
- Subagent fan-out can re-pay large cold system prompts
- Heavier TS/Bun + TUI hot paths vs native Grok-style runtime

## Possible future cherry-picks (not commitments)

- Minimal “verify before claim done” gate without the full harness
- Isolated ideas only with an ADR and speed budget
