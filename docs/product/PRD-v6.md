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
