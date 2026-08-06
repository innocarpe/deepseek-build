# Ultragoal chain — overnight continuity

**Purpose:** When one wave plan completes, the next starts without re-deriving the final goal.  
**Master board:** [MASTER_PLAN.md](./MASTER_PLAN.md)

## Plan order (strict)

| Order | Plan id | Wave | Prompt |
|-------|---------|------|--------|
| 1 | `dogfood-0x` | A | [ULTRAGOAL_PROMPT_COLD_START_0x.md](./ULTRAGOAL_PROMPT_COLD_START_0x.md) |
| 2 | `native-0x` | B | [ULTRAGOAL_PROMPT_COLD_START_NATIVE.md](./ULTRAGOAL_PROMPT_COLD_START_NATIVE.md) |
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
3. Do not flip G4/G5/G6 without specs.  
4. Prefer small PRs; one SemVer minor theme per merge train when possible.  
5. Parent runtime = parent family only.  
6. **Before coding any story:** complete a [PR unit plan](./ULTRAGOAL_PR_PLANNING.md) (units + sequential/parallel + stacking + atomic commits).  
7. **Atomic commits** on branches; squash to `main` per repo culture.  
8. **Stack/chain PRs** for sequential slices; parallel agents only on disjoint units.

## Status snapshot template (for human)

```text
Wave A dogfood-0x:   ?/7
Wave B native-0x:    not started | n/m
Wave C throughput:   not started | n/m
Wave D rc-1.0.0:     not started | n/m
Cargo version:       X.Y.Z
```
