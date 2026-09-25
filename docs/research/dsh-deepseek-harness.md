# DeepSeek Harness (dsh) — what DeepSeek's official harness has that this product does not

**Nature:** Non-binding evidence ([SOURCES.md](../product/SOURCES.md) precedence).
A judgment table, not a product commitment. Promote an item into
`docs/specs/`, `docs/product/`, or an ADR before building it.

**Question asked:** DeepSeek ships an official open-source agent harness
(`dsh`). What are its core designs, and which belong in DeepSeek Build?

**Subject:** [`deepseek-ai/deepseek-harness`](https://github.com/deepseek-ai/deepseek-harness)
at `477b4f4` (= `0.1.7-rc.2`), read 2026-09-25 from a shallow clone at
`OpenSources/_upstream/deepseek-harness`. The published npm `latest` at the
same time was `0.1.7-rc.1`.

**Method.** Six parallel read-only surveys of the source tree — LLM provider
layer; session/prompt/cache lifecycle; tool surface and execution pipeline;
approval/sandbox/skills/plan/goal/guard; subagents/jobs/terminals/SSH; and
extension/config/hooks/testing. Every claim below carries a file path from
that tree; the survey reports are summarized here and the raw findings stay
in the session record. dsb's side was read against its own `docs/specs/`,
`crates/`, and `third_party/grok-build/`.

> **One line.** dsh is not a better *agent* than this product — it is a
> different *architecture* (TypeScript, everything-is-a-plugin, browser-first)
> that has solved several problems this product has not yet reached, all of
> them in one family: **keeping a cached prefix alive while the session
> changes**. Its second contribution is a culture of executable invariants.

---

## 1. What dsh is, in shape

| | dsh | dsb |
|---|---|---|
| Language / runtime | TypeScript on Node, Cordis plugin tree (its primer describes a service-key/effect system) | Rust; vendored Grok Build pager + `dsb-*` overlay crates |
| Composition | **Everything is a plugin.** ~60 package groups, ~90 service keys; bundles → profile → patch overlays | Two Cargo workspaces; overlay crates + 14 carried patches on the vendor tree |
| Entry points | `web` (browser UI), `headless`, `sdk`, `acp` — **no official TUI** | Full-screen TUI is the product |
| Extension model | Register on a `ctx.*` service key; registrations are reversible effects | Patch files + overlay crates + vendored local patches |
| Model transport | **Anthropic Messages subset** on `api.deepseek.com/anthropic` | Chat Completions ([ADR 0005](../adr/0005-deepseek-provider-contract.md)) |
| Where it is strong | Session/prompt/cache discipline; tool pipeline; provider wire detail | TUI, packaging, install story, subagent/worktree throughput |

This is a developer preview that states plainly it will break compatibility
(`README.md`). Nothing here is a reason to change the product's shape; the
value is in specific mechanisms.

---

## 2. Axis 1 — The cache-preserving context lifecycle

dsb's [spec 10](../specs/10-cache-contract.md) makes the stable prefix
**byte-stable and hashed** (an epoch). dsh goes one step further in the same
direction: when something in the prefix *must* change, it does not rewrite the
prefix — it **appends after the cached history**.

| Mechanism | What dsh does | dsb today | Verdict |
|---|---|---|---|
| **Prompt is a surface node, not a request field** | The rendered prompt is `system/message` at surface node 0; `EpochHeader.system` is typed `never` (`session.md:194-200`). A prompt edit cannot churn the request envelope. | System prompt is part of the stable prefix; a change moves the epoch. | **Take** — the cleanest single idea in the tree. An epoch is honest, but a prompt edit should not have to cost the whole prefix. |
| **`systemPromptUpdate: 'in-history'`** | Routes declaring this capability receive a *changed* prompt appended after cached history, so the prefix up to that point is reused (`llm/llm/src/types.ts:396`; `agent-loop/README.md:162`). | Prompt change → new prefix. | **Take (largest item)** — requires a wire that tolerates a mid-history system message; needs provider verification before it is a contract. |
| **Tool changes as history events** | Tool additions/removals are `developer/message` `tool_addition`/`tool_removal` blocks referencing the defining header by `headerSeq` (`session.md:55-61`); adding a tool does not rewrite the tool schema head. | Tool-set change → epoch bump. | **Take (conditional)** — same dependency on the wire as above. Value is real: MCP mounts and skill-driven tool changes currently cost a full prefix. |
| **Policy state rides runtime context** | Approval/sandbox policy changes append a sourced snapshot `user/message`; the `system/message` is never touched; an unchanged policy costs 0 tokens (`approval.md:48`; `sandbox-policy/README.md:78-88`). | Permission mode changes are not represented in context at all. | **Take** — cheap, immediately useful, no wire dependency. |
| **Retry reuses the assembly** | A retry repeats neither `agent/pre-step` nor prompt assembly; it reuses the frozen rendered assembly (`agent-loop/README.md:119`). | Not specified. | **Take (small)** — a correctness property as much as a cache one. |
| **Compaction reuses the warm prefix** | The summarizer replays the exact system prompt, last routed tools, and shadowed messages byte-for-byte so the auxiliary call is a genuine prefix of the conversation (`compaction-basic/README.md:108`). | Compaction is an M1 stub. | **Take (design intent)** — when compaction is designed, this is the bar: the summarizer must not be a cold call. |
| **Request series (`request/header` reasons)** | `initial`/`resume`/`change`/`series`, with `startsSeries` declarations that wrapping listeners must preserve; a surface replacement after attachment starts a new series (`session.md:183`; `agent-loop/README.md:121`). | Epoch hashing only. | **Hold** — it is a precise instrument for a problem dsb mostly does not have yet (see §5). |
| **Prompt assembly with centered ordering** | Sections registered by name; `order` values come from a central table, not from contributors picking numbers (`system-prompt/README.md:45-49,107`). | Prefix sections are code-ordered. | **Take (small)** — a convention that prevents silent ordering churn. |

**What this axis is really about.** dsb treats the prefix as a document that
gets rebuilt when inputs change. dsh treats the *conversation* as append-only
and the prefix as a window over it. The second model is strictly more
cache-friendly, and it is the reason dsh's compaction and policy changes do
not move a byte of the head.

---

## 3. Axis 2 — Invariants that run

dsh does not only test its context contract; it **checks it on every request**
and refuses to proceed when it is violated.

- `agent-loop/src/invariant.ts:23-57` hooks `llm/stream` with `prepend: true`
  and fails the request when: the request is not frozen, messages are not
  frozen, `request/header` is missing from the log, `system` is present on the
  request, header fields disagree with the log — or
  `JSON.stringify(options.messages) !== JSON.stringify(deriveMessages())`
  ("log-reconstruction desync").
- The session log validates `seq` monotonicity, turn/step scoping, and
  tool/result ↔ tool/call pairing **before commit** (`session/src/invariant.ts:59-172`).
- An unknown event type without `ignorable: true` must **refuse to restore**
  the session — the default is fail-close, because continuing silently
  produces a gutted session (`session.md:276-285`).
- Every package ships an `./invariant` companion; `verify-package-invariants`
  mechanically rejects empty checkers and mis-wired registration.

| Item | dsh source | dsb today | Verdict |
|---|---|---|---|
| Request/log equality invariant | `agent-loop/src/invariant.ts` | Spec 100 JSONL replay; no runtime check that the wire equals the projection | **Take** — the highest-value idea after Axis 1. It turns "model-visible means logged" from a doc sentence into a thing the process cannot violate. |
| Fail-close on unknown log events | `session.md:276-285` | Not specified | **Take (small)** — a one-line policy with a real failure mode behind it. |
| Pre-commit event validation | `session/src/invariant.ts:241-249` | Not specified | **Take** — validates before publish, staging the transition and discarding it if a later listener refuses. |
| Package-attributed invariant registry | `runtime-diagnostics/invariants` | — | **Hold** — the npm-package-name attribution is dsh-shaped; the portable part is "a checker per subsystem, registered and verified". |

---

## 4. Axis 3 — Tool result and long-running-work contracts

| Item | What dsh does | dsb today | Verdict |
|---|---|---|---|
| **Spill** | Tool results over `maxInlineTokens` are replaced by ordered head/tail plus a **file locator and retrieval hint**; the full result stays on disk; failure keeps the original inline (best-effort) (`spill-policy/README.md:44-58`; `spill.md:94`). Append-only, so cache-safe. | Large `grep`/`read`/bash output goes inline. | **Take** — direct context-cost win; the fail-open rule is the right one. |
| **Jobs as one controller** | Background bash, PTY sends, and subagents are all **jobs** — started, listed, read, killed through the same three tools, with a bounded output ring and a consumer cursor separate from the observer cursor (`jobs.md:126-179`). | Background shell and subagents are separate surfaces. | **Hold (partial take)** — the unified *controller* is worth taking; the ring/cursor machinery is more than the product needs today. |
| **`additionalContexts`** | A post-execute decision can attach context that rides alongside the tool result instead of replacing it — e.g. repeat-call reminders keep the original result intact (`repeat-tool-reminder/README.md:90-96`). | — | **Take** — small, and it fixes the common hack of stuffing notices into tool output. |
| **Monotonic guards** | A guard can **deny or abstain, never allow** — so listener ordering can never resurrect something a stricter layer refused (`tools.md:326-338`). | Deny/ask rules exist; no ordering law. | **Take** — one sentence, prevents a whole class of policy-bypass bugs. |
| **Read-before-write with version tokens** | The filesystem owns an opaque `FsVersion`; the observation policy refuses to edit an unseen file (`FS_NOT_OBSERVED`) or a stale one (`FS_STALE_VERSION`) with an explicit "re-read the file, then retry" message; the version check runs **before** literal matching so a stale edit reports staleness, not a confusing no-match (`filesystem.md:151-190`, `tool-fs/README.md:74-76`). | Snippet store: session-owned `snippet_id` + full-file hash (stricter on identity). | **Take (two details)** — checker-before-matcher ordering and the fixed recovery sentence. The snippet contract stays. |
| **Sandbox escalation** | When confined, the model may retry **once** with `sandbox_permissions` + `justification`; escalation is strictly wider-only and approved before execution; a rejection tells the model to stop and explain, not work around it (`escalation.ts:28-30,166-209`). | allow/deny/ask only. | **Hold** — the vocabulary is right; dsb has no sandbox modes to escalate between yet. Revisit with a sandbox story. |
| **Tool catalog churn 0** | `exit_plan_mode` stays in the schema while plan mode is inactive; `lsp` keeps its schema when no provider is registered and fails at execution (`tool-catalog.md:593,1716`). | — | **Take (small)** — a discipline that keeps the tool head stable. |
| **Error taxonomy** | Structured `{name, code}` is kept durable while the model sees `Error: <message>`; unknown/denied/failed each normalize to one envelope (`tools.md:339-345,428-430`). | Partial. | **Take (small)** — shape, not machinery. |
| **`EMPTY_RESPONSE` is an error** | A terminal stop with no content blocks is a retryable canonical failure, not a silent success (`translate.ts:149`). | Not observed. | **Take (small)** — cheap, and it removes a silent-failure class. |
| **Spill/truncation three layers** | Source truncation (tail + spill file), result policy (`maxInlineTokens`), compaction-time pruner (`thresholdChars 8192`, head/marker/tail) — each with a different unit and trigger (`compaction-tool-result-pruner/README.md:49-52`). | One layer at most. | **Take (concept)** — the separation of "at result time" vs "at pressure time" is the useful part. |

---

## 5. Where dsh is weaker, or solving a problem dsb does not have

Recording this matters as much as the take list — several of dsh's more
elaborate mechanisms exist to pay for its own architecture.

| dsh mechanism | Why it is not a dsb target |
|---|---|
| Request **series** bookkeeping (`initial/resume/change/series`, `surfaceOp` shadowing, `contentGeneration`) | dsh must reconstruct request identity because prompt and tools are *history*, and history can be shadowed by compaction. dsb's prefix hash is a simpler instrument for a simpler model. Adopt Axis 1 first; revisit this only if the request model moves. |
| **Agent teams** (roster, mailbox, task board, `waitForChange`) | A durable coordination layer over continuable subagents. It exists because dsh's Web UI needs multi-agent state nobody is watching. dsb's L3 is worktree + subagent fan-out; the product explicitly defers this class ([NON_GOALS](../product/NON_GOALS.md)). |
| **Everything-is-a-plugin** | A genuine architectural strength for dsh and not portable to a Rust pager: it is what makes dsh's ~90 seams possible *and* what makes its boot path 90 rows deep. dsb's overlay model trades that extensibility for a shippable binary. |
| **Anthropic Messages transport** | See §6 — a separate investigation, not an automatic take. |
| **Webhook runtime, session query tools, schedule, deliverables** | Useful surfaces with no current product need. Noted, not taken. |
| **Cordis/HMR/live profile patching** | Its cost is visible in dsh's own docs (patches replace whole config rows, no deep merge). Not a model for dsb. |

**Where dsb is stronger, for the record.** The snippet edit contract is
stricter than dsh's `edit` + observation policy (session-owned identity with a
full-file hash versus a path-scoped version token). dsb's side-effect
classifier with declared effects is stricter than dsh's approval modes plus
hooks. dsb ships a real TUI, an install story, and vendored-tree sync
infrastructure that dsh does not attempt.

