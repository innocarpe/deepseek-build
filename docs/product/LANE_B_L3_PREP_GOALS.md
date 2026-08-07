# Ultragoal board — **`l3-prep-lane-b`** (parallel prep, not 4.0.0 cut)

**Plan id:** `l3-prep-lane-b`  
**Purpose:** Finish **all parallel-safe 4.0 prep** while **heart-3x** ships 3.0.0.  
**Does NOT** tag 4.0.0 or change product defaults.  
**Ops:** [PARALLEL_3X_4X_PLAN.md](./PARALLEL_3X_4X_PLAN.md) Lane B  
**Worktree:** `../deepseek-build-l3-prep` (never stash heart WIP)

After this plan is **complete**, remaining 4.0 work is:

1. Live re-smoke when API credentials exist (optional follow-up PR)  
2. Wait for **`v3.0.0`**  
3. Then **`fleet-4x`** ([FLEET_4X_GOALS.md](./FLEET_4X_GOALS.md))

---

## Stories

| ID | Title | Done when |
|----|-------|-----------|
| **B001** | SmokeScript | `scripts/test-l3-smoke.sh` on main |
| **B002** | UserGuides | user-guide 11–14 + README links |
| **B003** | GapInventory | `docs/research/l3-productization-gap.md` with code pointers |
| **B004** | EvidenceNote | `docs/product/evidence/L3_SMOKE_*.md` |
| **B005** | ProductHonesty | KNOWN_LIMITS + chain/PARALLEL checklist mark Lane B closed |
| **B006** | OfflineSmokeGreen | `./scripts/test-l3-smoke.sh --offline-only` exit 0 |
| **B007** | LiveSmokeOrBlocked | Live suite green **or** explicit BLOCKED note (no credentials) |
| **B008** | PrepBoardClosed | This file + ledger 8/8; no open Lane B todos |

---

## Create ledger

```bash
omc ultragoal create-goals --plan-id l3-prep-lane-b \
  --brief "Lane B L3 prep while heart-3x runs. Worktree only. No product defaults. No fleet-4x until v3.0.0." \
  --goal "B001 SmokeScript::test-l3-smoke.sh on main" \
  --goal "B002 UserGuides::user-guide 11-14 L3 docs" \
  --goal "B003 GapInventory::code pointers for L3 surfaces" \
  --goal "B004 EvidenceNote::L3_SMOKE evidence file" \
  --goal "B005 ProductHonesty::KNOWN_LIMITS + PARALLEL checklist" \
  --goal "B006 OfflineSmokeGreen::test-l3-smoke --offline-only green" \
  --goal "B007 LiveSmokeOrBlocked::live green or credentials blocked" \
  --goal "B008 PrepBoardClosed::lane B 8/8 closed"
```

```bash
omc ultragoal complete-goals --plan-id l3-prep-lane-b
```

Never `--force` wipe progress.
