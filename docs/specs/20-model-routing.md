# Spec 20 — Model routing (Flash / Pro)

| Field | Value |
|-------|--------|
| Status | **ready-for-impl** |
| Philosophy | HARNESS §5, §7; Reasonix Flash-first |
| Gate | Part of **G2** |
| Tests | Automated routing table tests required |
| Contracts | [ADR 0005](../adr/0005-deepseek-provider-contract.md) · [ADR 0015](../adr/0015-deepseek-v41-flash-1017.md) |

## 1. Provider wire IDs and product slots

A **product slot** is the local configuration key. A **provider wire ID** is
the value a request sends in its JSON `model` field. The layers are distinct:
a slot name is never sent as a wire ID, and a provider ID is never used as a
config section name.

| Tier | Product slot | DeepSeek direct wire | OpenRouter wire |
|------|--------------|----------------------|-----------------|
| Flash | `deepseek-v4-flash` | `deepseek-flash` | `deepseek/deepseek-v4.1-flash-1017` |
| Pro | `deepseek-v4-pro` | `deepseek-v4-pro` | `deepseek/deepseek-v4-pro` |

- The slot names stay for config compatibility (`[model.deepseek-v4-flash]`,
  `[model.deepseek-v4-pro]`, `[models] default = "deepseek-v4-flash"`).
  Renaming a slot is a separate breaking change.
- Flash's direct ID is the stable official alias `deepseek-flash`. The retired
  aliases `deepseek-v4-flash` and `deepseek-v4-flash-vision-exp` are
  read-compatibility values; new product-managed writes use `deepseek-flash`.
- Flash's OpenRouter ID is the pinned 1017 snapshot.
  `deepseek/deepseek-v4-flash` is the older 2026-04-23 model, not an alias of
  the snapshot.
- Pro keeps `deepseek-v4-pro` on the direct API and
  `deepseek/deepseek-v4-pro` on OpenRouter. The Flash change does not remove
  or rewrite the Pro path.
- Provider IDs are explicit mapping entries. They are not derived by prefixing
  the slot name.

## 2. Behavior

### 2.1 Defaults

| Session preset | Model | Default effort (spec 30) |
|----------------|-------|---------------------------|
| `flash` (product default) | Flash | `low` or `high` (product default **`high`** for coding quality; user can lower) |
| `balanced` | Flash for tools; Pro for escalations | Flash `high`, Pro `high` |
| `max` | Pro | `max` |

M1 minimum: implement **session default Flash** + **explicit escalate to Pro**.

A default turn on either provider uses the Flash slot and the provider's Flash
wire ID from §1. The default turn log shows that wire ID, not the slot name.

### 2.2 Escalation triggers (product)

| Trigger | Result |
|---------|--------|
| User `/pro` or “use pro for this turn” | Next turn Pro, then return to default unless sticky preset |
| User `/preset max` | Session sticky Pro |
| User `/preset flash` | Session sticky Flash |
| Automatic router (optional M1) | May escalate **one turn** to Pro when: plan/architecture keywords **and** user did not force flash; must log `escalate_reason` |

**Precedence:** explicit user model/preset **>** sticky session preset **>**
automatic router **>** default Flash.

A Pro turn uses the provider's Pro wire ID from §1. An OpenRouter Pro turn
sends `deepseek/deepseek-v4-pro`; a direct Pro turn sends `deepseek-v4-pro`.

### 2.3 Visibility

- Every turn log/UI must show **which wire model** ran:
  `model=deepseek-flash`, `model=deepseek/deepseek-v4.1-flash-1017`,
  `model=deepseek-v4-pro`, or the provider's Pro ID.
- The product slot label may appear beside the wire ID. The wire ID is the
  value the provider received.
- Never silently stay on Pro after one-shot `/pro` unless preset is `max`.

### 2.4 Unavailable model

