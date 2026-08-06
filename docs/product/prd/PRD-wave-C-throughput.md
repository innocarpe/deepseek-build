# PRD — Wave C: Grok-class throughput

| Field | Value |
|-------|--------|
| SemVer band | **`0.12.0` – `0.14.0`** |
| Plan id | `throughput-0x` |
| Status | Planned (after Wave B) |
| Depends on | Wave B; **G4** before parallel runtime; **G5** before subagents |

## Problem

Even a native single agent leaves wall-clock on the table: serial tools, no background shell collect, no explore/implement fan-out. Grok Build users will not switch without throughput.

## Goal

**Grok-class progress rate** under L1/L2 constraints: parallel tools, bg shell, subagents, optional worktrees, **worker cache law** enforced.

## Non-goals

- Breaking snippet/permission invariants “for speed”  
- Unique cold prefixes per worker  
- YOLO shell  

## User stories

1. Independent tool calls in one turn run concurrently.  
2. Long shell jobs run in background with collect-by-id.  
3. Explore subagent (read-only) and implement worker can run while parent continues.  
4. Workers default Flash; Pro optional for review.  
5. Worktree isolation available for write workers.  

## Exit criteria

- [ ] Spec **50** ready-for-impl + **G4 green** + parallel runtime shipped  
- [ ] Spec **60** ready-for-impl + **G5 green** + subagents shipped  
- [ ] Worker cache law documented and tested (shared stable template)  
- [ ] Ultragoal `throughput-0x` complete  
- [ ] SemVer **`0.12.0`–`0.14.0`**  

## Suggested minors

| SemVer | Theme |
|--------|--------|
| `0.12.0` | Spec 50 + G4 + parallel tools |
| `0.13.0` | Background shell + collect |
| `0.14.0` | Spec 60 + G5 + subagents/worktree |

## Failure if

- Parallelism ships without G4  
- Subagents ship without cache rules  
- Parent tools skip permissions  
