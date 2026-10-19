# ADR 0015 — Follow the current DeepSeek Flash API model

- **Status:** Accepted
- **Date:** 2026-10-19
- **Supersedes:** [ADR 0005](./0005-deepseek-provider-contract.md) (Flash wire ID only)
- **Evidence:** [DeepSeek change log](https://api-docs.deepseek.com/updates/), [models and pricing](https://api-docs.deepseek.com/quick_start/pricing/), [OpenRouter model](https://openrouter.ai/deepseek/deepseek-v4.1-flash-1017)

## Context

DeepSeek upgraded V4.1 Flash to the 1017 checkpoint on 2026-10-17 without
changing its API call shape or price. The documented DeepSeek API model ID is
`deepseek-flash`. The previously pinned `deepseek-v4-flash` names a retired
model and is only a temporary compatibility alias. OpenRouter publishes the
1017 checkpoint under `deepseek/deepseek-v4.1-flash-1017`.

DSB's native loop and full-screen agent used different model-selection paths.
Both must reach the current model, including existing homes seeded before this
change.

## Decision

1. Native DeepSeek requests use `deepseek-flash`; `ModelId::from_wire` accepts
   the retired ID but serializes Flash using the current ID.
2. The full-screen config keeps `[model.deepseek-v4-flash]` as its stable
   *local slot*. The `model` field inside it becomes `deepseek-flash` on the
   DeepSeek API or `deepseek/deepseek-v4.1-flash-1017` on OpenRouter. Existing
   `[models]` references, `/flash`, and saved sessions keep resolving the slot.
3. On launch or provider switch, migrate only a Flash stanza using the
   product's old wire ID on the corresponding official endpoint. Preserve a
   custom wire ID, endpoint, display name, and other explicit user choices.
4. Keep the V4 Pro slot and its opt-in routes: DeepSeek's current pricing and
   model list still advertise `deepseek-v4-pro`. No benchmark result measured
   before 2026-10-17 is relabeled as a 1017 result.

## Consequences

- The current DeepSeek API alias follows later Flash checkpoint upgrades.
  OpenRouter's model ID remains pinned to 1017 until deliberately updated.
- The local Flash slot retains an old-looking key for compatibility; the
  actual outgoing `model` field is the authority for which model is used.
- The documented 1,048,576-token context and 393,216-token API output limit
  remain valid. This change does not add Search & Fetch APIs or change the
  product's `max_tokens` default from ADR 0014.
