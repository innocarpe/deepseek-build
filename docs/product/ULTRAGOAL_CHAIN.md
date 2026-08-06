# Ultragoal chain — overnight continuity

**Purpose:** When one wave plan completes, the next starts without re-deriving the final goal.  
**Product SSOT:** [REPLAN_2.0.md](./REPLAN_2.0.md)  
**Scaffold history board:** [MASTER_PLAN.md](./MASTER_PLAN.md)

---

## Active product chain (from 2026-08-06)

**One ultragoal plate through product `2.0.0` — plan id `grokbase-2x` (12 stories).**  
There is **no** separate overnight plan after this for the original owner intent.

| Order | Plan / stage | Role | Prompt / board |
|-------|--------------|------|----------------|
| **1** | **`grokbase-2x`** | **Only active product plan** G001→G012 | [ULTRAGOAL_PROMPT_COLD_START_2.0.md](./ULTRAGOAL_PROMPT_COLD_START_2.0.md) · [GROKBASE_2X_GOALS.md](./GROKBASE_2X_GOALS.md) · [WAVE_2x_PR_DAG.md](./WAVE_2x_PR_DAG.md) · [REPLAN_2.0.md](./REPLAN_2.0.md) |

```text
G001 ReplanOnMain (docs #55 — often already complete)
  → G002 ADR-0008 base
  → G003 W0 spike
  → G004–G006 W1 shell alpha (integrate → entry TUI → brand/auth)
  → G007–G008 W2 DeepSeek beta (default models → edit loop)
  → G009–G010 W3 L1/L2 overlays
  → G011–G012 W4 install/docs → tag 2.0.0
```

**Do not** restart A–D plans as product SSOT. Those closed the **scaffold** line only.  
**Do not** invent a second product plan-id mid-train; extend [GROKBASE_2X_GOALS.md](./GROKBASE_2X_GOALS.md) only via docs PR if the board must change.

---

## Historical scaffold chain (complete — archive)

| Order | Plan id | Wave | Prompt |
|-------|---------|------|--------|
| H1 | `dogfood-0x` | A | [ULTRAGOAL_PROMPT_COLD_START_0x.md](./ULTRAGOAL_PROMPT_COLD_START_0x.md) |
| H2 | `native-0x` | B | [ULTRAGOAL_PROMPT_COLD_START_NATIVE.md](./ULTRAGOAL_PROMPT_COLD_START_NATIVE.md) |
| H3 | `throughput-0x` | C | [ULTRAGOAL_PROMPT_COLD_START_THROUGHPUT.md](./ULTRAGOAL_PROMPT_COLD_START_THROUGHPUT.md) |
| H4 | `rc-1.0.0` | D | [ULTRAGOAL_PROMPT_COLD_START_RC.md](./ULTRAGOAL_PROMPT_COLD_START_RC.md) |

---

## Operator loop (product — run until 12/12)

```bash
# Always
git fetch origin && git checkout main && git pull origin main

# Single product plan
omc ultragoal status --plan-id grokbase-2x   # create if missing: GROKBASE_2X_GOALS.md
omc ultragoal complete-goals --plan-id grokbase-2x

# Active story only → PR units from WAVE_2x → merge → checkpoint complete → complete-goals again
# Stop only when status shows 12/12 complete (or blocked with evidence)
```

## Continuity rules

1. Final product goal text is always [REPLAN_2.0.md](./REPLAN_2.0.md) §2 / §9 — do not renegotiate overnight.  
2. SemVer only full triples; dual CLI always; **2.0.0 only when P0 green**.  
3. Prefer small PRs; use [WAVE_2x_PR_DAG.md](./WAVE_2x_PR_DAG.md) fixed units.  
4. Parent runtime = parent family only.  
5. **Before coding:** [ULTRAGOAL_PR_PLANNING.md](./ULTRAGOAL_PR_PLANNING.md) + [stack-merge-runbook.md](../contributing/stack-merge-runbook.md).  
6. Default **serial merge**; stack only when needed; repair with `rebase --onto` after squash.  
7. Failure ladder max 3 retries → `blocked` checkpoint.  
8. npm **publish** never agent-forced complete (ADR 0007).  
9. **1.x freeze** for product features — see REPLAN §5.

## Status snapshot template (for human)

```text
Plan grokbase-2x:    n/12 complete
Next story:          G00x-…
Scaffold A–D:        complete (historical)
Cargo/npm:           1.x scaffold | 2.0.0-alpha.* | 2.0.0-beta.* | 2.0.0
```
