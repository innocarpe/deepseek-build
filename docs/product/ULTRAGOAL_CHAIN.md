# Ultragoal chain — overnight continuity

**Purpose:** When one wave plan completes, the next starts without re-deriving the final goal.  
**Major lines:** [versions/README.md](./versions/README.md) · [PRD-v4.md](./PRD-v4.md)  
**Scaffold history board:** [MASTER_PLAN.md](./MASTER_PLAN.md)

---

## Active product chain (from 2026-08-07)

**One ultragoal plate through product `4.0.0` — plan id `fleet-4x`.**

| Order | Plan / stage | Role | Prompt / board |
|-------|--------------|------|----------------|
| **1** | **`fleet-4x`** | **Active** L3 productization → tag **`4.0.0`** | [ULTRAGOAL_PROMPT_COLD_START_4.0.md](./ULTRAGOAL_PROMPT_COLD_START_4.0.md) · [FLEET_4X_GOALS.md](./FLEET_4X_GOALS.md) · [WAVE_4x_PR_DAG.md](./WAVE_4x_PR_DAG.md) · [L3_CAPABILITY_MATRIX.md](./L3_CAPABILITY_MATRIX.md) · [PRD-v4.md](./PRD-v4.md) |

```text
G001–G004 prep (Lane B during 3.0 — complete)
  → G005 TrainStart (ready-for-impl)
  → G006 L3-Matrix
  → G007 L3-Defaults
  → G008 Cut 4.0.0 (tag v4.0.0)
```

**Do not** invent a second product plan-id mid-train; extend FLEET_4X_GOALS via docs PR only.

### Completed trains

| Plan id | Role |
|---------|------|
| **`heart-3x`** | 3.0.0 heart fusion — **complete** (tag `v3.0.0`, npm 3.0.0) |
| **`l3-prep-lane-b`** | Parallel L3 prep during 3.0 — **complete** |
| **`grokbase-2x`** | 2.x Grok base + DeepSeek shell |

---

## Historical scaffold chain (archive)

| Plan id | Wave |
|---------|------|
| `dogfood-0x` / `native-0x` / `throughput-0x` / `rc-1.0.0` | A–D scaffold |

---

## Operator loop (until 8/8 fleet-4x)

```bash
git fetch origin && git checkout main && git pull origin main
./scripts/test-product-offline.sh
./scripts/test-l3-smoke.sh --offline-only
omc ultragoal status --plan-id fleet-4x
omc ultragoal complete-goals --plan-id fleet-4x
```

## Continuity rules

1. Final goal for this train: [PRD-v4.md](./PRD-v4.md) — keep Path A hearts.  
2. PR planning: [ULTRAGOAL_PR_PLANNING.md](./ULTRAGOAL_PR_PLANNING.md).  
3. Merge commits (squash disabled).  
4. Full SemVer only; dual CLI names.  
