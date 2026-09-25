# Chat Completions wire inventory — U0.2

**Nature:** Non-binding evidence ([SOURCES.md](../product/SOURCES.md) precedence).
Measurements for the `6.1.0` depth board
([DEEPSEEK_NATIVE_DEPTH_6X_GOALS.md](../product/DEEPSEEK_NATIVE_DEPTH_6X_GOALS.md)
U0.2). Not a product commitment and not a transport change.

**Tree:** `main` at `687582c` (PR #204 merged; the `6.0.0` release commit
`87b82d2` is already on this history). Vendor pin
`036a5d8348cd744767cd0b08518ab17bf608fa7f`. On-disk `Cargo.toml` version
`6.0.0`. This note does not bump, tag, or publish anything.

**Question.** On `api_backend = "chat_completions"`, three facts decide the
gated units: does a cache key reach the wire, which usage fields report cache
reads, and can a prompt or tool change be appended after cached history?

---

## 0. What was re-checked because `main` moved

The cold-start pin `593a1e4` is an ancestor of `687582c`. The load-bearing
paths in board §1 still exist. Two line numbers in the cold start have moved:

| Cold-start cite | At `687582c` |
|---|---|
| `acp_session_impl/turn.rs:2187` calls `apply_spec10_to_conversation_request` | `turn.rs:3003` |
| `agent_status.rs:353` `format_cache_hit_pct` | `agent_status.rs:375` |

`xai-grok-shell` still does not depend on `dsb-context` or
`dsb-provider-deepseek`. The overlay call
`dsb_context::assemble_path_a_context` is
`stamp_path_a_prefix_epoch` in `crates/dsb-cli/src/agent_launch.rs:949`, which
writes `path_a_prefix_epoch.txt` at launch. The turn path that builds the
request is `apply_spec10_to_conversation_request` in
`third_party/grok-build/crates/codegen/xai-grok-shell/src/session/helpers/spec10_path_a_assembly.rs:430`,
called from `turn.rs:3003`.

A vendored `cargo` build was already running in another worktree. This unit
did not start one.

---

## 1. A cache key does not reach Chat Completions

**Code.** The main turn sets `prompt_cache_key: None`
(`xai-chat-state/src/actor/request_builder.rs:98`). Side calls do set it to
the session id (`xai-grok-shell/src/session/acp_session_impl/side_call.rs:127`),
and then `ApiBackend::forwards_prompt_cache_key` (`xai-grok-sampling-types/src/types.rs:1071`)
is true only for `Responses`. `ChatCompletionRequest` (`types.rs:52`) has no
`prompt_cache_key` field. `From<ConversationRequest> for ChatCompletionRequest`
(`conversation/chat_completions.rs:245`) does not copy it. The sampler sends
that struct (`xai-grok-sampler/src/client.rs:2065`). The Responses mapping is
the one that copies the field (`conversation/responses.rs:136`). The test
`prompt_cache_key_reaches_the_wire_only_where_the_backend_claims`
(`conversation.rs:2322`) locks the split.

