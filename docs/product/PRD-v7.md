# PRD-v7 — DeepSeek-native depth

**Line:** `7.x` · **First cut:** `7.0.0` · **Status:** proposed
**Owner-readable summary:** (filled at the cut)
**Prior line:** [PRD-v6.md](./PRD-v6.md)

---

## 1. Why this line exists

`5.x` made the product real. `6.x` made its base current. Neither line asked a
narrower question: **does this product yet exploit what the DeepSeek models and
the DeepSeek platform actually offer?**

The product's own philosophy answers that it should — the identity is
"DeepSeek-native harness", not "generic agent on DeepSeek" — and
[HARNESS_PHILOSOPHY](../architecture/HARNESS_PHILOSOPHY.md) §2 says the
quality lever is *model × harness*. Two rounds of reading other harnesses have
now happened. The first ([upstream-gap-sweep-2026-09-25](../research/upstream-gap-sweep-2026-09-25.md))
read Deep Code and Reasonix and produced small, sharp fixes. The second
([dsh-deepseek-harness](../research/dsh-deepseek-harness.md)) read **DeepSeek's
own official harness** and found something different in kind: a coherent body
of work on **keeping a cached prefix alive while the session changes**, plus a
habit of enforcing context contracts with runtime invariants rather than tests
alone.

That is the thesis of this line. Not new surface, not a new architecture —
**depth on the axis the product already claims to own.**

---

## 2. Architecture — the layers, restated for this line

| Layer | Owner | What `7.x` does to it |
|---|---|---|
| **L1** DeepSeek-native contracts | Deep Code (+ dsh evidence) | **Deepened.** The prompt moves from "a prefix section" to "a history node with an in-history update path"; policy state stops being invisible to the model; tool-shape discipline (catalog churn 0, recovery sentences) becomes explicit. |
| **L2** Cost & session economics | Reasonix (+ dsh evidence) | **Deepened, this line's center.** Cache-miss *attribution*, the append-don't-rewrite law, spill, and the summarize-from-warm-prefix rule. Measured, not asserted. |
| **L3** Execution throughput | Grok Build | **Almost untouched.** The jobs-controller idea is scoped small; everything else in this line is L1/L2. |

**L3 never overrides L1/L2** ([HARNESS_PHILOSOPHY](../architecture/HARNESS_PHILOSOPHY.md) §3).
This line is unusually clean about that: it takes no throughput feature at all
in its first cut.

**Source priority note.** dsh is **not** a new L1 owner. Deep Code remains the
L1 primary. dsh enters as *evidence* — the strongest available evidence for
DeepSeek-native behavior, because it is DeepSeek's own harness — and its items
enter `docs/specs/` only after being re-checked and, where the wire is
involved, measured. [SOURCES.md](./SOURCES.md) gains a row; it does not change
the layer map.

---

## 3. What `7.0.0` claims

### Claimed

| # | Claim | Evidence |
|---|---|---|
| C1 | A prompt or tool-set change no longer invalidates the whole cached prefix | Prefix-reuse evidence on a live route, before/after, with the epoch and cache fields recorded |
| C2 | Cache misses are *attributable*: the product reports what moved, not only that something moved | `prefix_epoch` plus a structured reason on every epoch change |
| C3 | The context contract is enforced by the process, not only by tests | A runtime check that the request equals the log-derived projection, failing loudly |
| C4 | Large tool results stop silently consuming context | Spill with a file locator, fail-open, measured on a real long session |
| C5 | The line stays honest about path and route: what was measured, on which route, and what was not | Honesty table below, filled from merged results |

### Shipped — honesty table

Filled at the cut; each row states the measured result rather than the intent.
A row that cannot be evidenced stays **not claimed**.

| Claim | Status | Evidence |
|---|---|---|
| C1 | | |
| C2 | | |
| C3 | | |
| C4 | | |
| C5 | | |

---

## 4. Scope of the line

**In scope**

- The cache-preserving context lifecycle (Axis 1 of the dsh sweep), starting
  with the items that need no wire change.
- Cache-miss attribution and a cumulative cache surface — the two "take (next)"
  rows the first sweep already identified but did not build.