If Pro returns 404/unsupported: fall back Flash + user-visible warning; do not
crash loop. The fallback uses the provider's Flash wire ID from §1. The
Pro-404 fallback does not change the configured Pro value.

### 2.5 Cost

Do not auto-loop Pro on every tool turn. Subagents (M4) default Flash
(HARNESS worker law).

## 3. Compatibility and migration

### 3.1 Read rules

| Value read | Provider | Resolves to | Product-managed write |
|---|---|---|---|
| `deepseek-flash` | DeepSeek | Flash slot | `deepseek-flash` |
| `deepseek-v4-flash` | DeepSeek | Flash slot (legacy alias) | `deepseek-flash` |
| `deepseek-v4-flash-vision-exp` | DeepSeek | Flash slot (legacy alias) | `deepseek-flash` |
| `deepseek-v4-pro` | DeepSeek | Pro slot | `deepseek-v4-pro` |
| any other value | DeepSeek | custom model | no rewrite |
| `deepseek/deepseek-v4.1-flash-1017` | OpenRouter | Flash slot | same |
| `deepseek/deepseek-v4-flash` | OpenRouter | Flash slot (legacy product value) | `deepseek/deepseek-v4.1-flash-1017` |
| `deepseek/deepseek-v4-pro` | OpenRouter | Pro slot | `deepseek/deepseek-v4-pro` |
| any other value | OpenRouter | custom model | no rewrite |

### 3.2 Write rules

1. A product-managed Flash stanza writes the provider's current Flash wire ID
   from §1. A product-managed Pro stanza writes the provider's Pro wire ID.
2. Migration rewrites the Flash stanza only when its existing `model` value is
   one of the known legacy Flash values in §3.1.
3. A custom value — including a user-entered Pro or custom model ID — is
   preserved byte-for-byte. The migration does not rewrite `[model.<other>]`
   stanzas.
4. Migration is idempotent. It does not duplicate stanzas or drop unrelated
   keys, comments, or sections.
5. Explicit `-m` / `--model` values and slash-command choices follow the read
   rules. Known legacy Flash aliases resolve to Flash; unknown values pass
   through; Pro values pass through.
6. `[models] default` continues to name a product slot. A value that names a
   custom slot is not changed.

## 4. Provider limits

