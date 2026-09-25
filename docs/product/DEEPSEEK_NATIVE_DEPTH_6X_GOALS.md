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
| The wire investigation (U0.2) concludes the product should move to the **Anthropic Messages** transport | Identity-relevant → its own major (`7.0.0`), with a PRD |
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
| Runtime invariant: request equals the log projection | **Absent.** `xai-chat-state` and `xai-grok-sampler` use "invariant" only for their own local rules; nothing checks the outgoing request against the session log | **Take** |
| Context attached *beside* a tool result | **Absent.** `xai-grok-hooks` has `PostToolUse`/`PostToolUseFailure` events but no path that adds model-visible context alongside a result | **Take** |
| Guard that may deny but never allow | **Absent.** No deny-only guard in `xai-tool-runtime` | **Take** |
| Cache-miss **attribution** | **Absent.** Epoch changes are logged as a new hash; nothing records *which component* moved | **Take** (see the coordination note in §3) |
| Session-cumulative cache surface | **Partial.** `xai-grok-pager/src/views/agent_status.rs:353` has `format_cache_hit_pct` — a *per-turn* `cache 45%` chip; no cumulative counter | **Take** (coordination note) |
| Scored cache regression bench | **Absent.** Prefix-equality goldens exist; no scenario bench with a threshold | **Take** (coordination note) |
| A changed prompt/tool set appended after cached history | **Absent, and wire-dependent.** `spec10_path_a_assembly.rs:83` assembles the stable body with `tools_document()` inlined; there is no update path | **Take — gated on U0.2** |

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

**U0.2 result (2026-09-25), measured at `687582c`.** Full table:
[chat-completions-wire-inventory-2026-09-25.md](../research/chat-completions-wire-inventory-2026-09-25.md).
`prompt_cache_key` does not reach Chat Completions, and a prefix hit was
observed without one, so U2.1 does not survive as a wire change. Cache reads
are still reported (`prompt_cache_hit_tokens` /
`prompt_cache_miss_tokens` on the official schema;
`prompt_tokens_details.cached_tokens` on the OpenRouter route that was
actually called). Path A keeps the hit and drops the miss. An appended
system message is expressible; rewriting the leading system measured
`cached_tokens = 0`. A tool update has no history-event form; it is a full
`tools` array.

| Gated unit | After U0.2 |
|---|---|
| U2.1 cache-key reachability | Closed by the note. No further wire change. |
| U2.2 cache-miss attribution | Survives, against usage fields, in `spec10_path_a_assembly.rs` / `turn.rs`. PR #209 merged the overlay attribution in `crates/` only; `origin/main` at `1b9bb1c` still does not touch the vendored turn. |
| U2.3 cumulative cache surface | Survives. No miss count is retained on Path A. |
| Scored cache bench (board §1) | Survives, with a wide threshold. Identical bodies were not token-stable. |
| U3.1 in-history prompt/tool update | Survives for an appended system message. Tool updates are a `tools` array. |

### Wave 1 — wire-independent, absent-by-measurement

| Unit | Deliverable | Dep | Size |
|---|---|---|---|
| **U1.1** | **Runtime invariant: request equals the log projection.** Before dispatch, check that what will be sent matches what the session log says was sent; fail loudly on divergence. Fail-close on log entries that cannot be restored. **Lands in the vendored tree**, at the sampling boundary — not in the overlay | none | medium |
| **U1.2** | **Context beside a tool result.** A result-preserving path for notices (repeat-call nudges, policy notes) so they do not have to be stuffed into the tool's own output | none | small |
| **U1.3** | **Deny-only guard.** A policy hook that may deny or abstain but can never allow, so listener order cannot resurrect a refused action | none | small |

### Wave 2 — cache axis (after U0.2, coordinated)

| Unit | Deliverable | Dep |
|---|---|---|
| **U2.1** | **Cache-key reachability.** Whatever U0.2 finds: make the product's cache key actually land on its backend, or record why it cannot | U0.2 |
| **U2.2** | **Cache-miss attribution.** On an epoch change, record which component moved — and make the categories ones that are actually distinguishable, not invented | U0.2 · U2.1 |
| **U2.3** | **Cumulative session cache surface.** The per-turn chip exists; add the session-level view | U0.2 |

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

### Wave 3 — wire-gated

| Unit | Deliverable | Dep |
|---|---|---|
| **U3.1** | **Prompt/tool updates appended after cached history** — the highest-value item in the sweep, and the one that cannot be specified until U0.2 answers whether the transport can express it | U0.2 |

### Wave 4 — honesty and cut

| Unit | Deliverable | Dep |
|---|---|---|
| **U4.1** | The *not taken* list, with the evidence path for each row of §1 — the discipline [UPSTREAM_SYNC_LEDGER.md](./UPSTREAM_SYNC_LEDGER.md) established for vendor syncs, applied to harness ideas | — |
| **U4.2** | `6.1.0` cut: honesty table filled from merged results, owner-readable summary, release | claimed units |

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

| Not done | Why |
|---|---|
| Spill, compaction cache alignment, snippet staleness, task-output surface | Measured present in the `1.0.41` tree — §1 |
| Anthropic Messages migration | Investigation (U0.2), not a commitment — §0 |
| A new major line / new PRD | Rule: a minor is not a new PRD unless identity shifts |
| Agent teams / mailboxes / task boards | Serves a browser UI this product does not have; deferred by [NON_GOALS](./NON_GOALS.md) |
| Sandbox modes + escalation | Correct vocabulary, no substrate |
| dsh's request-series bookkeeping | Precise instrument for a model this product does not use |
| Everything-is-a-plugin | Another product's architecture; not portable to a Rust pager |
| New TUI surfaces | This train is depth, not surface |
