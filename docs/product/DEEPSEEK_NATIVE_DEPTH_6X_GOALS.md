# DeepSeek-native depth — **`6.1.0`** (line continuation, not a new major)

> [!NOTE]
> **PROPOSED.** This board has no execution authority until its PR merges and
> the owner accepts it. It is a **continuation of the `6.x` line**
> ([PRD-v6.md](./PRD-v6.md) §7), not a new major line.

| Field | Value |
|-------|--------|
| **Plan id** | `deepseek-native-depth-6x` |
| **SemVer path** | **`6.1.0`**, then later `6.x` minors as work lands |
| **Belongs to** | [PRD-v6.md](./PRD-v6.md) §7 — the `6.x` line |
| **Base** | `main` after the `1.0.0` → `1.0.41` port ([PR #200](https://github.com/innocarpe/deepseek-build/pull/200), merged `593a1e4`). The board's units are written against that tree. |
| **North star** | [VISION.md](./VISION.md) + [HARNESS_PHILOSOPHY.md](../architecture/HARNESS_PHILOSOPHY.md) |
| **Evidence** | [research/dsh-deepseek-harness.md](../research/dsh-deepseek-harness.md) · [research/upstream-gap-sweep-2026-09-25.md](../research/upstream-gap-sweep-2026-09-25.md) |
| **PR planning** | [ULTRAGOAL_PR_PLANNING.md](./ULTRAGOAL_PR_PLANNING.md) |
| **Cold start** | [ULTRAGOAL_PROMPT_COLD_START_DEEPSEEK_DEPTH_6X.md](./ULTRAGOAL_PROMPT_COLD_START_DEEPSEEK_DEPTH_6X.md) |
| **Ledger** | [UPSTREAM_SYNC_LEDGER.md](./UPSTREAM_SYNC_LEDGER.md) — this board's *not taken* list is its harness-idea counterpart |

**Do not** plan releases as `5.0.1`–`5.7.0` or `6.0.0` — those targets are used
on `main`, tagged, or owned by another lane.

---

## 0. Why this is `6.1.0` and not a new major

The rule is [versions/README.md](./versions/README.md) §Rules 1: *minors are
changelog + release notes, not new PRDs unless behavior identity shifts*. This
work adds no new surface, no new pillar, no new identity. It makes an existing
claim — "DeepSeek-native harness" — true in places where it currently is not.

**Two conditions would move this off the 6.x line:**

| Condition | Consequence |
|---|---|
| The wire investigation (U0.2) concludes the product should move to the **Anthropic Messages** transport | Identity-relevant → its own major (`7.0.0`), with a PRD. **Did not fire** (2026-09-26): [PRD-v6 §7.2](./PRD-v6.md) re-affirms [ADR 0005](../adr/0005-deepseek-provider-contract.md) |
| `6.x` minor numbers are already spent by base-port follow-up when this train starts | The depth work moves to the next free line |

---

## 1. What the base already has — measured, not assumed

The first draft of this board listed eight dsh items as "take". Re-measuring
against the `1.0.41` tree removed four of them: the base already does it. The
table below is the corrected inventory, with the evidence path for each.

| dsh item | State in the `1.0.41` tree | Verdict |
|---|---|---|
| Spill: oversized tool output → head/tail + locator | **Present.** `xai-grok-tools/src/types/output.rs:1276` emits `[truncated: showing first/last X of Y - full output at: <path>]`, plus structured `truncated`/`total_bytes`/`output_file`, and the field doc says to use `read_file` for the full text | **Dropped** — already there |
| Compaction that reuses the warm prefix | **Present.** `session/helpers/session_compact.rs:475,702` carries `// Prefix-cache alignment`, and `prepared_compaction_history.rs:1` reads *"Prepares one cache-aligned … compaction request history"* | **Dropped** — already there |
| Snippet staleness guard on edit | **Present and stricter than dsh.** `xai-grok-tools/src/types/snippet_store.rs` implements issue/require/expire against a full-file `hex(sha256(bytes))`; dsh uses a path-scoped version token | **Dropped** — this product's contract already wins |
| Background-task read surface | **Present.** `xai-grok-tools/src/bridge.rs` wires `get_task_output`, `task_ids_param`, `list_tasks` | **Dropped** — the "unify" idea had no measured gap under it |
| Runtime invariant: request equals the log projection | **Present as of the Wave 1 unit.** `request_log_invariant.rs` checks non-system items before `run_turn_via_sampler` (`turn.rs`). System messages are the Spec 10 overlay. A tool result may be the logged text, the hard-clear placeholder, or a head/tail trim. Anything else fails the turn. An item that cannot be serialized fails closed | **Taken** |
| Context attached *beside* a tool result | **Present in this tree.** `PostToolUse` `additional_context` is a separate `ConversationItem` via `wrap_hook_note` (`reminders.rs`), pushed in `tool_calls.rs` after the tool result. `post_tool_use_delivery_tests.rs` asserts block/context do not replace `model_output` | **Dropped** — already on the live path. Re-measured 2026-09-26; the 1.0.41 note that said absent is stale |
| Guard that may deny but never allow | **Present as of the Wave 1 unit.** `DenyOnly` in `permission/policy.rs` has deny and abstain. `combine_decisions` uses it, so an allow on either side cannot replace a reject or policy deny | **Taken** |
| Cache-miss **attribution** | **Present on Path A.** `observe_path_a_prefix_change` hashes the five documents `assemble_spec10_path_a_turn` concatenates, and `turn.rs` logs `prefix_change=` when the epoch differs from this session's previous assembly in this process. PR #235 | **Taken** |
| Session-cumulative cache surface | **Present as a log line.** `Usage.prompt_cache_miss_tokens` is kept. `CacheSessionTotals` on the in-memory session ledger logs `cache_session=` from `emit_turn_completed`. The chip at `agent_status.rs` is still the per-turn percentage. The counter is not persisted. PR #238 | **Taken** |
| Scored cache regression bench | **Present for the overlay mock.** Spec 10 §1.9, `crates/dsb-agent/tests/cache_guard.rs`, PR #229. It scores `dsb-context` bytes, not the bytes Path A sends. A live threshold is per route (wire inventory §6) | **Taken** for the mock |
| A changed prompt/tool set appended after cached history | **Present for the stable body.** Spec 10 §1.10 / `place_stable_body`. A later body is appended; the earlier system message stays byte-for-byte. A `tools` array replacement is not a history event | **Taken** — U3.1 |

**Two facts that shape everything below:**

1. **The product's live path is the vendored tree.** `xai-grok-shell` does not
   depend on `dsb-context`; it has its own
   `assemble_spec10_path_a_turn` / `tools_document` / `skills_document`, and
   `acp_session_impl/turn.rs` calls that. The overlay crate
   `dsb-context::assemble_path_a_context` is invoked at launch only to stamp an
   epoch file. **Any change that must reach the product goes in the vendored
   tree** ([OWNER_BAR_ACCEPTANCE §2.1](./OWNER_BAR_ACCEPTANCE.md): only Path A
   counts for product P0).
2. **Path A runs Chat Completions.** `dsb-cli/src/agent_launch.rs:174` (and six
   other stanzas) pin `api_backend = "chat_completions"`. In that tree,
   `ConversationRequest::prompt_cache_key` reaches the wire **only on the
   Responses mapping** (`xai-grok-sampling-types/src/types.rs:1068-1071`), and
   the code comments the consequence: *"a key that never reaches the wire looks
   like a 0% cache hit, not a bug."* Whether the product's cache keys are inert
   on its own backend is **U0.2's question** and the precondition for the cache
   units.

---

## 2. The thesis in one paragraph

The product claims to be a **DeepSeek-native harness**. Reading DeepSeek's own
official harness (`dsh`) showed what that claim still owes: dsh never lets a
session change rewrite a cached prefix — it appends after it — and it enforces
its context contract with runtime invariants rather than tests. This train
buys the half that the base has not already solved, in cost order, and it
refuses to claim cache wins without numbers.

---

## 3. Units

Ordered by dependency, not importance. One worktree, one branch, one PR each
([worktree-dispatch](../../skills/worktree-dispatch/SKILL.md)).

### Wave 0 — gates

| Unit | Deliverable | Dep |
|---|---|---|
| **U0.1** | This board and PRD-v6 §7, corrected against the `1.0.41` measurement | — |
| **U0.2** | **Wire-inventory investigation.** On `api_backend = "chat_completions"`, trace and measure: does `prompt_cache_key` reach the wire; what the DeepSeek usage payload reports for cache reads; whether an in-history prompt/tool update can be expressed at all. **Produces evidence, not a decision.** | none (Cargo-only, reads existing config and client code) |

**U0.2 decides the fate of three units.** If cache keys are inert on this
backend, the cache units are redefined as "make the key reach the wire first";
if an in-history update cannot be expressed, the prompt/tool unit is dropped
from this line and becomes `7.0.0` material.

**U0.2 result.** Full table:
[chat-completions-wire-inventory-2026-09-25.md](../research/chat-completions-wire-inventory-2026-09-25.md).
The 2026-09-25 pass (tree `687582c`) ran on OpenRouter: that environment had
no credential for `https://api.deepseek.com`. That limit is closed. On
2026-09-26 the same questions went to
`POST https://api.deepseek.com/chat/completions` (model `deepseek-chat`,
`thinking.type = disabled`, prefix about 1500 tokens, twelve HTTP 200
responses). See §6 of that note. The model id is not an ADR 0005 pin
(`deepseek-v4-flash` / `deepseek-v4-pro`).

`prompt_cache_key` still does not reach Chat Completions in this product,
and a prefix hit was observed without one on both hosts (OpenRouter A3 =
768/768; official A2 = A3 = 1280 hit / 228 miss on a 1508-token prompt).
Sending `prompt_cache_key` on the official host (row E) returned the same
1280/228, HTTP 200. U2.1 stays closed.

The official payload reports `prompt_cache_hit_tokens`,
`prompt_cache_miss_tokens`, and `prompt_tokens_details.cached_tokens`. On
every 2026-09-26 row the hit count equalled `cached_tokens`, and hit + miss
equalled `prompt_tokens`. OpenRouter sent only
`prompt_tokens_details.cached_tokens` and omitted the hit and miss names.
Path A's `Usage` still has no `prompt_cache_miss_tokens` field, so serde
drops the miss the official server sends. The server sends it. The client
discards it.

An appended system message is expressible. On the official host, two
different unseen leading systems (F) were each 0 hit / 1512 miss. Those two
calls are not a same-byte replay. Appending a system message after the user
turn and leaving the leading system unchanged (G) was 1280 hit / 244 miss
both times, on the same bytes (1280/1524 = 84%). Uncached input 1512 versus
244 is 6.2×. A `tools` array did not zero the cache (D: prompt 1508 → 1766,
hit 1408 then 1536 on the same bytes).

The OpenRouter sample was not token-stable (641, 712, 768 on one 792-token
body). That spread is why the 2026-09-25 row below called for a wide live
threshold. It is not how the official host behaved: A, B, and G repeated
exactly. D is the one official same-byte pair that moved, which fits a cache
still warming and does not prove it. The wide-threshold reason is gone on
the route `deepseek-build` / `dsb` uses when it talks to
`api.deepseek.com`. A live threshold has to differ by route. An OpenRouter
band is the wrong band for the official host, and the official sample is the
wrong band for OpenRouter. Spec 10 §1.9's landed 90% is a separate number —
Reasonix's threshold on a prefix-accounting mock — and this result does not
retune it.

[PRD-v6 §7.2](./PRD-v6.md) is closed by re-affirming ADR 0005. The §0
condition that would move this line to `7.0.0` did not fire. The official
host caches (1280-token hit), expresses an in-history system append (84%),
and does not need a cache key. ADR 0005's decision text is unchanged. The
evidence lives in the research note.

| Gated unit | After the official remeasure (2026-09-26) |
|---|---|
| U2.1 cache-key reachability | Closed. No further wire change. A key is unnecessary on the official host too. |
| U2.2 cache-miss attribution | Landed on Path A. `observe_path_a_prefix_change` hashes the five documents `assemble_spec10_path_a_turn` already concatenates, and `turn.rs` logs `prefix_change=` only when the epoch differs from this session's previous assembly in this process. `unattributed` is the coverage bug, not a sixth component. PR #209 remains the overlay (`crates/`) path. |
| U2.3 cumulative cache surface | Landed on Path A for the log line. `Usage.prompt_cache_miss_tokens` is kept, and `emit_turn_completed` logs `cache_session=` from the in-memory session ledger. The pager chip is unchanged. The persisted counter remains the overlay (`dsb-agent`) on the REPL / `run` path. |
| Scored cache bench (board §1) | The live OpenRouter spread (641/712/768) was the wide-threshold reason. That reason does not hold on `api.deepseek.com` same-byte replays (A, B, G exact; D moved once). Re-review any live threshold per route. Do not retune spec 10 §1.9's mock 90% from this table. |
| U3.1 in-history prompt/tool update | Premise confirmed on the official host for `deepseek-chat`. Two cold heads: 0/1512 each. Same-byte append: 1280/244 twice (84%, 6.2× uncached input versus the rewrite). Tool updates remain a `tools` array, and that array did not zero the cache. |

### Wave 1 — wire-independent, absent-by-measurement

| Unit | Deliverable | Dep | Size |
|---|---|---|---|
| **U1.1** | **Runtime invariant: request equals the log projection.** Done: `check_request_projects_log` runs before `run_turn_via_sampler`. Divergence fails the turn. Unserializable items fail closed | none | medium |
| **U1.2** | **Context beside a tool result.** Already on the live path (`wrap_hook_note` + `tool_calls.rs`). No second implementation | none | small |
| **U1.3** | **Deny-only guard.** Done: `DenyOnly` + `combine_decisions`. A later allow does not reopen a deny | none | small |

### Wave 2 — cache axis (after U0.2, coordinated)

| Unit | Deliverable | Dep |
|---|---|---|
| **U2.1** | **Cache-key reachability.** Whatever U0.2 finds: make the product's cache key actually land on its backend, or record why it cannot | U0.2 |
| **U2.2** | **Done.** Cache-miss attribution on Path A. On an epoch change, `turn.rs` logs which of the five assembled documents moved. Categories the assembly cannot see are not invented. Spec 10 Path A attribution | U0.2 · U2.1 |
| **U2.3** | **Done for the Path A log.** `prompt_cache_miss_tokens` is kept on Chat Completions `Usage`. The §1.5.2 line is logged once per turn from the in-memory session ledger. The chip is unchanged. The persisted REPL / `run` counter stays in `dsb-agent` | U0.2 |

> **Coordination note.** A separate session on `feat/cache-attribution` is
> already working on attribution, the cumulative surface and a bench
> (`docs/specs/10-cache-contract.md` + `crates/dsb-context/`, uncommitted at the
> time of writing). Its brief states *"this unit only touches `crates/` — the
> cache layer is the product workspace"*. **That premise is false for Path A**:
> the live assembly is `spec10_path_a_assembly.rs` in the vendored tree, and
> `crates/dsb-context`'s Path A function only stamps an epoch file. U2.2 must
> not duplicate it. Reconcile before starting: either that session redirects its
> work into the vendored tree, or it delivers the spec + tests and this board
> carries the Path A wiring.

#### Coordination for `cache-session-totals` and `cache-regression-bench` (2026-09-25)

Two worktrees are open for the units the spec left pending. Both were clean
at `1b9bb1c` when this was written (no commit ahead of that pin):
`feat/cache-session-totals` (spec 10 §1.5.2) and
`feat/cache-regression-bench` (spec 10 §1.9). Both briefs take the product
workspace (`crates/`) as the place the cache layer lives. That is the same
premise as the note above. Re-measured on this tree, it is false for Path A,
and the two units do not get the same instruction.

**What is already on the overlay, and what Path A runs.** U2.2 landed in PR
#209 under `crates/dsb-context`, `crates/dsb-agent`, and `crates/dsb-cli`.
The production log is `report_prefix_change` (`crates/dsb-cli/src/main.rs:603`),
called from `bind_session` (`main.rs:571`) when `Commands::Run` goes to
`run_once` (`main.rs:374`, `:764`) or `Commands::Chat` / `Repl` goes to
`run_repl` (`main.rs:388`). A search of
`third_party/grok-build/crates/codegen/xai-grok-shell/Cargo.toml` for
`dsb-context` and `dsb-agent` is empty. The turn Path A sends is
`apply_spec10_to_conversation_request`
(`xai-grok-shell/src/session/helpers/spec10_path_a_assembly.rs:430`), called
from `acp_session_impl/turn.rs:3003`. The overlay
`assemble_path_a_context` runs only inside `stamp_path_a_prefix_epoch`
(`crates/dsb-cli/src/agent_launch.rs:949`) and writes an epoch file.
[OWNER_BAR_ACCEPTANCE.md](./OWNER_BAR_ACCEPTANCE.md) §2.1: the bare
`deepseek-build` / `dsb` TUI is the only product P0 path. `dsb run`,
`dsb chat`, and `cargo test -p dsb-agent` / `dsb-context` are Path B and are
not cut evidence.

**`cache-session-totals` — continue in `crates/` (A), inside the fence the
spec already drew.** Spec 10 §1.5.2 names the REPL / `run` turn line as its
surface, and says the full-screen TUI status line is a later unit in the
vendored tree, which that contract does not govern. `run_once` and
`run_repl` call `dsb-agent`, so a `cache_session=` line printed there is
reached by Path B. The TUI does not read it.

Path A already sums cache reads for the session, and the sum is not the
§1.5.2 line. `UsageLedger::record_main_loop_call`
(`xai-chat-state/src/usage.rs:120`) folds each call's
`cached_prompt_tokens` into `cached_read_tokens`.
`try_get_session_usage` (`xai-chat-state/src/handle.rs:498`) is that session
bill. `try_get_prompt_usage` (`handle.rs:489`) clears when the prompt index
increments; the test
`prompt_usage_ledger_via_handle_resets_and_clears` keeps the session
`model_calls` after that clear. `x.ai/deepseek/status`
(`xai-grok-shell/src/extensions/deepseek.rs:81`) returns
`PromptUsage` built from `try_get_session_usage`. The pager chip passes
`totals.cached_read_tokens` and `totals.input_tokens` from that response
into `format_cache_hit_pct`. The ledger has no miss field and no
unreported-turn count. Chat Completions `TokenUsage` drops
`prompt_cache_miss_tokens` (U0.2). This session implements the REPL line the
spec named. It does not reimplement the chip, and a green `crates/` counter
is not the TUI.

**`cache-regression-bench` — the bytes under test move to the vendored
assembly (B).** Spec 10 §1.9 scores epoch count and a hit rate from the
request bytes a mock receives. Its test plan names
`cargo test -p dsb-context -p dsb-agent`. Those bytes are the overlay
builder. Path A's bytes are the system body after
`apply_spec10_to_conversation_request`, with `tools_document()` inlined into
the leading system message (`spec10_path_a_assembly.rs:83`). U0.2 measured
that rewriting that leading system returned `cached_tokens = 0` on every
sample, and that a later system message did not force that zero. The
2026-09-26 official-host pass confirms that split (two cold heads at 0/1512;
the same appended body at 1280/244 twice). It also retires the OpenRouter
spread as a reason to widen a live threshold on `api.deepseek.com`. A live
threshold has to be chosen per route. Spec 10 §1.9's 90% stays the mock's
number. A negative control that perturbs `dsb-context` inputs does not
perturb the body
`turn.rs` sends, so it stays green when Path A breaks. The rate arithmetic
can stay a pure function. The scenario's request bytes are the output of
`assemble_spec10_path_a_turn` and the Chat Completions mapping. A guard that
only tests `dsb-context` is Path B evidence under OWNER_BAR §2.1.

### Wave 3 — wire-gated

| Unit | Deliverable | Dep |
|---|---|---|
| **U3.1** | **Stable-body update appended after cached history.** Spec 10 §1.10. A later change of the assembled body appends a system message and leaves the earlier one byte-for-byte. A `tools` array replacement is not a history event. `replace_or_insert_system_head` is outside this unit | U0.2 |

### Wave 4 — honesty and cut

| Unit | Deliverable | Dep |
|---|---|---|
| **U4.1** | **Done.** §7 is the *not taken* ledger: each dsh idea this train declined, with the evidence path and the reason. The discipline [UPSTREAM_SYNC_LEDGER.md](./UPSTREAM_SYNC_LEDGER.md) set for vendor syncs, applied to harness ideas | — |
| **U4.2** | Honesty table and owner summary are in [CHANGELIST_6_1_0.md](./CHANGELIST_6_1_0.md) and PRD-v6 §7. The tag `v6.1.0` is not cut | claimed units |

---

## 4. Sequencing rules

1. **U0.2 before Wave 2 and Wave 3.** No cache claim and no in-history design
   before the transport question is answered.
2. **Wave 1 does not wait for anything** — those three gaps are confirmed
   absent and need no wire change.
3. **Any product-behavior change lands in the vendored tree.** A change that
   only touches `crates/dsb-*` cannot control Path A unless it is wired into the
   vendored code that owns the path.
4. **One Grok build at a time across all worktrees** — a cold vendored build is
   30–60+ min. Wave 1 units touch the vendored tree, so they serialize with each
   other and with any other vendor work.
5. **Vendor changes follow the carried-patch discipline** ([GROK_VENDOR.md](../architecture/GROK_VENDOR.md)):
   in-tree edits are listed, and the next `grok-sync` re-applies them.

---

## 5. Per-unit honesty requirements

Every PR answers, in its body:

- **Cache impact** — none / low / medium / high, and *why* ([HARNESS_PHILOSOPHY](../architecture/HARNESS_PHILOSOPHY.md) §12). For Wave 2 this is the central question.
- **Which layer** — L1, L2, or L3.
- **Path A reachability** — does this change the code the installed `deepseek-build` / `dsb` actually runs? Name the call site, or say it does not.
- **Measured or asserted** — for cost or reuse claims, name the measurement.
- **What was not taken** — the dsh rows this unit declined, with the reason.

---

## 6. Stop conditions

Re-plan if any of these become true:

- **U0.2 finds the cache key cannot reach this backend at all.** Then the cache
  axis is a transport problem, not an attribution problem, and its priority
  changes.
- **U0.2 finds the transport cannot express an in-history update.** Drop U3.1
  from this line; it becomes `7.0.0` material.
- **U1.1 cannot be placed where the request is actually built.** If the only
  reachable seam is outside the live path, the unit is Path B theater and must
  be redesigned rather than shipped.
- Two consecutive units cannot be measured — the measurement story is wrong,
  not the units.

---

## 7. What this train deliberately does not do

This is the harness-idea counterpart of [UPSTREAM_SYNC_LEDGER.md](./UPSTREAM_SYNC_LEDGER.md): each row is a thing the dsh sweep put in front of this train, the place the evidence lives, and the decision. A later session inherits the judgment instead of re-reading dsh to rediscover it.

| Idea | Evidence | Decision | Why |
|---|---|---|---|
| Request-series bookkeeping (`initial` / `resume` / `change` / `series`, `surfaceOp`, `contentGeneration`) | [dsh-deepseek-harness.md](../research/dsh-deepseek-harness.md) §5 | **Not taken** | dsh reconstructs request identity because the prompt and the tools are history, and compaction can shadow that history. This product's instrument is the prefix hash (spec 10 §1.5). The series would be a second identity for a request model this product does not use. |
| Agent teams (roster, mailbox, task board, `waitForChange`) | [dsh-deepseek-harness.md](../research/dsh-deepseek-harness.md) §5 · [NON_GOALS.md](./NON_GOALS.md) | **Not taken** | A durable coordination layer for a browser UI that has to remember agents nobody is watching. This product's L3 is worktree plus subagent fan-out. NON_GOALS defers that class. |
| Sandbox escalation | [dsh-deepseek-harness.md](../research/dsh-deepseek-harness.md) §4, `escalation.ts:28-30,166-209` | **Not taken** | One wider-only retry, with a justification, approved before it runs. That needs sandbox modes to escalate between. This product has allow / deny / ask and no such modes. |
| Everything-is-a-plugin | [dsh-deepseek-harness.md](../research/dsh-deepseek-harness.md) §1 shape table and §5 | **Not taken** | Cordis, on the order of 90 service keys. That is what makes dsh's seams possible and its boot path long. This product ships a Rust pager and an overlay, not a plugin host. |
| Anthropic Messages transport | [dsh-deepseek-harness.md](../research/dsh-deepseek-harness.md) §6 · [wire inventory](../research/chat-completions-wire-inventory-2026-09-25.md) §6 · [PRD-v6 §7.2](./PRD-v6.md) · [ADR 0005](../adr/0005-deepseek-provider-contract.md) | **Not taken.** Re-affirmed 2026-09-26 | Official Chat Completions, model `deepseek-chat`, twelve HTTP 200 calls: a stable prefix hit 1280/228 on a 1508-token prompt, an appended system message kept 1280/1524 (84%), and a `prompt_cache_key` did not change the hit. The §0 condition that would open a `7.0.0` major did not fire. Effort levels, signed thinking blocks, and image handling were not measured, and they are not recorded as a reason to switch. |
| Webhook runtime, session query tools, schedule, deliverables | [dsh-deepseek-harness.md](../research/dsh-deepseek-harness.md) §5 | **Not taken** | Surfaces with no current product need. |
| Cordis / HMR / live profile patching | [dsh-deepseek-harness.md](../research/dsh-deepseek-harness.md) §5 | **Not taken** | Patches replace whole config rows. Not a model for this repo. |
| Spill, compaction cache alignment, snippet staleness, task-output surface | This board §1 | **Not re-implemented** | Measured present in the `1.0.41` tree. The snippet contract is stricter than dsh's path-scoped version token. |
| New TUI surfaces | This board §0 | **Not taken** | The train is depth. The §1.5.2 chip stays the existing per-turn percentage. The new session line is a log, not a new pane. |
| A new major line / a new PRD | [versions/README.md](./versions/README.md) §Rules 1 | **Not taken** | Identity did not shift. The Messages condition in §0 did not fire. |
