# Ultragoal chain — overnight continuity

**Purpose:** When one wave plan completes, the next starts without re-deriving the final goal.  
**Major lines:** [versions/README.md](./versions/README.md) · [PRD-v3.md](./PRD-v3.md)  
**Scaffold history board:** [MASTER_PLAN.md](./MASTER_PLAN.md)

---

## Active product chain (from 2026-08-07)

**`heart-3x` complete** — tag **`v3.0.0`**. Next product plate: **`fleet-4x`** (after WAVE_4x ready).

| Order | Plan / stage | Role | Prompt / board |
|-------|--------------|------|----------------|
| **1** | **`heart-3x`** | **Complete** G001→G008 → tag **`v3.0.0`** | [ULTRAGOAL_PROMPT_COLD_START_3.0.md](./ULTRAGOAL_PROMPT_COLD_START_3.0.md) · [HEART_3X_GOALS.md](./HEART_3X_GOALS.md) · cut [evidence/CUT_3_0_0_2026-08-07.md](./evidence/CUT_3_0_0_2026-08-07.md) |

```text
G001 PrepOnMain (base_url + pre-3x harness)
  → G002 PlanOnMain (this chain + DAG + cold-start)
  → G003 SpecMap
  → G004–G005 L1 snippet + permissions (alpha)
  → G006–G007 L2 prefix + repair/Flash-Pro (beta)
  → G008 Cut 3.0.0 (tag v3.0.0 only)
```

**Do not** restart A–D or `grokbase-2x` as product SSOT for new work.  
**Do not** invent a second product plan-id mid-train; extend [HEART_3X_GOALS.md](./HEART_3X_GOALS.md) only via docs PR if the board must change.  

### Parallel prep ultragoal (not fleet-4x)

While `heart-3x` runs, **Lane B** may use plan id **`l3-prep-lane-b`** in a **separate worktree**:

- Board: [LANE_B_L3_PREP_GOALS.md](./LANE_B_L3_PREP_GOALS.md)  
- Cold start: [ULTRAGOAL_PROMPT_LANE_B_L3.md](./ULTRAGOAL_PROMPT_LANE_B_L3.md)  
- Ops: [PARALLEL_3X_4X_PLAN.md](./PARALLEL_3X_4X_PLAN.md)  
- DAG draft: [WAVE_4x_PR_DAG.md](./WAVE_4x_PR_DAG.md)  
- Future cut train: [FLEET_4X_GOALS.md](./FLEET_4X_GOALS.md)  

**Do not** `omc ultragoal create-goals --plan-id fleet-4x` until **`v3.0.0`** and WAVE_4x is ready-for-impl.

### Next product chain (after 3.0.0)

| Order | Plan / stage | Role | Prompt / board |
|-------|--------------|------|----------------|
| **2** | **`fleet-4x`** | L3 productization → tag **`4.0.0`** | [ULTRAGOAL_PROMPT_COLD_START_4.0.md](./ULTRAGOAL_PROMPT_COLD_START_4.0.md) · [FLEET_4X_GOALS.md](./FLEET_4X_GOALS.md) · [WAVE_4x_PR_DAG.md](./WAVE_4x_PR_DAG.md) · [PRD-v4.md](./PRD-v4.md) · [PARALLEL_3X_4X_PLAN.md](./PARALLEL_3X_4X_PLAN.md) |

---

## Completed product chain (archive)

| Plan id | Role | Prompt / board |
|---------|------|----------------|
| **`grokbase-2x`** | 2.x Grok base + DeepSeek shell → **2.0.0+** | [ULTRAGOAL_PROMPT_COLD_START_2.0.md](./ULTRAGOAL_PROMPT_COLD_START_2.0.md) · [GROKBASE_2X_GOALS.md](./GROKBASE_2X_GOALS.md) · [WAVE_2x_PR_DAG.md](./WAVE_2x_PR_DAG.md) · [REPLAN_2.0.md](./REPLAN_2.0.md) |

---

## Historical scaffold chain (complete — archive)

| Order | Plan id | Wave | Prompt |
|-------|---------|------|--------|
| H1 | `dogfood-0x` | A | [ULTRAGOAL_PROMPT_COLD_START_0x.md](./ULTRAGOAL_PROMPT_COLD_START_0x.md) |
| H2 | `native-0x` | B | [ULTRAGOAL_PROMPT_COLD_START_NATIVE.md](./ULTRAGOAL_PROMPT_COLD_START_NATIVE.md) |
| H3 | `throughput-0x` | C | [ULTRAGOAL_PROMPT_COLD_START_THROUGHPUT.md](./ULTRAGOAL_PROMPT_COLD_START_THROUGHPUT.md) |
| H4 | `rc-1.0.0` | D | [ULTRAGOAL_PROMPT_COLD_START_RC.md](./ULTRAGOAL_PROMPT_COLD_START_RC.md) |

---

## Operator loop (product — run until 8/8)

```bash
# Always
git fetch origin && git checkout main && git pull origin main

# Everyday regression (no vendor-full disk bomb)
./scripts/test-pre3x-baseline.sh --live

# Single product plan
omc ultragoal status --plan-id heart-3x   # create if missing: HEART_3X_GOALS.md
omc ultragoal complete-goals --plan-id heart-3x

# Active story only → PR units from WAVE_3x → merge → checkpoint complete → complete-goals again
# Stop only when status shows 8/8 complete (or blocked with evidence)
```

## Continuity rules

1. Final product goal text for this train is always [PRD-v3.md](./PRD-v3.md) §3 / §6 — do not renegotiate overnight.  
2. PR planning mandatory: [ULTRAGOAL_PR_PLANNING.md](./ULTRAGOAL_PR_PLANNING.md).  
3. Child runtime = parent (Grok → grok only unless user explicitly crosses).  
4. Full SemVer only; dual CLI names always.
