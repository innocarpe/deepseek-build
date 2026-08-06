# Ultragoal chain — overnight continuity

**Purpose:** When one wave plan completes, the next starts without re-deriving the final goal.  
**Master board:** [MASTER_PLAN.md](./MASTER_PLAN.md)

## Plan order (strict)

| Order | Plan id | Wave | Prompt |
|-------|---------|------|--------|
| 1 | `dogfood-0x` | A | [ULTRAGOAL_PROMPT_COLD_START_0x.md](./ULTRAGOAL_PROMPT_COLD_START_0x.md) |
| 2 | `native-0x` | B | [ULTRAGOAL_PROMPT_COLD_START_NATIVE.md](./ULTRAGOAL_PROMPT_COLD_START_NATIVE.md) **← paste this after Wave A / 0.7.0** |
| 3 | `throughput-0x` | C | [ULTRAGOAL_PROMPT_COLD_START_THROUGHPUT.md](./ULTRAGOAL_PROMPT_COLD_START_THROUGHPUT.md) |
| 4 | `rc-1.0.0` | D | [ULTRAGOAL_PROMPT_COLD_START_RC.md](./ULTRAGOAL_PROMPT_COLD_START_RC.md) |

## Operator loop

```bash
# Always
git fetch origin && git checkout main && git pull origin main

# Detect active wave
omc ultragoal status --plan-id dogfood-0x
omc ultragoal status --plan-id native-0x
omc ultragoal status --plan-id throughput-0x
omc ultragoal status --plan-id rc-1.0.0

# Work the first plan that is not fully complete
omc ultragoal complete-goals --plan-id <active>
# … implement, PR, merge, pull …
omc ultragoal checkpoint --plan-id <active> --goal-id <id> --status complete \
  --evidence "…" --claude-goal-json '…'
```

When `status` shows all complete for a plan, **do not stop**: create next plan if missing (commands in cold-start prompts), then `complete-goals` on the next plan id.

## Continuity rules

1. Final goal text is always [MASTER_PLAN.md](./MASTER_PLAN.md) §1 — do not renegotiate overnight.  
2. SemVer only full triples; dual CLI always.  
3. Subgates: G6a/b/c/d; G4 before parallel; G5 before subagents.  
4. Prefer small PRs; use [WAVE_*_PR_DAG](./WAVE_A_PR_DAG.md) fixed units.  
5. Parent runtime = parent family only.  
6. **Before coding:** [ULTRAGOAL_PR_PLANNING.md](./ULTRAGOAL_PR_PLANNING.md) + [stack-merge-runbook.md](../contributing/stack-merge-runbook.md).  
7. Default **serial merge**; stack only when needed; repair with `rebase --onto` after squash.  
8. Failure ladder max 3 retries → `blocked` checkpoint.  
9. npm **publish** never agent-forced complete (ADR 0007).

## Status snapshot template (for human)

```text
Wave A dogfood-0x:   ?/7
Wave B native-0x:    not started | n/m
Wave C throughput:   not started | n/m
Wave D rc-1.0.0:     not started | n/m
Cargo version:       X.Y.Z
```
