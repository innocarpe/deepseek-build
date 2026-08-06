# Specs overview

Behavioral contracts for what DeepSeek Build **must** do when implemented.

**Design spine:** [HARNESS_PHILOSOPHY.md](../architecture/HARNESS_PHILOSOPHY.md)  
**Gates:** [GATES.md](../GATES.md)

## Planned spec index

| ID | Spec | Philosophy owner | Status |
|----|------|------------------|--------|
| 00 | This overview | — | ready |
| 10 | [Cache contract](./10-cache-contract.md) | L1/L2 | **ready-for-impl** |
| 15 | [Tool-call repair](./15-tool-call-repair.md) | L2 | **ready-for-impl** |
| 20 | [Model routing Flash/Pro](./20-model-routing.md) | L2 | **ready-for-impl** |
| 30 | [Thinking & effort](./30-thinking-effort.md) | L1 | **ready-for-impl** |
| 40 | Core tools surface (small set) | L1 + L3 | TODO |
| 45 | Snippet edit contract | L1 Deep Code A | TODO (blocks M2 edit) |
| 50 | Parallelism & background | L3 | TODO |
| 60 | Subagents (+ worker cache law) | L3 under L2 | TODO |
| 70 | Skills as structured context | L1 | TODO |
| 80 | MCP | L1 | TODO |
| 90 | Side-effect permissions | L1 | TODO (min before shell) |
| 100 | Sessions | L1 | TODO |
| 110 | Plan mode (light) | L1 | TODO |
| 120 | Project config | All → config owner | TODO |

## MVP cut

| Milestone | Specs |
|-----------|--------|
| M1 | **10, 15, 20, 30** (+ read-only tools later in 40 slice) |
| M2 | **45 → 90min → 40 → 50** |
| M3 | 70, 90 full, 120 |
| M4 | 60 |
| M5 | 80, 100, 110 |

## Spec quality bar

See HARNESS_PHILOSOPHY §11 and each spec’s test plan.  
Critical specs require **automated** tests in the plan.
