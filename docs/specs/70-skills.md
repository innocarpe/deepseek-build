# Spec 70 — Skills as structured context (minimum)

| Field | Value |
|-------|--------|
| Status | **ready-for-impl** (minimum for Wave A `0.6.0` / Wave B full) |
| Philosophy | HARNESS §4.3 Pillar C |
| Gate | **G6b** |
| Tests | Automated for index determinism + load |

## 1. Behavior (minimum)

1. Discover skills from deterministic paths (project + user + bundled).  
2. **Index** (name + one-line description, sorted) may sit in **stable prefix**.  
3. **Bodies** load on demand into **volatile** context / tool result — not full dump every turn.  
4. Changing skills index → new prefix epoch (spec 10).  
5. Skill may opt out of implicit invocation (document flag).

## 2. Non-goals (minimum)

- Full marketplace  
- Network skill install  

## 3. Test plan

| Test | Expect |
|------|--------|
| `index_sorted_stable` | same inputs → same index bytes |
| `body_not_in_stable_prefix` | body text absent from prefix builder output |

## 4. Implementation notes

Minimum surface shipped at **0.6.0**; Wave B expands product completeness.
