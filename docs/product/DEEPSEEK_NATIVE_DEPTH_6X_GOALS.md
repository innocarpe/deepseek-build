# DeepSeek-native depth — **`6.1.0`** (line continuation, not a new major)

> [!NOTE]
> **PROPOSED.** This board has no execution authority until its PR merges and
> the owner accepts it. It is a **continuation of the `6.x` line**
> ([PRD-v6.md](./PRD-v6.md) §7), not a new major line: `main` carries
> **`5.7.0`**, and the `6.0.0` base port (PR #200, `1.0.0` → `1.0.41`) is still
> open.

| Field | Value |
|-------|--------|
| **Plan id** | `deepseek-native-depth-6x` |
| **SemVer path** | **`6.1.0`**, then later `6.x` minors as work lands |
| **Belongs to** | [PRD-v6.md](./PRD-v6.md) §7 — the `6.x` line |
| **Depends on** | **`6.0.0` base port merged** for any unit touching `third_party/grok-build/`; wire-independent units may start earlier |
| **North star** | [VISION.md](./VISION.md) + [HARNESS_PHILOSOPHY.md](../architecture/HARNESS_PHILOSOPHY.md) |
| **Evidence** | [research/dsh-deepseek-harness.md](../research/dsh-deepseek-harness.md) · [research/upstream-gap-sweep-2026-09-25.md](../research/upstream-gap-sweep-2026-09-25.md) |
| **PR planning** | [ULTRAGOAL_PR_PLANNING.md](./ULTRAGOAL_PR_PLANNING.md) |
| **Ledger** | [UPSTREAM_SYNC_LEDGER.md](./UPSTREAM_SYNC_LEDGER.md) — this board's *not taken* list is its harness-idea counterpart |

**Do not** plan releases as `5.0.1`–`5.7.0` or `6.0.0` — those targets are used
on `main`, tagged, or owned by an in-flight train.

---

## 0. Why this is `6.1.0` and not a new major

The rule is [versions/README.md](./versions/README.md) §Rules 1: *minors are
changelog + release notes, not new PRDs unless behavior identity shifts*. This
work adds no new surface, no new pillar, no new identity. It makes an existing
claim — "DeepSeek-native harness" — true in places where it currently is not.

The dependency direction agrees. `6.0.0`'s own PRD §2 records what the base
move did to L2: **per-effort model identifiers, effort levels sourced from the
API, per-model retry configuration, request-size caps.** Those are exactly the
substrate this board's units build on. §2 strengthened the substrate; this
board spends it.

**Two conditions would move this off the 6.x line:**

| Condition | Consequence |
|---|---|
| The wire investigation (U5.1/U5.2) concludes the product should move to the **Anthropic Messages** transport | Identity-relevant → its own major (`7.0.0`), with a PRD |
| `6.x` minor numbers are already spent by base-port follow-up when this train starts | The depth work moves to the next free line |

---

## 1. The thesis in one paragraph

The product claims to be a **DeepSeek-native harness**. Reading DeepSeek's own
official harness (`dsh`) showed what that claim still owes: dsh never lets a
session change rewrite a cached prefix — it appends after it — and it enforces
its context contract with runtime invariants rather than tests. This train
buys that depth in cost order, cheapest and most independent first, and it
refuses to claim cache wins without numbers.

**Not in this train:** the model-transport question (Anthropic Messages vs Chat
Completions). It is a separate investigation with its own evidence bar.

---

## 2. Units

Ordered by dependency, not by importance. Each unit is one worktree, one
branch, one PR ([worktree-dispatch](../../skills/worktree-dispatch/SKILL.md)).

### Wave 0 — preconditions (no product code)

| Unit | Deliverable | Base dep | Parallel? |
|---|---|---|---|
| **U0.1** | This evidence + plan: research doc, PRD-v6 §7, board, index updates | none | — |
| **U0.2** | **Cache measurement harness** — a repeatable way to read prefix reuse, epoch, and cache fields from a real turn on a real route, so every later claim has numbers | none (Cargo-only) | **parallel-safe** |

### Wave 1 — wire-independent cache lifecycle (L1/L2)

These need no transport change. They are the cheapest half of dsh Axis 1.

| Unit | Deliverable | Dep | Notes |
|---|---|---|---|
| **U1.1** | **Policy state as runtime context** — approval/permission/sandbox state appended as a sourced context snapshot instead of being invisible or rewriting the head; unchanged policy costs nothing | U0.2 (for the "costs nothing" evidence) | Small, immediately useful, no wire dependency. |
| **U1.2** | **Prompt/tool change path** — move the prompt so a change can append after cached history rather than rebuild the head; keep the epoch model as the fallback | U0.2 · **U5.2** if the wire answer changes the design | Design must state which behavior applies when the route cannot do in-history. |
| **U1.3** | **Retry reuses the assembly** — a retry repeats neither pre-step nor prompt assembly; it reuses the frozen rendered assembly | none | Correctness property as much as a cache one. |
| **U1.4** | **Centered section ordering** — prompt/prefix sections registered by name with centrally allocated order values | none | Prevents silent ordering churn. |

### Wave 2 — attribution and honesty (L2)

| Unit | Deliverable | Dep | Notes |
|---|---|---|---|
| **U2.1** | **Cache-miss attribution** — on every epoch change, record a structured *reason* (what moved), not only the new hash. The first sweep already named this the highest-value L2 gap. | U0.2 | Spec 10 §1.5 extension. |
| **U2.2** | **Cumulative cache surface** — session-level hit/miss accounting, visible to the owner in a real session | U0.2 | Small; directly serves the cost culture. |
| **U2.3** | **Scored cache regression bench** — the existing prefix-equality goldens plus a scenario bench with a threshold, so "cache-first" is testable rather than aspirational | U0.2 | Makes U1.x claims durable instead of one-off measurements. |

### Wave 3 — contract enforcement (L1)

| Unit | Deliverable | Dep | Notes |
|---|---|---|---|
| **U3.1** | **The request-equals-log invariant** — a runtime check that the request payload matches the log-derived projection, failing loudly; plus fail-close on unknown log events | U2.1 (attribution makes failures diagnosable) | The highest-value item after Wave 1. Turns "model-visible means logged" into something the process cannot violate. |

### Wave 4 — tool result and policy shapes (L1/L3-lite)

| Unit | Deliverable | Dep | Notes |
|---|---|---|---|
| **U4.1** | **Spill** — large tool results become ordered head/tail plus a file locator and retrieval hint; full result on disk; fail-open | U0.2 | Direct context-cost win. Must not weaken snippet/edit safety. |
| **U4.2** | **`additionalContexts`** — a post-execute decision can attach context *alongside* a tool result instead of overwriting it | none | Small; removes the "stuff notices into tool output" hack. |
| **U4.3** | **Monotonic guards** — a guard may deny or abstain, never allow; ordering can never resurrect a refused action | none | One sentence of law; prevents a class of policy bypass. |
| **U4.4** | **Tool-shape discipline** — keep the tool head stable across mode changes (plan tool stays declared while inactive; provider-less tools keep their schema and fail at execution); structured error codes durable while the model sees one envelope; treat an empty model response as a retryable error | none | A bundle of small, independent disciplines. May split if the PR grows. |
| **U4.5** | **File-edit recovery details** — version check before literal matching, and a fixed "re-read then retry" recovery sentence | none | Snippet contract stays; these are ordering and message details. |
| **U4.6** | **Jobs as one controller** (scoped) — one list/read/kill surface across background shell, PTY, and subagents | U3.1 | Scope deliberately small: the unified *controller*, not dsh's full ring machinery. |
| **U4.7** | **Agent guardrails** — a subagent concurrency cap that **refuses** rather than queues, a default delegation depth of 1, and fork children inheriting the parent route so the copied prefix stays reusable | U3.1 | Small; each is a one-line policy with a cache or starvation reason behind it. |

### Wave 5 — the wire investigation (evidence only)

| Unit | Deliverable | Dep | Notes |
|---|---|---|---|
| **U5.1** | **Transport comparison** — measure Anthropic Messages vs Chat Completions on this product's routes for cache accounting, effort, thinking replay, images; state what in-history prompt/tool updates would cost or gain on each | U0.2 | **Produces evidence, not a decision.** |
| **U5.2** | **ADR 0005 disposition** — amend, or record why Chat Completions stays | U5.1 | Closes the question one way or the other. |

### Wave 6 — cut

| Unit | Deliverable | Dep |
|---|---|---|
| **U6.1** | Honesty table filled from merged results; owner-readable summary; `6.1.0` release cut and publish | all claimed units |

---

## 3. Sequencing rules

1. **Wave 0 before anything else.** No cache claim without U0.2.
2. **Vendor-touching units wait for `6.0.0`.** Units that only touch
   `crates/dsb-*` or docs run in parallel with the base port.
3. **One Grok build at a time across all worktrees** — this board's units are
   Cargo-overlay and docs work, so they mostly avoid the rule; U1.2 and U3.1
   may not.
4. **Wave 1 before Wave 3** — attribution first makes invariant failures
   diagnosable rather than merely loud.
5. **U5.x never blocks Waves 1–4.** If the wire answer changes the design of
   U1.2, U1.2 is revised in a follow-up; the train does not stall.

---

## 4. Per-unit honesty requirements

Every PR in this train answers, in its body:

- **Cache impact** — none / low / medium / high, and *why*
  ([HARNESS_PHILOSOPHY](../architecture/HARNESS_PHILOSOPHY.md) §12).
- **Which layer** — L1, L2, or L3.
- **Measured or asserted** — if a claim is about cost or reuse, name the
  measurement; if it was not measured, say so.
- **What was not taken** — the dsh rows this unit declined, with the reason.

---

## 5. Stop conditions

Stop the train and re-plan if any of these become true:

- U0.2 shows the prefix is **not** in fact stable/observable on the routes the
  product ships — the whole thesis would need re-derivation.
- U5.1 shows the two transports are equivalent for this product's purposes —
  then U1.2's in-history path is unnecessary and the train shrinks.
- The `6.0.0` base port changes the request/assembly code U1.2 and U3.1 build
  on, after those units are written.
- Two consecutive units cannot be measured, which would mean the measurement
  story is wrong rather than the units.

---

## 6. What this train deliberately does not do

| Not done | Why |
|---|---|
| Anthropic Messages migration | Investigation, not a commitment — [PRD-v6](./PRD-v6.md) §7.2. |
| A new major line / new PRD | Rule: a minor is not a new PRD unless behavior identity shifts. Nothing here shifts identity. |
| Agent teams / mailboxes / task boards | Serves a browser UI this product does not have; deferred by [NON_GOALS](./NON_GOALS.md). |
| Sandbox modes + escalation | Correct vocabulary, no substrate; revisit with a sandbox story. |
| dsh's request-series bookkeeping | Precise instrument for a model this product does not (yet) use. |
| Everything-is-a-plugin | Architectural identity of another product; not portable to a Rust pager. |
| New TUI surfaces | This train is depth, not surface. |
