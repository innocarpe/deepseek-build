# PRD-v6 — base refresh and harness consolidation

**Line:** `6.x` · **First cut:** `6.0.0` · **Status:** base refresh
**Owner-readable summary:** [CHANGELIST_6_0_0.md](./CHANGELIST_6_0_0.md)
**Ledger:** [UPSTREAM_SYNC_LEDGER.md](./UPSTREAM_SYNC_LEDGER.md)
**Prior line:** [PRD-v5.md](./PRD-v5.md)

---

## 1. Why this line exists

`5.x` finished the **owner-bar**: the product is a real DeepSeek coding agent
with a full-screen TUI, its own provider, its own config surface, and its own
npm/install story. What it did not keep current was its **base**.

The product's runtime is open-source **Grok Build**, vendored under
`third_party/grok-build/` ([ADR-0008](../adr/0008-grok-build-base.md)). That
tree was pinned to Grok Build **`1.0.0`** on 2026-08-09. Upstream reached
**`1.0.41`** on 2026-09-22 — **41 releases** covering 472 changelog items,
3,587 changed files, and 21 new workspace crates.

`6.x` is the line that moves the base and keeps it movable. The product-facing
consequence is a large amount of accumulated latency work, bug fixing and TUI
polish arriving at once; the engineering consequence is that syncing is now
documented, measured and repeatable instead of rediscovered each time.

**This is not a re-imagining of the product.** L1/L2/L3 ownership, the snippet
edit contract, the permission model and the DeepSeek identity are unchanged.
The base moved; the contracts did not.

---

## 2. Architecture — the layers, restated for this line

| Layer | Owner | What `6.0.0` does to it |
|---|---|---|
| **L1** DeepSeek-native contracts | Deep Code | **Unchanged.** Snippet edit, skills-as-context, side-effect permissions stay as specified. The port was audited for anything that would weaken them. |
| **L2** Cost & session economics | Reasonix | **Strengthened.** Upstream added per-effort model identifiers, effort levels sourced from the API rather than hardcoded lists, per-model retry configuration and request-size caps — all directly useful to cache and cost behavior. |
| **L3** Execution throughput | Grok Build | **Refreshed.** This is the layer that moved: bounded subagent spawning, lifecycle ordering, background task rows, worktree status, and a large body of latency work. |

**L3 never overrides L1/L2** ([HARNESS_PHILOSOPHY](../architecture/HARNESS_PHILOSOPHY.md) §3).
Upstream changes that would have traded snippet-edit safety or prefix stability
for speed were rejected rather than absorbed; the verdicts are in the ledger.

---

## 3. What `6.0.0` claims, and what it shipped

### Claimed

| # | Claim | Evidence |
|---|---|---|
| C1 | The vendored base is Grok Build `1.0.41`, not `1.0.0` | `third_party/grok-build/SOURCE_REV`; vendor check exits 0 |
| C2 | The product's own overlay survived the move intact | status line, themes, prompt identity, DeepSeek sampling mapping, Path A helpers all present and tested |
| C3 | The owner can read, in one file, what improved and what did not | [CHANGELIST_6_0_0.md](./CHANGELIST_6_0_0.md) |
| C4 | Syncing is now infrastructure, not rediscovery | [grok-sync skill](../../skills/grok-sync/SKILL.md), [runbook](../contributing/grok-sync-runbook.md), [ledger](./UPSTREAM_SYNC_LEDGER.md), `scripts/grok-sync-inventory.sh` |
| C5 | The product gates stay green across the port | `test-owner-bar`, `check-path-a-linkage`, `test-heart-regression`, `test-grok-vendor-offline` |

### Shipped — honesty table

Each row states the measured result, taken on `main` after the port merged
(2026-09-25). A claim that could not be evidenced would stay **not claimed**;
none did.

| Claim | Status | Evidence |
|---|---|---|
| C1 | **shipped** | `SOURCE_REV = 036a5d8348cd744767cd0b08518ab17bf608fa7f`; the vendored `xai-grok-version` crate reports `1.0.41`; `build-grok-pager.sh check` exits 0 |
| C2 | **shipped** | Each overlay group is present in the tree: DeepSeek Night themes (12 files), the `x.ai/deepseek/status` extension (6), the snippet store (4), `DEEPSEEK_BUILD_VERSION` injection (6), Path A cache signal (3), spec-10 assembly (3), the product update rules (2). The agent prompt carries no vendor claim, and the product overwrites `system_prompt_label` on every DeepSeek stanza (`dsb-cli`) |
| C3 | **shipped** | [`CHANGELIST_6_0_0.md`](./CHANGELIST_6_0_0.md) — faster / fixed / new / changed / not-taken, one file |
| C4 | **shipped** | [`skills/grok-sync`](../../skills/grok-sync/SKILL.md), [`grok-sync-runbook.md`](../contributing/grok-sync-runbook.md), [`UPSTREAM_SYNC_LEDGER.md`](./UPSTREAM_SYNC_LEDGER.md), `scripts/grok-sync-inventory.sh` — all on `main`; the inventory reports the pin, upstream HEAD, and the 472-bullet cluster table in one command |
| C5 | **shipped** | Run on `main` after the port merged: `test-owner-bar` **ALL PASS (PASS=60 FAIL=0 NOT_RUN=0)** · `check-path-a-linkage` **PASS** · `test-heart-regression` **PASS** · `test-grok-vendor-offline` **ALL PASSED** · `test-product-offline` **ALL PASSED** · `check-semver` ok · CI `grok clippy` and `grok fmt` **pass** |

**Not claimed.** The port did not exercise credential-gated live behaviour: the
L3 checks (`L3.1`–`L3.3`, `L3.5`) and the Path A e2e are `SKIP` without an API
key. The merge left them unverified, and this table does not imply otherwise.

