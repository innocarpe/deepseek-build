# Ultragoal chain — overnight continuity

**Purpose:** When one wave plan completes, the next starts without re-deriving the final goal.  
**Product SSOT:** [REPLAN_2.0.md](./REPLAN_2.0.md)  
**Scaffold history board:** [MASTER_PLAN.md](./MASTER_PLAN.md)

---

## Active product chain (from 2026-08-06)

| Order | Plan / stage | Wave | Prompt / DAG |
|-------|--------------|------|----------------|
| **0** | replan + wiring | docs | [REPLAN_2.0.md](./REPLAN_2.0.md) |
| **1** | `grokbase-2x` | W0–W4 | [ULTRAGOAL_PROMPT_COLD_START_2.0.md](./ULTRAGOAL_PROMPT_COLD_START_2.0.md) · [WAVE_2x_PR_DAG.md](./WAVE_2x_PR_DAG.md) |

```text
replan-2.0 → ADR-0008 → W0 research → W1 shell → W2 DeepSeek → W3 L1/L2 → W4 cut 2.0.0
```

**Do not** restart A–D plans as product SSOT. Those closed the **scaffold** line only.

---

## Historical scaffold chain (complete — archive)

| Order | Plan id | Wave | Prompt |
|-------|---------|------|--------|
| H1 | `dogfood-0x` | A | [ULTRAGOAL_PROMPT_COLD_START_0x.md](./ULTRAGOAL_PROMPT_COLD_START_0x.md) |
| H2 | `native-0x` | B | [ULTRAGOAL_PROMPT_COLD_START_NATIVE.md](./ULTRAGOAL_PROMPT_COLD_START_NATIVE.md) |
| H3 | `throughput-0x` | C | [ULTRAGOAL_PROMPT_COLD_START_THROUGHPUT.md](./ULTRAGOAL_PROMPT_COLD_START_THROUGHPUT.md) |
| H4 | `rc-1.0.0` | D | [ULTRAGOAL_PROMPT_COLD_START_RC.md](./ULTRAGOAL_PROMPT_COLD_START_RC.md) |

---

## Operator loop (product)

```bash
# Always
git fetch origin && git checkout main && git pull origin main

# Product plan
omc ultragoal status --plan-id grokbase-2x   # create if missing per cold-start 2.0

# Work next incomplete WAVE_2x unit only
# … implement, PR, merge, pull …
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
Product replan:      merged | open
W0 research:         ?/3
W1 shell:            not started | n/m
W2 deepseek:         not started | n/m
W3 l1/l2:            not started | n/m
W4 cut 2.0.0:        not started | n/m
Scaffold A–D:        complete (historical)
Cargo/npm:           1.x scaffold | 2.0.0-alpha.* | …
```
