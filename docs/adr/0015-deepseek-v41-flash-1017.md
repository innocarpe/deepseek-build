# ADR 0015 — DeepSeek V4.1 Flash 1017 model IDs

- **Status:** Accepted
- **Date:** 2026-10-19
- **Amends:** [ADR 0005](./0005-deepseek-provider-contract.md) (Flash wire ID and provider-specific model mapping)
- **Related:** [Spec 20](../specs/20-model-routing.md)
- **Evidence date:** 2026-10-17 (DeepSeek update) · 2026-10-19 (OpenRouter catalog metadata)

## Context

DeepSeek's 2026-10-17 update ships **DeepSeek-V4.1-Flash-1017**. The official
API's current model ID is `deepseek-flash`. The previous Flash IDs
`deepseek-v4-flash` and `deepseek-v4-flash-vision-exp` are retired aliases: the
API still accepts them and forwards them to the current Flash model, but they
are no longer the ID to write into new configuration.

OpenRouter exposes the same 1017 snapshot under the reproducible ID
`deepseek/deepseek-v4.1-flash-1017` (canonical slug
`deepseek/deepseek-v4.1-flash-20261017`). Its older
`deepseek/deepseek-v4-flash` entry is a different model: the 2026-04-23
snapshot, with text-only input and a different completion limit. The two
OpenRouter IDs are not aliases of one another.

DSB currently uses `deepseek-v4-flash` as both a local config slot and the
direct-API wire ID, and derives the OpenRouter ID by prefixing `deepseek/`.
That worked while the slot name and the wire ID happened to match. It cannot
express the current state: the direct API wants `deepseek-flash`, OpenRouter
wants a dated 1017 snapshot, and the Pro slot must keep its own IDs. This ADR
separates the two layers before the implementation changes.

The 2026-10-17 update also publishes benchmark figures. They are vendor
claims; DSB has not independently measured V4.1 Flash-1017 and this ADR does
not restate those figures as DSB results.

## Decision

### 1. Separate product slots from provider wire IDs

A **product slot** is the local configuration key (`[model.<slot>]` and
`[models] default`). A **provider wire ID** is the value a request puts in its
JSON `model` field. The slot is stable across providers; the wire ID is
provider-specific.

| Product slot | DeepSeek direct wire ID | OpenRouter wire ID |
|---|---|---|
| Flash `deepseek-v4-flash` | `deepseek-flash` | `deepseek/deepseek-v4.1-flash-1017` |
| Pro `deepseek-v4-pro` | `deepseek-v4-pro` | `deepseek/deepseek-v4-pro` |

No common wire ID collapses the two providers. Provider IDs are explicit
mapping entries, not a prefix rule applied to the slot name.

### 2. Direct DeepSeek API

- New requests and new product-managed configuration write **`deepseek-flash`**
  for the Flash slot.
- `deepseek-v4-flash` and `deepseek-v4-flash-vision-exp` are **legacy input
  aliases**. They resolve to the Flash slot when read. A product-managed write
  replaces either with `deepseek-flash`.
- The aliases remain accepted by the API for hand-written configurations and
  older clients; DSB does not require the API to reject them.
- The Pro slot continues to use `deepseek-v4-pro`.

### 3. OpenRouter

- Flash uses the pinned snapshot **`deepseek/deepseek-v4.1-flash-1017`**.
- `deepseek/deepseek-v4-flash` is the legacy product value seeded by earlier
  DSB versions. It resolves to the Flash slot for migration, and a
  product-managed write replaces it with
  `deepseek/deepseek-v4.1-flash-1017`. It is not the 1017 snapshot.
- Pro continues to use `deepseek/deepseek-v4-pro`.
- The direct-API ID `deepseek-flash` is never sent to OpenRouter, and the
  OpenRouter ID is never sent to the direct API.

### 4. Keep the local config slots

- `[model.deepseek-v4-flash]`, `[model.deepseek-v4-pro]`, and
  `[models] default = "deepseek-v4-flash"` remain valid. A slot rename is a
  breaking config change and is out of scope.
- The slot name is a local key. It is not a wire ID and is not sent to a
  provider.
- The `model =` line inside a Flash stanza is the provider wire ID for that
  home. The same slot can therefore hold `deepseek-flash` in a DeepSeek home
  and `deepseek/deepseek-v4.1-flash-1017` in an OpenRouter home.
- The display label `DeepSeek V4.1 Flash` remains the product name.

### 5. Preserve Pro

- `/pro`, `/preset max`, one-shot Pro turns, the Pro row in the model catalog,
  and the Pro-unavailable fallback keep their existing behavior.
- The Flash migration never rewrites a Pro slot, a user-entered Pro ID, or a
  `[model.<other>]` stanza.
- OpenRouter Pro keeps `deepseek/deepseek-v4-pro`; this ADR does not introduce
  a new Pro snapshot.

### 6. Compatibility and migration rules

