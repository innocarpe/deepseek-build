# ADR 0003 — Pull request process

- **Status:** Accepted (depth-amended 2026-08-06)
- **Date:** 2026-08-06

## Context

DeepSeek Build will accumulate specs, provider code, tools, and orchestration. Contributors (including coding agents) need a single process that:

- Produces reviewable history  
- Forces kind/intent to be explicit  
- Fits a docs-first, milestone-driven roadmap (M0–M6)  
- Does not require a full-time review staff on day one  

A first process PR can fail by shipping **only gates and boilerplate** without teaching what a good unit of work is for this product. The process docs must carry project-specific substance (cache impact, spec-before-feat, source priorities).

## Decision

1. **All meaningful changes** land via pull request to `main`.  
2. **Conventional Commits** PR titles; types include first-class `spec`.  
3. **Exactly one kind label** on non-draft PRs (CI).  
4. **Squash-only** merge; PR title becomes `main` subject.  
5. Normative guides under `docs/contributing/` (PR, commits, branches, examples, review checklist).  
6. Soft size guidance + optional `size/*` labels.  
7. **Spec-before-large-feat** expectation for agent behavior.  
8. **Cache-impact** disclosure on prompt/tool/schema-related PRs.  
9. Solo self-merge allowed when checklist quality bar is met (not checkbox theater).

## Alternatives considered

| Alternative | Why rejected (for now) |
|-------------|-------------------------|
| GitFlow (`develop` + release branches) | Overhead for solo/docs-first; revisit at multi-release stage |
| Merge commits only | Noisy history; harder changelog-from-log |
| Labels optional | Agents “finish” with unlabeled PRs; broken skimability |
| Mandatory issues for every PR | Friction; specs can be the artifact |
| Hard LOC blocker in CI | Punishes legitimate spec prose; use soft size + review |
| CLA / DCO bot day one | Apache-2.0 + CONTRIBUTING license clause sufficient early |
| Require 1–2 human reviewers | No second human; fake gates are worse than honest self-merge |

## Consequences

### Positive

- `main` reads as intentional product history  
- CI encodes a minimum bar (title + kind)  
- Agents have a written contract (`AGENTS.md` + contributing guides)  
- Spec/feat split matches how DeepSeek-native behavior must be designed (cache, routing)

### Negative / costs

- More PR overhead than direct push  
- Must maintain docs when process evolves (via `docs` PRs + ADR amendments)  
- Self-merge can rubber-stamp weak bodies — mitigated by explicit quality bar and examples  

### Follow-ups

- Tighten branch protection (required checks) when comfortable  
- Optional: PR size bot or dangerfile later — not required for M1  

## References

- [docs/contributing/pull-requests.md](../contributing/pull-requests.md)  
- [docs/contributing/examples.md](../contributing/examples.md)  
- [docs/contributing/review-checklist.md](../contributing/review-checklist.md)  