**Official schema, read 2026-09-25.**
[Create chat completion](https://api-docs.deepseek.com/api/create-chat-completion)
lists no `prompt_cache_key`. The cache guide
([Context Caching](https://api-docs.deepseek.com/guides/kv_cache)) describes
automatic prefix units: a later request hits only when it fully matches a
persisted unit, and the client sends no cache key. `user_id` on that schema
is documented for isolation, and this product does not send it
(`ChatCompletionRequest.user` stays `None` in the same `From` impl).

**Live call.** No credential for `https://api.deepseek.com` was available.
The route that was available is the product's configured Chat Completions
host for this session: `POST https://openrouter.ai/api/v1/chat/completions`,
model `deepseek/deepseek-v4-flash`, `thinking.type = disabled`,
`reasoning_effort = none`, `max_tokens = 16`, `stream = false`.

| Call | `prompt_tokens` | `cached_tokens` | What was sent |
|---|---:|---:|---|
| A1 first sight of a ~768-token system+user prefix | 768 | 0 | no cache key |
| A2 same body, 5s later | 768 | 0 | no cache key |
| A3 same body, 5s later | 768 | 768 | no cache key |
| E 12-token user message plus a `prompt_cache_key` field | 12 | 0 | HTTP 200 |

A full hit happened with no cache key on the body (A3). OpenRouter accepted
an extra `prompt_cache_key` field (E, HTTP 200). That acceptance is this
host's behavior. It is not a measurement of `api.deepseek.com`, and the
product's own serializer never emits the field on this backend.

**Implication for U2.1.** Making `prompt_cache_key` land on Chat Completions
has nowhere to land. The official body has no such parameter, the vendored
mapping drops it, and a full prefix hit was observed without it. U2.1's
remaining half — record why the key cannot reach this backend — is this
section. Further wire work on the key does not survive.

---

## 2. What the usage payload reports

**Official Chat Completions schema, same fetch.** `usage` carries:

| Field | Role in the schema |
|---|---|
| `prompt_tokens` | Full input. Documented as `prompt_cache_hit_tokens + prompt_cache_miss_tokens`. |
| `prompt_cache_hit_tokens` | Required. Tokens served from the context cache. |
| `prompt_cache_miss_tokens` | Required. Tokens not served from the context cache. |
| `prompt_tokens_details.cached_tokens` | Same number as `prompt_cache_hit_tokens`. |
| `completion_tokens_details.reasoning_tokens` | Reasoning tokens, when present. |

The schema does not list a cache-write / cache-creation field on this API.

**What Path A keeps.** `Usage` (`types.rs:525`) has `prompt_cache_hit_tokens`
and `prompt_tokens_details.cached_tokens`. It has no
`prompt_cache_miss_tokens`. Serde drops unknown fields, so a miss count on
the official payload is discarded before it becomes `TokenUsage`.
`TokenUsage::from` (`conversation.rs:800`) prefers
`prompt_tokens_details.cached_tokens` whenever the details object is present,
and only then falls back to `prompt_cache_hit_tokens`.
`cache_creation_prompt_tokens` is hardcoded `0`. The pager chip reads
`cached_read_tokens` (`agent_status.rs:375`).

The overlay parser in `crates/dsb-provider-deepseek/src/usage.rs` does keep
both `prompt_cache_hit_tokens` and `prompt_cache_miss_tokens`. `xai-grok-shell`
does not depend on that crate. Path A never calls it.

**Live OpenRouter payload.** The same calls returned a different object.
Present keys: `prompt_tokens`, `completion_tokens`, `total_tokens`,
`prompt_tokens_details.cached_tokens`, `prompt_tokens_details.cache_write_tokens`,
plus audio/video and cost fields. `prompt_cache_hit_tokens` and
`prompt_cache_miss_tokens` were absent. Because `prompt_tokens_details` was
present, Path A's mapper would store `cached_tokens` and drop
`cache_write_tokens` (not a field on `Usage`). On this host the hit count
still arrives, through `cached_tokens`. A miss count does not.

`cache_write_tokens` was `0` on every call below, including the full miss
(A1) and the full hit (A3). This host did not report cache creation as a
separate non-zero bucket.

---

## 3. An appended system message is expressible. A head rewrite is not cache-preserving.

The client can already emit a `system` role at any index.
`conversation_to_chat_messages` (`conversation/chat_completions.rs:186`) maps
each `ConversationItem::System` in order. The official message list is a
`oneOf` of system, user, assistant, and tool, with no "system must be first"
constraint on the schema page. The product's turn assembly does not use that
freedom: `apply_spec10_to_conversation_request` overwrites the leading system
item in place, and `tools_document()` is inlined into that body
(`spec10_path_a_assembly.rs:83` and `:467`).

Tools on the wire are a top-level `tools` array, replaced in full on every
request. There is no message role that carries a tool schema. dsh's
`tool_addition` / `tool_removal` history blocks have no counterpart on this
body.

**Live, same host and model.** Prefix bytes were held constant except where
the row says they changed. Hit counts for one identical body are not stable;
the guide calls the cache best-effort, and the table shows it.

| Call | Shape | `prompt_tokens` | `cached_tokens` |
|---|---|---:|---:|
| B append a turn | system, user, assistant, user | 792 | 712 |
| B again | same bytes | 792 | 768 |
| B warm | same bytes | 792 | 641 |
| F rewrite the leading system | new head + same tail | 799 | 0 |
| G splice the update into the leading system | one system message | 787 | 0 |
| G two leading system messages | stable system, then the update, then user | 786 | 683 |
| C insert system before the last user | …, assistant, system, user | 811 | 721 |
| C again | same bytes | 811 | 0 |
| C insert, later sample | same bytes | 811 | 729 |
| C insert, repeat | same bytes | 811 | 656 |
| C append system after the whole history | …, user, system | 813 | 699 |
| D large `tools` array added to B | messages unchanged, tool text counted | 1611 | 1337 |
| D same tools, repeat | same bytes | 1611 | 1433 |

Every row was HTTP 200. The model replied. Head rewrites (F, G-concat) were
the only shapes that returned `cached_tokens = 0` on every sample, and the
concatenated head cost about three times the two-system call
(upstream prompt cost `7.1617e-5` vs `2.18036e-5` USD). A later
system message was accepted in every sample. Four of the five insert/append
samples kept a large hit; one identical insert returned 0. That is
best-effort preservation, not a guaranteed breakpoint.

Adding a large tool array raised `prompt_tokens` from 792 to 1611 and did
not zero `cached_tokens`. A tiny tool sent with `tool_choice: none` left
`prompt_tokens` at 792, so that particular body did not count the tool; it
is not evidence about tool caching. There is still no in-history tool
event. The expressible tool update on this transport is a new full `tools`
array.

**What this does not show.** `api.deepseek.com` was not called. OpenRouter's
`cached_tokens` is the field this host returned; it may be OpenRouter's
accounting of the upstream prefix cache. The official field names in §2
stay the contract for a direct DeepSeek call until someone measures that
host. Identical replays moved by more than a hundred tokens (641, 712, 768
on the same 792-token body), so a unit cannot treat a single hit count as a
precise byte boundary.

**Implication for U3.1.** The transport can express `systemPromptUpdate:
'in-history'`: a second system message after history is legal on this route,
and rewriting the leading system is the shape that measured a full miss.
The unit survives for that shape. `toolUpdate` as a history event does not
have a wire form here. A changed `tools` array is the form that does, and
on this route it did not force the miss that a system-head rewrite did.
The product's current assembly would still defeat both, because a tool-set
change rewrites the leading system body through `tools_document()`. That is
an assembly fact for the unit, not a reason to drop it.

---

## 4. Which gated units survive

The cold start's five gated units, from this evidence:

| Unit | Survives on this line? | Because |
|---|---|---|
| **U2.1** Cache-key reachability | No further wire change. This note is the record. | §1. The key cannot land, and hits do not require it. |
| **U2.2** Cache-miss attribution | Yes, against the usage fields in §2. | A head rewrite measured 0 cached tokens and a stable replay measured hundreds. The signal is which prefix component changed, correlated with `cached_tokens` / `prompt_cache_hit_tokens`. It is not a server-side cache key. Counts vary on identical bodies, so the categories have to be coarse. The live assembly to instrument is `spec10_path_a_assembly.rs`, called from `turn.rs:3003`. |
| **U2.3** Cumulative session cache surface | Yes. | Per-turn `cached_tokens` is a real number to sum. Path A does not retain `prompt_cache_miss_tokens`, and OpenRouter did not send it. |
| **Scored cache regression bench** (board §1, no unit number) | Yes, with a wide threshold. | Head rewrite vs replay separates. A tight token threshold would flap: the same body returned 641, 712, and 768. |
| **U3.1** Prompt/tool update after cached history | Yes for an appended system message. | §3. A tool change is a full `tools` array, not a history event. |

U2.2, U2.3, and the bench still belong on the vendored turn path. They are
not blocked by the inert `prompt_cache_key`. They are blocked by the
coordination note already on the board: `feat/cache-attribution` is in
`crates/`, and Path A does not read `dsb-context` on the turn.

---

## 5. What this session did not do

- No change under `crates/` or `third_party/`.
- No version bump, CHANGELOG edit, tag, or npm publish. The `6.0.0` lane
  was already on `main`; npm still reported `5.7.0` at the start of the
  session, and this note leaves both alone.
- No second vendored build.
- No Anthropic Messages migration. The Messages mapping keeps system text in
  a top-level `system` field (`conversation/messages.rs:281`), which is a
  different body. That comparison is not a decision to switch.