| Value read | Provider | Treatment | Product-managed write |
|---|---|---|---|
| `deepseek-flash` | DeepSeek | Flash slot | `deepseek-flash` |
| `deepseek-v4-flash` | DeepSeek | Legacy Flash alias → Flash slot | `deepseek-flash` |
| `deepseek-v4-flash-vision-exp` | DeepSeek | Legacy Flash alias → Flash slot | `deepseek-flash` |
| `deepseek-v4-pro` | DeepSeek | Pro slot, unchanged | `deepseek-v4-pro` |
| any other value | DeepSeek | Custom model, used as-is | no rewrite |
| `deepseek/deepseek-v4.1-flash-1017` | OpenRouter | Flash slot, keep | same |
| `deepseek/deepseek-v4-flash` | OpenRouter | Legacy product Flash value → Flash slot | `deepseek/deepseek-v4.1-flash-1017` |
| `deepseek/deepseek-v4-pro` | OpenRouter | Pro slot, unchanged | `deepseek/deepseek-v4-pro` |
| any other value | OpenRouter | Custom model, used as-is | no rewrite |

Rules:

1. A rewrite happens only inside the product-managed Flash stanza and only
   when its `model` value is one of the known legacy Flash values above.
2. A value that is not a known Flash or Pro pin is a custom model. It is
   preserved byte-for-byte in configuration and used unchanged on the wire.
3. A user-entered Pro value is preserved even when the Flash migration runs.
4. Migration is idempotent: running it twice produces the same file, and it
   does not duplicate a stanza or drop unrelated keys, comments, or sections.
5. Explicit process-level model choices (`-m`, `--model`, and slash commands)
   follow the same read rules: known legacy Flash aliases resolve to the Flash
   slot; unknown values pass through; Pro values pass through.

### 7. Context and output limits

| Provider / model | Context | Max output | Input | Source |
|---|---|---|---|---|
| DeepSeek direct `deepseek-flash` | 1,048,576 tokens (1M) | 393,216 tokens (384K) | text + image | [DeepSeek pricing](https://api-docs.deepseek.com/quick_start/pricing/) · [2026-10-17 update](https://api-docs.deepseek.com/updates/) |
| OpenRouter `deepseek/deepseek-v4.1-flash-1017` | 1,048,576 (`context_length`); 1,048,576 (`top_provider.context_length`) | 393,216 (`top_provider.max_completion_tokens`) | text + image | `GET https://openrouter.ai/api/v1/models`, read 2026-10-19 |
| OpenRouter `deepseek/deepseek-v4-flash` (legacy 2026-04-23) | 1,048,576 (`context_length`); 1,048,575 (`top_provider.context_length`) | 943,717 (`top_provider.max_completion_tokens`) | text | same catalog read |
| OpenRouter `deepseek/deepseek-v4-pro` | 1,048,576 (`context_length`); 1,024,000 (`top_provider.context_length`) | 384,000 (`top_provider.max_completion_tokens`) | text | same catalog read |

- The direct-API and OpenRouter numbers are recorded separately. A provider
  stanza uses that provider's value; it does not copy the other provider's
  metadata.
- The product seed's `context_window = 1048576` matches both the direct API
  and the OpenRouter 1017 Flash metadata. The native loop's `max_tokens`
  default remains **65536** ([ADR 0014](./0014-native-completion-budget.md)),
  inside both providers' completion ranges.
- The old OpenRouter `deepseek/deepseek-v4-flash` row is recorded because it is
  the value earlier DSB versions seeded. Its different completion limit is one
  reason the old and new OpenRouter IDs must not be treated as
  interchangeable.

### 8. Visibility

Every turn's visibility line shows the provider wire ID that was actually
sent. A direct Flash turn shows `model=deepseek-flash`; an OpenRouter Flash
turn shows `model=deepseek/deepseek-v4.1-flash-1017`; a direct Pro turn shows
`model=deepseek-v4-pro`. The product slot label may appear beside it, but the
wire ID is the value the provider received.

## Consequences

- ADR 0005's Flash pin is amended. Its Pro pin, endpoints, thinking/tool
  behavior, and cache contract remain in force.
- [Spec 20](../specs/20-model-routing.md) carries the behavior and the
  follow-up acceptance criteria. This ADR adds no runtime code, tests,
  configuration migration, README, user-guide, or CHANGELOG changes.
- Until the follow-up `feat` unit lands, current installs still send the old
  IDs. Direct-API installs keep working through the retired alias; OpenRouter
  installs send `deepseek/deepseek-v4-flash`, which the catalog maps to the
  2026-04-23 model, so they do not receive the 1017 snapshot until the
  implementation changes.
- The first turn after the wire-ID change may start a new provider-side prefix
  cache for that model. The prompt, tool schemas, and skills index are
  unchanged; the expected cache cost is one cold prefix per provider, not a
  prompt-shape change.
- If DeepSeek retires `deepseek-flash`, or OpenRouter retires the 1017
  snapshot, open a superseding ADR with a new dated pin.

## References

- DeepSeek update, 2026-10-17: https://api-docs.deepseek.com/updates/
- DeepSeek V4.1 Flash news: https://api-docs.deepseek.com/news/news261017/
- DeepSeek pricing and model table: https://api-docs.deepseek.com/quick_start/pricing/
- OpenRouter 1017 model page: https://openrouter.ai/deepseek/deepseek-v4.1-flash-1017
- OpenRouter model catalog: `GET https://openrouter.ai/api/v1/models` (read 2026-10-19)
- [ADR 0005](./0005-deepseek-provider-contract.md)
- [ADR 0014](./0014-native-completion-budget.md)
- [Spec 20](../specs/20-model-routing.md)