| Provider / model | Context | Max output | Input | Source |
|---|---|---|---|---|
| DeepSeek direct `deepseek-flash` | 1,048,576 (1M) | 393,216 (384K) | text + image | [pricing](https://api-docs.deepseek.com/quick_start/pricing/) · [2026-10-17 update](https://api-docs.deepseek.com/updates/) |
| OpenRouter `deepseek/deepseek-v4.1-flash-1017` | 1,048,576 (`context_length`); 1,048,576 (`top_provider.context_length`) | 393,216 (`top_provider.max_completion_tokens`) | text + image | `GET https://openrouter.ai/api/v1/models`, 2026-10-19 |
| OpenRouter `deepseek/deepseek-v4-flash` (legacy 2026-04-23) | 1,048,576 (`context_length`); 1,048,575 (`top_provider.context_length`) | 943,717 (`top_provider.max_completion_tokens`) | text | same catalog read |

- The numbers are provider-specific. A stanza or request uses its provider's
  values.
- The product seed's `context_window = 1048576` matches the direct API and the
  OpenRouter 1017 Flash metadata. The native `max_tokens` default remains
  **65536** ([ADR 0014](../adr/0014-native-completion-budget.md)), inside both
  ranges.
- The old OpenRouter `deepseek/deepseek-v4-flash` row is recorded because
  earlier DSB versions seeded it; it is not interchangeable with the 1017
  snapshot.

## 5. Non-goals

- Multi-provider routing beyond the DeepSeek direct API and OpenRouter.
- Renaming the product config slots.
- Removing Pro, changing the Pro catalog row, or changing Pro escalation
  behavior.
- Implementing runtime code, constants, config migration, tests, README, user
  guide, or CHANGELOG in the spec PR that adds this section. Those are the
  follow-up `feat` unit.
- Re-measuring or restating the 2026-10-17 benchmark figures. Vendor claims
  stay vendor claims.
- Changing the context-window migration already landed (`1048576`).

## 6. Acceptance criteria and test plan (follow-up implementation)

These names are the behavior the follow-up implementation must test. This
spec PR adds the criteria, not the tests.

| Test | Expect |
|---|---|
| `default_flash_direct_wire` | A default direct-API session sends `model: "deepseek-flash"`. |
| `default_flash_openrouter_wire` | A default OpenRouter session sends `model: "deepseek/deepseek-v4.1-flash-1017"`. |
| `current_pins_resolve_to_flash` | Stored `deepseek-flash` (direct) and `deepseek/deepseek-v4.1-flash-1017` (OpenRouter) resolve to the Flash slot and are kept on the next product-managed write. |
| `provider_wire_ids_are_explicit` | The provider mapping lists each slot's direct and OpenRouter IDs explicitly; no test can change one provider's ID by editing a slot string or relying on a `deepseek/` prefix. |
| `direct_legacy_flash_alias_normalized` | A product-managed Flash stanza containing `deepseek-v4-flash` or `deepseek-v4-flash-vision-exp` resolves to Flash and is rewritten to `deepseek-flash` on the next product-managed write. |
| `openrouter_legacy_flash_migrates` | A product-managed Flash stanza containing `deepseek/deepseek-v4-flash` resolves to Flash and is rewritten to `deepseek/deepseek-v4.1-flash-1017`. |
| `custom_model_id_preserved` | A value outside the known Flash/Pro pins is kept byte-for-byte and is sent unchanged. |
| `pro_wire_ids_unchanged` | `/pro`, `/preset max`, and one-shot Pro send `deepseek-v4-pro` direct and `deepseek/deepseek-v4-pro` OpenRouter. |
| `pro_oneshot_then_flash` | After a one-shot `/pro` turn, the next default turn uses the provider's Flash wire ID. |
| `pro_and_custom_values_preserved` | The Flash migration changes no Pro value and no `[model.<other>]` stanza. |
| `slot_name_is_config_key` | `[model.deepseek-v4-flash]` and `[models] default = "deepseek-v4-flash"` remain valid; the slot name never appears as a wire ID. |
| `visibility_shows_wire_id` | The turn visibility line contains the provider wire ID actually sent. |
| `context_window_per_provider` | A provider stanza uses that provider's context value; the 1017 Flash entries use 1,048,576 and the legacy OpenRouter entry's 943,717 completion limit is not copied onto the new pin. |
| `legacy_openrouter_model_not_seeded` | A new OpenRouter seed never writes `deepseek/deepseek-v4-flash`. |
| `preset_max_sticky` | A sticky `max` preset remains Pro. |
| `user_beats_router` | A forced Flash turn is not overridden by the automatic router. |
| `pro_unavailable_fallback` | Pro 404/unsupported falls back to the provider's Flash wire ID with a warning flag. |

## 7. Implementation notes

- The mapping table belongs in one place in the provider/config layer; callers
  ask for a slot + provider and receive the wire ID.
- The product seed and home repair read the provider from the saved
  credentials, then write the matching wire ID into the Flash and Pro stanzas.
  They keep the slot headers and the `[models]` slot names unchanged.
- The repair recognizes the legacy values in §3.1 by exact match. It does not
  pattern-match prefixes or substrings; a custom ID that happens to contain
  `deepseek` is preserved.
- The model picker keeps its single `DeepSeek V4.1 Flash` row. The row's
  selection still names the Flash slot; the provider mapping decides the wire
  ID.
- Session titles, web search, and image description remain pinned to the Flash
  slot.
- The visibility line and the request body must use the same resolved wire ID.
  A test should compare the two rather than recompute one from the other.
- `context_window` stays `1048576` for the product stanzas that match the
  provider metadata; do not add a second migration for it.
