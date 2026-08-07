# Ultragoal cold-start — product **`fleet-4x`** → **`4.0.0`**

**Status:** Stub for **after `v3.0.0`**.  
**Do not paste as active train while heart-3x is incomplete.**

When Phase 2 of [PARALLEL_3X_4X_PLAN.md](./PARALLEL_3X_4X_PLAN.md) starts, expand this file to full cold-start form (mirror [ULTRAGOAL_PROMPT_COLD_START_3.0.md](./ULTRAGOAL_PROMPT_COLD_START_3.0.md)) and flip [WAVE_4x_PR_DAG.md](./WAVE_4x_PR_DAG.md) to **ready-for-impl**.

---

## Preconditions (check before any fleet-4x session)

```bash
git fetch origin && git checkout main && git pull origin main
# 3.0.0 must be tagged
git tag -l 'v3.0.0' | grep -q . || { echo "BLOCKED: need v3.0.0"; exit 1; }
test -f docs/product/WAVE_4x_PR_DAG.md
test -f docs/product/PARALLEL_3X_4X_PLAN.md
rg -n 'ready-for-impl|DRAFT' docs/product/WAVE_4x_PR_DAG.md | head
./scripts/test-pre3x-baseline.sh --live
```

If WAVE_4x still says **DRAFT** only → run finalize docs PR (4x-P0-5) first.

---

## Placeholder mission (replace at finalize)

```text
# ROLE
You ship DeepSeek Build 4.0.0 (L3 productization) after hearts are green.
Plan id: fleet-4x. Child runtime = parent (grok only unless user crosses).

# FINAL GOAL
PRD-v4 P0: product defaults for parallel/bg/subagent/worktree; docs; evidence;
tag v4.0.0 only. Never weaken L1/L2 (HARNESS).

# BOARD
docs/product/FLEET_4X_GOALS.md
docs/product/WAVE_4x_PR_DAG.md
docs/product/PARALLEL_3X_4X_PLAN.md

# STOP
Do not start if v3.0.0 missing or hearts red.
```

---

## Prep-only reminder (during 3.0)

Use a **second worktree** and normal docs PRs to `main` for 4x-P0-1..4.  
Do **not** run `omc ultragoal create-goals --plan-id fleet-4x` until the gate passes.
