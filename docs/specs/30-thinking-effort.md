# Spec 30 — Thinking mode and reasoning effort

| Field | Value |
|-------|--------|
| Status | **ready-for-impl** (API wiring); UX polish may remain manual |
| Philosophy | HARNESS §4 Deep Code surface; ADR 0005 |
| Gate | Part of **G2** |
| Tests | Automated request-shape tests required; UI manual OK |

## 1. Behavior

### 1.1 API wiring (M1 must)

For Chat Completions to DeepSeek:

```json
{
  "model": "<wire id from spec 20>",
  "messages": [ ... ],
  "stream": true,
  "reasoning_effort": "low|high|max",
  "tools": [ ... optional ... ]
}
```

Plus `thinking` in provider-specific extra body:

```json
{ "thinking": { "type": "enabled" } }
```

or `"disabled"`.

**Defaults (coding agent):**

| Preset | thinking | effort |
|--------|----------|--------|
| flash | enabled | high |
| balanced | enabled | high |
| max | enabled | max |
| user disables thinking | disabled | (omit effort or low) |

### 1.2 Stream handling

- Accumulate `reasoning_content` deltas separately from `content`.  
- Expose to TUI as collapsible “thinking” in M3; M1 may log or simple print.  
- **Tool turns:** persist `reasoning_content` on assistant message for API replay (ADR 0005).

### 1.3 Sampling

Omit `temperature` / `top_p` when thinking enabled.

### 1.4 UX (M3 polish; not blocking G2)

- `/model` shows thinking + effort.  
- User can set effort without changing model tier.

## 2. Non-goals

- Showing full CoT in stable prefix  
- Relying on temperature for “creativity” in thinking mode  

## 3. Failure modes

| Case | Behavior |
|------|----------|
| API 400 missing reasoning_content with tools | Fix transcript; surface error |
| Effort value rejected | Fall back `high` + warn |

## 4. Test plan

| Test | Type | Expect |
|------|------|--------|
| `request_includes_thinking_enabled` | auto | extra body shape |
| `request_includes_reasoning_effort` | auto | field present |
| `tool_turn_keeps_reasoning_content` | auto | transcript fixture round-trip |
| `omits_temperature_when_thinking` | auto | no temperature key |
| Manual: stream shows thinking then answer | manual M3 | optional |

## 5. Implementation notes

- `dsb-provider-deepseek` owns wire encoding.  
- Effort aliases: product `medium` → wire `high` if we expose medium in UX.  
