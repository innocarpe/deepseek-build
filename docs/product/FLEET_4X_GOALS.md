# Ultragoal board — **`fleet-4x`** → **`4.0.0`**

**Plan id:** `fleet-4x`  
**Status:** **Active** after `v3.0.0`  
**DoD:** [PRD-v4.md](./PRD-v4.md)  
**PR units:** [WAVE_4x_PR_DAG.md](./WAVE_4x_PR_DAG.md)  
**Matrix:** [L3_CAPABILITY_MATRIX.md](./L3_CAPABILITY_MATRIX.md)  
**Ops:** [PARALLEL_3X_4X_PLAN.md](./PARALLEL_3X_4X_PLAN.md)  
**Cold start:** [ULTRAGOAL_PROMPT_COLD_START_4.0.md](./ULTRAGOAL_PROMPT_COLD_START_4.0.md)

---

## Stories (G001–G008)

| ID | Title | WAVE_4x | Band | Done when |
|----|-------|---------|------|-----------|
| **G001** | PrepParallelPlan | 4x-P0-1 | docs | PARALLEL plan on main |
| **G002** | PrepWaveDraft | 4x-P0-2 | docs | WAVE_4x on main |
| **G003** | PrepGapInventory | 4x-P0-3 | docs | L3 gap inventory |
| **G004** | PrepDogfoodNotes | 4x-P0-4 | docs | L3 smoke evidence |
| **G005** | TrainStart | 4x-P0-5 | docs | ready-for-impl + cold-start 4.0 |
| **G006** | L3-Matrix | 4x-H1-* | alpha | Capability matrix + heart regression |
| **G007** | L3-Defaults | 4x-H2-* | beta | Product defaults + subagent/worktree docs + smoke |
| **G008** | Cut-4.0.0 | 4x-H3-* | **4.0.0** | Tag **`v4.0.0`** + npm |

---

## Create ledger

```bash
omc ultragoal create-goals --plan-id fleet-4x \
  --brief "DeepSeek Build 4.0.0 L3 productization after heart fusion. SSOT: PRD-v4, WAVE_4x, L3_CAPABILITY_MATRIX. Keep yolo=false / Path A hearts." \
  --goal "G001 PrepParallelPlan::PARALLEL_3X_4X_PLAN on main" \
  --goal "G002 PrepWaveDraft::WAVE_4x on main" \
  --goal "G003 PrepGapInventory::L3 gap inventory doc" \
  --goal "G004 PrepDogfoodNotes::L3 dogfood evidence without default changes" \
  --goal "G005 TrainStart::WAVE_4x ready-for-impl + cold-start 4.0 after v3.0.0" \
  --goal "G006 L3-Matrix::capability matrix + heart regression green" \
  --goal "G007 L3-Defaults::product defaults + subagent/worktree dogfood" \
  --goal "G008 Cut-4.0.0::docs + evidence + tag v4.0.0"
```

Never `--force` wipe an in-progress ledger.

```bash
omc ultragoal complete-goals --plan-id fleet-4x
```

---

## Non-goals

- Weakening L1/L2 for speed  
- Dual full ultragoals against a second heart train  
- Claiming 4.0.0 from docs alone  