---

## 6. The wire question — Anthropic Messages vs Chat Completions

dsh's own DeepSeek adapter speaks the **Anthropic Messages subset** at
`https://api.deepseek.com/anthropic` (`llm-deepseek/config.ts:105`), not Chat
Completions. Its wire differences are not cosmetic:

- `thinking: {type}` + `output_config.effort` with exactly four levels
  (`off|low|high|max`), rejected before network I/O when unsupported
  (`serialize.ts:146-153`).
- Reasoning replays as a **signed `thinking` block**, not `reasoning_content`.
- Usage is **disjoint by construction**: `inputTokens` is uncached input only;
  cache reads/writes are separate fields, and the adapter subtracts
  DeepSeek's folded `prompt_tokens` back out (`llm/src/types.ts:167-189`).
- `cache_read_input_tokens` / `cache_creation_input_tokens` on the response.
- Capability flags govern prompt/tool update strategy:
  `systemPromptUpdate: 'in-history'`, `toolUpdate: 'addition-only'`.

**This is recorded as an investigation item, not a verdict.** Two facts pull
in opposite directions: dsh is DeepSeek's own harness and chose this path, and
dsb's [ADR 0005](../adr/0005-deepseek-provider-contract.md) pinned Chat
Completions against live evidence. Which is right for this product depends on
what the two paths actually return today — cache accounting, effort levels,
thinking replay, and image handling — measured on dsb's own routes (the
official `api.deepseek.com` endpoint and OpenRouter). That measurement is its
own unit of work; nothing in Axis 1–3 should wait for it, except that
`systemPromptUpdate`/`toolUpdate` cannot be specified until it is settled.