- Runtime invariants for the context contract.
- Spill for tool results.
- Tool-result and policy-state context shapes that do not break the prefix.
- A measurement harness so cache claims have numbers behind them.

**Out of scope (explicitly)**

- **Changing the model transport** (Anthropic Messages vs Chat Completions).
  This is an investigation item with its own PRD section and its own evidence
  requirement; nothing in `7.0.0` may depend on its outcome, and no spec may
  specify `systemPromptUpdate`/`toolUpdate` until it lands. See §6.
- New product identity, new pillars, new major surfaces.
- Multi-agent coordination layers (teams, mailboxes, task boards). Deferred by
  [NON_GOALS.md](./NON_GOALS.md), and dsh's version of it exists to serve a
  browser UI this product does not have.
- Sandbox modes and escalation vocabulary. Correct idea, no substrate yet.
- Re-cutting `5.x` / `6.x`; those are shipped or in flight.
- Closing the 6.x base sync. That train owns the vendor tree; this line must
  rebase its vendor-touching work on whatever `6.x` lands.

---

## 5. Risks and how they are handled

| Risk | Handling |
|---|---|
| **The base moves under this line.** `7.x` is planned while a `1.0.0 → 1.0.41` vendor port is in flight. | Sequencing rule: no `7.x` unit that touches `third_party/grok-build/` starts before `6.x` merges; units that live in `crates/dsb-*` or docs may proceed in parallel. The first cut's Path A work is explicitly gated on the base landing. |
| **The in-history prompt path is adopted on faith.** | It is not adopted until measured on a live route (§6 for the wire, plus a prefix-reuse test). Until then, spec 10 keeps the epoch model and the in-history path stays a design note. |
| **"dsh has it" becomes the reason.** | Every item in this line traces to a **dsb** failure mode as well — a cost that is currently paid, or a class of bug that currently can ship. [HARNESS_PHILOSOPHY](../architecture/HARNESS_PHILOSOPHY.md) §12 question 3 (cache impact) must be answered in each PR. |
| **Cache work is claimed green on intent.** | The measurement harness is a unit of work, not a nice-to-have, and the honesty table above stays unfilled until numbers exist. This mirrors the `6.x` lesson that gates must be recorded honestly. |
| **Premature abstraction.** dsh's request-series machinery is powerful and dsh-shaped. | Adopt Axis 1 items in cost order — cheapest and most independent first — and re-evaluate series bookkeeping only if the request model actually changes. |

---

## 6. The wire question (investigation, PRD-tracked)

dsh speaks the **Anthropic Messages subset** of the DeepSeek API; this product
speaks **Chat Completions** per [ADR 0005](../adr/0005-deepseek-provider-contract.md).
The two paths differ in ways that matter to this line's claims — cache
accounting fields, effort levels, reasoning replay, image handling, and
whether a prompt or tool update can be expressed mid-history at all.

**This section is a placeholder for an ADR-amending investigation, not a
decision.** Its required deliverable is evidence, not prose:

1. Measure both paths on this product's own routes: what each returns for
   cache accounting, effort, thinking, and images, on the same prompt.
2. State what the in-history prompt/tool strategy would cost or gain on each.
3. Only then: amend ADR 0005, or record why Chat Completions stays.

Until that lands, `7.0.0` ships the wire-independent half of Axis 1 and says so.

---

## 7. Exit criteria

1. The wire-independent cache-preserving lifecycle is merged and **measured**
   on a live route, with before/after evidence.
2. Cache-miss attribution and the cumulative cache surface are visible to the
   owner in a real session.
3. The context-contract runtime invariant is merged, with a test that shows it
   failing when the contract is broken.
4. Spill is merged and demonstrated on a session that previously blew up the
   context.
5. The wire investigation is complete as evidence, with ADR 0005 either amended
   or explicitly re-affirmed.
6. [dsh-deepseek-harness.md](../research/dsh-deepseek-harness.md) rows this
   line did **not** take are recorded with reasons — the ledger discipline the
   `6.x` sync established, applied to harness ideas rather than upstream code.
7. Tag `v7.0.0` published to npm with a verified global install; GitHub release
   notes carrying the owner-readable summary.
