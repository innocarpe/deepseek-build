# ADR 0005 — DeepSeek provider contract

- **Status:** Accepted  
- **Date:** 2026-08-06  
- **Gate:** G1b  
- **Evidence date:** 2026-08-06 against official docs  
  - https://api-docs.deepseek.com/quick_start/pricing  
  - https://api-docs.deepseek.com/guides/thinking_mode  
  - https://api-docs.deepseek.com/guides/tool_calls  

## Context

Claude/Codex adversarial reviews required **pinned model IDs** and an executable provider contract before specs 10/15/20/30 can be ready-for-impl.

## Decision

### Endpoints

| Item | Value |
|------|--------|
| Default base URL (OpenAI-compatible Chat Completions) | `https://api.deepseek.com` |
| Auth | `Authorization: Bearer <DEEPSEEK_API_KEY>` |
| Primary API for v1 agent loop | **Chat Completions** (`/chat/completions`) |
| Anthropic-compatible base | `https://api.deepseek.com/anthropic` — optional later, not M1 default |
| Responses API | Supported for **flash only** as of evidence date; **not** M1 default (pro unsupported) |

### Pinned model IDs (wire)

| Logical tier | Wire `model` string | Notes (evidence date) |
|--------------|---------------------|------------------------|
| Flash | **`deepseek-v4-flash`** | Version label DeepSeek-V4-Flash-0731; id stays `deepseek-v4-flash` |
| Pro | **`deepseek-v4-pro`** | Premium tier |

These IDs are **accepted as product pins** based on official pricing/docs pages on 2026-08-06. If DeepSeek renames them, open a superseding ADR.

### Context / limits (documented product assumptions)

| Item | Value |
|------|--------|
| Context length | 1M (docs) |
| Max output | up to 384K (docs) |
| Default product max_tokens (M1) | Implementation default **8192** unless user overrides (cost/latency; not API max) |

### Thinking / effort (OpenAI Chat Completions path)

| Control | How we send it |
|---------|----------------|
| Thinking on/off | `extra_body`: `{"thinking": {"type": "enabled"|"disabled"}}` |
| Effort | `reasoning_effort`: `low` \| `high` \| `max` (also accept product alias `medium` → map to `high` if needed) |
| Default product | thinking **enabled**, effort **high** for Pro escalations; Flash default effort **high** or **low** per preset (spec 20/30) |

**Mapping note (docs):** Flash maps `xhigh`→`high`; Pro mapping may change — treat unknown effort as `high` and log.

**Multi-turn + tools (normative for agent loop):**

- Without tool calls between user turns: prior `reasoning_content` **need not** be resent (API ignores if sent).  
- **With tool calls:** every subsequent request **must** pass full assistant `reasoning_content` back or API returns **400**.  
- Product must preserve `reasoning_content` on the session transcript whenever tools are in play.

Streaming: support SSE chat completion streaming; accumulate `delta.reasoning_content` and `delta.content` separately.

### Sampling parameters

In thinking mode, `temperature` / `top_p` / penalties have **no effect** (no error). Product should **omit** them in thinking mode to avoid false knobs.

### Tools

- OpenAI-style `tools` / `tool_calls` on Chat Completions.  
- Optional beta **strict** tool schema via `base_url=https://api.deepseek.com/beta` — **M2+**, not required for M1 read-only tools.  
- Malformed tool JSON → product repair (spec 15) before dispatch; never execute unparsed args.

### Cache / usage telemetry (acceptance evidence)

Official pricing distinguishes **cache hit vs cache miss** input rates. Product **must**:

1. Parse usage fields from responses when present (`prompt_tokens`, `completion_tokens`, and any cache hit/miss fields the API returns — field names verified at integration against live responses; store raw JSON usage for debugging).  
2. If cache hit/miss fields are **absent**, use **documented substitute** from this ADR:  
   - Two identical consecutive turns with same stable prefix → measure latency and billed tokens if available; log `cache_evidence=substitute_dual_call`.  
3. M1 exit requires **golden prefix bytes** (spec 10) **and** either real cache fields **or** substitute protocol logged — not golden alone.

### Errors / retries

| Class | Behavior |
|-------|----------|
| 401/403 | Fail closed; prompt re-auth |
| 400 (reasoning_content / schema) | Do not infinite-retry; surface; fix transcript |
| 429 | Exponential backoff with jitter; honor `Retry-After` if present |
| 5xx / network | Retry up to N=3 for idempotent reads; user-visible on exhaust |
| Cancel | Abort HTTP body; mark turn interrupted; repair tool pairs (spec 15) |

### Non-goals (this ADR)

- Implementing the client (runtime PR)  
- Anthropic-format first  
- Multi-provider routing  

## Consequences

- G1b → **green** when this ADR merges.  
- Specs 20/30 use wire IDs above.  
- VISION/PRD may name `deepseek-v4-flash` / `deepseek-v4-pro` again as pinned.

## References

- DeepSeek pricing / models  
- Thinking mode + tool calls guides  
- [HARNESS_PHILOSOPHY](../architecture/HARNESS_PHILOSOPHY.md)  