**Where this landed:** the judged items are scoped as the **`6.1.0`**
continuation of the `6.x` line — [PRD-v6 §7](../product/PRD-v6.md) and its
board [DEEPSEEK_NATIVE_DEPTH_6X_GOALS.md](../product/DEEPSEEK_NATIVE_DEPTH_6X_GOALS.md).
Not a new major: per [versions/README.md](../product/versions/README.md)
§Rules 1 a minor is not a new PRD unless behavior identity shifts, and this
work adds no surface and no identity — it makes an existing claim true. **This
§6 wire question is the one condition that would move the work to its own
major**, because changing the transport is identity-relevant.

---

## 7. What this sweep does not establish

- **No live-wire verification.** Every dsh claim is read from its source and
  docs. The Messages-vs-Chat-Completions comparison in §6 is a reading of
  dsh's code, not a measurement of what the endpoint returns.
- **Shallow clone.** `477b4f4` is a single commit; "recent" is not dated
  evidence, and `0.1.7-rc.2` is a release candidate that states it will break
  compatibility. Any adopted idea must be re-checked against a later release
  before it becomes a contract.
- **Cost was not measured.** dsh documents *why* its layout should be
  cache-friendly; it does not publish hit rates, and this sweep did not run
  either harness against a billed account.
- **The raw survey reports are not reproduced here.** Six surveys produced
  several thousand lines of file-cited findings; this document keeps the
  judged subset. Anything promoted into a spec needs its evidence re-read
  from the tree at that time.
- **dsh-side quality judgments are out of scope.** Whether dsh is a good
  product is not the question; several of its mechanisms are good answers to
  problems this product will otherwise rediscover.
