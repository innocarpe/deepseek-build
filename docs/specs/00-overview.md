# Specs overview

Behavioral contracts for what DeepSeek Build **must** do when implemented.  
Empty or stub specs mean “not specified yet” — implementers must not invent silently; open a draft spec first.

**Design spine:** [HARNESS_PHILOSOPHY.md](../architecture/HARNESS_PHILOSOPHY.md)  
Every ready-for-impl spec must cite which philosophy sections it implements.

## Planned spec index

| ID | Spec | Philosophy owner | Status |
|----|------|------------------|--------|
| 00 | This overview | — | Draft |
| 10 | Cache contract (stable prefix + session replay) | L1/L2 Deep Code B + Reasonix | TODO |
| 15 | Tool-call repair | L2 Reasonix | TODO |
| 20 | Model routing (Flash / Pro / presets) | L2 Reasonix + Deep Code UX | TODO |
| 30 | Thinking & reasoning effort | L1 Deep Code + API | TODO |
| 40 | Core tools surface (small set) | L1 Deep Code + L3 speed | TODO |
| 45 | Snippet edit contract | L1 Deep Code A | TODO |
| 50 | Parallelism & background tasks | L3 Grok | TODO |
| 60 | Subagents (+ worker cache law) | L3 Grok under L2 | TODO |
| 70 | Skills as structured on-demand context | L1 Deep Code C | TODO |
| 80 | MCP | L1 Deep Code surface | TODO |
| 90 | Side-effect permissions | L1 Deep Code D | TODO |
| 100 | Sessions (`/new`, `/resume`, `/fork`) | L1 Deep Code surface | TODO |
| 110 | Plan mode (light) | L1 Deep Code | TODO |
| 120 | Project config (`.deepseek-build/`) | All | TODO |

## Spec quality bar

A ready-to-implement spec answers:

1. User-visible behavior  
2. Failure modes  
3. Non-goals for that feature  
4. How we will know it works (manual or automated checks)  
5. Which **HARNESS_PHILOSOPHY** sections it implements  
6. Which source tool it learns from (and what it deliberately drops)  
7. **Cache-impact** class for runtime changes that follow  

## MVP cut (aligned with PRD v1 / milestones)

| Milestone | Specs (order matters) |
|-----------|------------------------|
| M1 | **10 → 15 → 20 → 30** (+ provider slice of 40) |
| M2 | **45 → 40 → 50** (snippet edit before generic “edit”) |
| M3 | 70, 90, 120 |
| M4 | 60 (must include worker cache law) |
| M5 | 80, 100, 110 |

**M2 note:** Do not implement free-form whole-file `old_string` edit as the primary path if `45` is not ready; that would violate Deep Code pillar A.

See [../product/PRD-v1.md](../product/PRD-v1.md) and [../product/MILESTONES.md](../product/MILESTONES.md).
