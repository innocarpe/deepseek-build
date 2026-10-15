# ADR 0014 — Raise the native completion budget past max-effort reasoning

- **Status:** Accepted
- **Date:** 2026-10-15
- **Amends:** [ADR 0005](./0005-deepseek-provider-contract.md) (default product max_tokens row)

## Context

ADR 0005 pinned the M1 implementation default `max_tokens` at **8192** as a
cost/latency guard, noting it was not the API maximum. The native loop
(`dsb run` / `dsb chat`) has sent that budget on every request since.

Measured against the API on 2026-10-15 with thinking enabled and
`--effort max`:

| Probe | Result |
|-------|--------|
| `max_tokens: 8192`, reasoning-heavy prompt | `finish_reason: length`, `completion_tokens: 8192`, `reasoning_tokens: 8091` — the whole budget spent inside reasoning |
| `max_tokens: 65536` / `131072` / `262144` | accepted |
| `max_tokens: 1048576` | rejected: "the valid range of max_tokens is [1, 393216]" |

A `dsb run` benchmark trial (terminal-bench `wal-recovery-ordering`, run for
the `feat/dsb-run-benchmarkable` unit) then showed the loop-facing symptom:
two consecutive responses with no visible content and no tool calls, each
burning a full budget, and the turn ended with the task untouched. A
reasoning-only response is the model's completion signal (spec 15 §1.5); a
truncated one is not, and the loop could not tell them apart.

## Decision

1. `DEFAULT_MAX_TOKENS` becomes **65536** — roughly 8x the measured
   truncation point and well inside the API's `[1, 393216]` range.
   `max_tokens` is an upper bound, not a target: billing follows tokens
   actually generated, and short responses are unaffected.
2. The loop emits a warning when a response arrives with
   `finish_reason == "length"`, so a truncated turn is visible instead of
   reading as a reasoning-only completion.

## Consequences

- Max-effort reasoning no longer truncates on ordinary coding turns.
- A runaway response may grow to the new bound before the provider stops it;
  the turn stays bounded by `max_tool_rounds`.
- ADR 0005's "unless user overrides" phrase remains unimplemented: there is no
  per-run `--max-tokens` flag. If a response is ever observed truncating at
  65536, the next step is that flag or the API ceiling; reuse the measurement
  table above rather than re-deriving it.