---

## 4. Scope of the line

**In scope**

- Moving the vendored base forward and re-deriving the overlay.
- Making sync repeatable: skill, runbook, ledger, inventory tooling.
- The upstream work that is worth having in a DeepSeek product (see the changelist).
- Recording, per item, what was taken, held and rejected — and why.

**Out of scope (explicitly)**

- New product identity, new pillars, new L1/L2 contracts.
- Adopting upstream's xAI-hosted surface: telemetry, xAI login/OIDC/team
  policy, billing, self-update channels, voice, video, desktop/Computer-Hub.
  These are `N/A` for this product, not "later".
- Re-cutting `5.0.0`–`5.6.0`; those are shipped and published.
- Chasing upstream's release cadence. Upstream ships near-daily; this product
  syncs when the accumulated value justifies a base move.

---

## 5. Risks and how they are handled

| Risk | Handling |
|---|---|
| The overlay is silently weakened by the move | The overlay is enumerated in the ledger and the vendor doc; the gates exercise it; the port PR states each carried group. |
| A breaking upstream change contradicts a dsb spec | Every breaking change gets an explicit verdict; where it bites (the `capability_mode` removal), the affected docs were updated in the same change. |
| The next sync is rediscovery again | The skill, runbook, ledger and inventory script are the deliverable for exactly this. A future session runs one command and reads one ledger. |
| The port is claimed green on gates that did not run | The PR's Testing section records commands and real outcomes; the ledger's gate row is filled from the merged result. |

---

## 6. Exit criteria

1. Vendored tree at the new base, overlay re-derived, vendor check green.
2. Product gates green, recorded honestly.
3. [CHANGELIST_6_0_0.md](./CHANGELIST_6_0_0.md) merged — one file, owner-readable.
4. Sync infrastructure merged and usable by a session with no prior context.
5. Tag `v6.0.0` published to npm with a verified global install; GitHub release
   notes carrying the changelist digest.

---

## 7. Line continuation — `6.1.0` DeepSeek-native depth

**Status:** proposed. Board: [DEEPSEEK_NATIVE_DEPTH_6X_GOALS.md](./DEEPSEEK_NATIVE_DEPTH_6X_GOALS.md).
Evidence: [research/dsh-deepseek-harness.md](../research/dsh-deepseek-harness.md).

`6.0.0` moved the base. The next question is narrower and belongs to the same
line: **does the product actually exploit what this base now makes possible?**

Reading DeepSeek's own official harness (`dsh`) answered it. dsh carries a
coherent body of work on **keeping a cached prefix alive while the session
changes** — the prompt, the tool set, and policy state all append after cached
history instead of rewriting the head — plus a habit of enforcing context
contracts with runtime invariants rather than tests alone. That is the natural
sequel to §2's L2 work: **§2 strengthens the substrate; §7 spends it.**

### Why `6.1.0` and not a new major

Per [versions/README.md](./versions/README.md) §Rules 1, a minor is a changelog
and release notes *unless behavior identity shifts*. This work adds no new
surface, no new pillar, and no new identity — it makes an existing claim
("DeepSeek-native harness") true in places it currently is not. That is a
minor by the rule, and the dependency direction agrees: §2's per-effort model
identifiers, API-sourced effort levels, per-model retry configuration, and
request-size caps are precisely the substrate the depth units build on.

**Two conditions move this off the 6.x line:**

1. If the wire investigation (§7.2 below) concludes the product should move to
   the **Anthropic Messages** transport, that is an identity-relevant change
   and takes its own major.
2. If `6.x` minor numbers are already spent by base-port follow-up work when
   this train starts, the depth work moves to the next free line.

### 7.1 Scope

**In scope (wire-independent, ships as `6.1.0` and later minors):**

- The cache-preserving context lifecycle, in cost order — policy state as
  runtime context, the prompt/tool change path, retry reuses the assembly,
  centered section ordering.
- Cache-miss **attribution** and a cumulative cache surface, so a miss says
  *what moved*, not only that something moved.
- A cache measurement harness — first, so no later unit can claim a win
  without numbers.
- A runtime invariant that the request equals the log-derived projection.
- Spill for oversized tool results.

**Out of scope:** multi-agent coordination layers, sandbox modes and
escalation, dsh's request-series bookkeeping, everything-is-a-plugin, and any
new TUI surface. Reasons are recorded per item in the research doc and the
board's §5.

### 7.2 The wire question (evidence, not a decision)

dsh speaks the **Anthropic Messages subset** of the DeepSeek API; this product
speaks **Chat Completions** per [ADR 0005](../adr/0005-deepseek-provider-contract.md).
The paths differ in cache accounting fields, effort levels, reasoning replay,
image handling, and whether a prompt or tool update can be expressed
mid-history at all.

**Nothing in §7.1 may depend on this outcome, and no spec may specify
`systemPromptUpdate`/`toolUpdate` until it lands.** The deliverable is
evidence on this product's own routes, followed by an ADR 0005 amendment or an
explicit re-affirmation.

### 7.3 Exit criteria

1. The wire-independent lifecycle is merged and **measured** on a live route.
2. Cache attribution and the cumulative surface are visible to the owner.
3. The context-contract invariant is merged, with a test that fails when the
   contract is broken.
4. Spill is demonstrated on a session that previously blew up the context.
5. The wire investigation closes with ADR 0005 amended or re-affirmed.
6. Rows this line did **not** take are recorded with reasons — the ledger
   discipline [UPSTREAM_SYNC_LEDGER.md](./UPSTREAM_SYNC_LEDGER.md) established
   for vendor syncs, applied to harness ideas.
7. Tag `v6.1.0` published to npm with a verified global install.
