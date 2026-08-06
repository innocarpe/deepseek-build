# Specs overview

Behavioral contracts for what DeepSeek Build **must** do when implemented.  
Empty or stub specs mean “not specified yet” — implementers must not invent silently; open a draft spec first.

## Planned spec index

| ID | Spec | Primary source | Status |
|----|------|----------------|--------|
| 00 | This overview | — | Draft |
| 10 | Cache contract (stable prefix) | Reasonix | TODO |
| 20 | Model routing (Flash / Pro) | Reasonix + Deep Code | TODO |
| 30 | Thinking & reasoning effort | Deep Code + DeepSeek API | TODO |
| 40 | Tools (read/edit/shell/search) | Grok + Deep Code | TODO |
| 50 | Parallelism & background tasks | Grok | TODO |
| 60 | Subagents | Grok | TODO |
| 70 | Skills discovery | Deep Code | TODO |
| 80 | MCP | Deep Code | TODO |
| 90 | Permissions | Deep Code | TODO |
| 100 | Sessions (`/new`, `/resume`, `/fork`) | Deep Code | TODO |
| 110 | Plan mode | Deep Code (light) | TODO |
| 120 | Project config (`.deepseek-build/`) | All | TODO |

## Spec quality bar

A ready-to-implement spec answers:

1. User-visible behavior
2. Failure modes
3. Non-goals for that feature
4. How we will know it works (manual or automated checks)
5. Which source tool it learns from (and what it deliberately drops)

## MVP cut (aligned with PRD v1 / milestones)

| Milestone | Specs |
|-----------|--------|
| M1 | 10, 20, 30 (+ provider slice of 40) |
| M2 | 40, 50 |
| M3 | 70, 90, 120 |
| M4 | 60 |
| M5 | 80, 100, 110 |

See [../product/PRD-v1.md](../product/PRD-v1.md) and [../product/MILESTONES.md](../product/MILESTONES.md).
