# Ultragoal chain — overnight continuity

**Purpose:** When one wave plan completes, the next starts without re-deriving the final goal.  
**Major lines:** [versions/README.md](./versions/README.md)  
**Owner-bar DoD:** [OWNER_BAR_ACCEPTANCE.md](./OWNER_BAR_ACCEPTANCE.md) · [OWNER_BAR_P0_LEDGER.md](./OWNER_BAR_P0_LEDGER.md)

---

## Active product chain

**Active:** plan id **`vision-complete-5x`** (close VISION north star inside **`5.x.y`** after owner-bar).

| Order | Plan / stage | Role | Prompt / board |
|-------|--------------|------|----------------|
| **1** | **`vision-complete-5x`** | **Active** — Deep Code + Reasonix + Grok feel under `5.x` | [VISION_COMPLETE_5X_GOALS.md](./VISION_COMPLETE_5X_GOALS.md) · [WAVE_5x_VISION_PR_DAG.md](./WAVE_5x_VISION_PR_DAG.md) · [VISION.md](./VISION.md) |
| — | **`owner-bar-5x`** | **Complete** owner-bar product cut **`5.0.0`** | [CUT_5_0_0_2026-08-07.md](./evidence/CUT_5_0_0_2026-08-07.md) · [OWNER_BAR_5X_GOALS.md](./OWNER_BAR_5X_GOALS.md) · [PRD-v5.md](./PRD-v5.md) |

```text
G001 TruthHarness (RED gate)
  → G002 PathA-R0-Rig
  → G003 Mint → G004 SnippetLive+liveness → G005 WriteBash
  ∥ G006 Perms ∥ G007 Repair ∥ G009 Routing
  → G008 Prefix/Skills/Resume
  → G010 L3 under hearts
  → G011 Install
  → G012 Freeze + dual review + tag v5.0.0
```

Owner-bar train is frozen complete. **Active** plan-id is **`vision-complete-5x`** only; do not invent a third product plan-id mid-train.

### Completed / superseded trains (not owner-bar green)

| Plan id | Role | Owner-bar? |
|---------|------|------------|
| **`fleet-4x`** | 4.0.0 L3 productization *attempt* — tagged `v4.0.0`; install/UX `4.0.1`/`4.0.2` | **NO** |
| **`heart-3x`** | 3.0.0 heart fusion *attempt* — tagged `v3.0.0` | **NO** |
| **`ship-3.0.0`** | 3.0.0 ship closeout (tag/release/npm verify) | closeout only |
| **`l3-prep-lane-b`** | Parallel L3 prep during 3.0 | prep only |
| **`grokbase-2x`** | 2.x Grok base + DeepSeek shell | shell only |

**Do not resume `heart-3x` / `fleet-4x` as product SSOT.** Use `owner-bar-5x` only.

Cut evidence (historical tags): [CUT_4_0_0_2026-08-07.md](./evidence/CUT_4_0_0_2026-08-07.md) · [CUT_3_0_0_2026-08-07.md](./evidence/CUT_3_0_0_2026-08-07.md)

---

## Historical scaffold chain (archive)

| Plan id | Wave |
|---------|------|
| `dogfood-0x` / `native-0x` / `throughput-0x` / `rc-1.0.0` | A–D scaffold |

---

## Operator loop (until 12/12 owner-bar-5x)

```bash
git fetch origin && git checkout main && git pull origin main
./scripts/test-owner-bar.sh || true
./scripts/check-path-a-linkage.sh || true
omc ultragoal status --plan-id owner-bar-5x
omc ultragoal complete-goals --plan-id owner-bar-5x
```

Cold start paste: [ULTRAGOAL_PROMPT_COLD_START_5.0.md](./ULTRAGOAL_PROMPT_COLD_START_5.0.md)

## Continuity rules

1. Final goal: [PRD-v5.md](./PRD-v5.md) — Path A only for hearts/L3.  
2. PR planning: [ULTRAGOAL_PR_PLANNING.md](./ULTRAGOAL_PR_PLANNING.md).  
3. Merge commits when repo forbids squash.  
4. Full SemVer only; dual CLI names.  
5. Dual adversarial review required on G012 before tag.  
6. GitHub public text English.  
7. Keep product default non-YOLO (`yolo = false`) while implementing hearts.  
