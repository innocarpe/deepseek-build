# Ultragoal board — **`fleet-4x`** → **`4.0.0`** (future)

**Plan id:** `fleet-4x`  
**Status:** **Do not create the omc ledger until** [PARALLEL_3X_4X_PLAN.md](./PARALLEL_3X_4X_PLAN.md) §6 gate (after **`v3.0.0`**).  
**DoD:** [PRD-v4.md](./PRD-v4.md)  
**PR units:** [WAVE_4x_PR_DAG.md](./WAVE_4x_PR_DAG.md)  
**Ops:** [PARALLEL_3X_4X_PLAN.md](./PARALLEL_3X_4X_PLAN.md)

While **heart-3x** is active, only **prep** units (WAVE_4x **4x-P0-***) may land — as ordinary docs PRs, **not** as this ultragoal train.

---

## Stories (planned G001–G008)

| ID | Title | WAVE_4x | Band | Done when |
|----|-------|---------|------|-----------|
| **G001** | PrepParallelPlan | 4x-P0-1 | docs | PARALLEL plan on main |
| **G002** | PrepWaveDraft | 4x-P0-2 | docs | WAVE_4x draft on main |
| **G003** | PrepGapInventory | 4x-P0-3 | docs | L3 gap inventory |
| **G004** | PrepDogfoodNotes | 4x-P0-4 | docs | Evidence without default changes |
| **G005** | TrainStart | 4x-P0-5 | docs | ready-for-impl + cold-start 4.0 (**after v3.0.0**) |
| **G006** | L3-Matrix | 4x-H1-* | alpha | Capability matrix + heart regression |
| **G007** | L3-Defaults | 4x-H2-* | beta | Product defaults + subagent/worktree dogfood |
| **G008** | Cut-4.0.0 | 4x-H3-* | **4.0.0** | Tag **`v4.0.0`** only |

Note: G001–G004 may be completed as **manual docs PRs** before the ledger exists; when creating the ledger after 3.0.0, checkpoint them complete with PR evidence immediately.

---

## Create ledger (only after v3.0.0)

```bash
omc ultragoal create-goals --plan-id fleet-4x \
  --brief "DeepSeek Build 4.0.0 L3 productization after heart fusion. SSOT: PRD-v4, WAVE_4x, PARALLEL_3X_4X_PLAN. Do not weaken L1/L2." \
  --goal "G001 PrepParallelPlan::PARALLEL_3X_4X_PLAN on main" \
  --goal "G002 PrepWaveDraft::WAVE_4x draft on main" \
  --goal "G003 PrepGapInventory::L3 gap inventory doc" \
  --goal "G004 PrepDogfoodNotes::L3 dogfood evidence without default changes" \
  --goal "G005 TrainStart::WAVE_4x ready-for-impl + cold-start 4.0 after v3.0.0" \
  --goal "G006 L3-Matrix::capability matrix + heart regression green" \
  --goal "G007 L3-Defaults::product defaults + subagent/worktree dogfood" \
  --goal "G008 Cut-4.0.0::docs + evidence + tag v4.0.0"
```

Never `--force` wipe an in-progress ledger.

---

## Non-goals

- Starting this plan while heart-3x incomplete  
- Dual full ultragoals  
- L1/L2 regressions for throughput
