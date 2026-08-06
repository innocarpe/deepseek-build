# ADR 0003 — Pull request process

- **Status:** Accepted (amended 2026-08-06 — harness over process-CI)
- **Date:** 2026-08-06

## Context

DeepSeek Build will accumulate specs, provider code, tools, and orchestration. Contributors (including coding agents) need a single process that:

- Produces reviewable history  
- Forces kind/intent to be explicit  
- Fits a docs-first, milestone-driven roadmap (M0–M6)  
- Does not require a full-time review staff on day one  

A first process pass can fail in two opposite ways:

1. Shipping only **gates/boilerplate** without teaching what a good unit of work is  
2. Shipping **process-police CI** (title regex, label counts, markdown path inventories) that looks “professional” but is not product development CI and was never requested  

The process must carry project-specific substance (Orca-level PR narrative, cache impact, spec-before-feat) via **docs + agent skill/harness**, not via green-check theater on an empty product surface.

## Decision

1. **All meaningful changes** land via pull request to `main`.  
2. **Conventional Commits** PR titles; types include first-class `spec`.  
3. **Exactly one kind label** on ready PRs — enforced by review/agent harness, **not** by GitHub Actions.  
4. **Squash-only** merge; PR title becomes `main` subject.  
5. Normative guides under `docs/contributing/` (including Orca-aligned body standard + examples).  
6. Agent skill: `skills/pr-authoring/SKILL.md` + standing rules in `AGENTS.md`.  
7. Soft size guidance + optional `size/*` labels.  
8. **Spec-before-large-feat** for agent behavior.  
9. **Cache-impact** disclosure on prompt/tool/schema-related PRs.  
10. Solo self-merge allowed when checklist quality bar is met (not checkbox theater).  
11. **No process-police CI.** Product CI is added only when there is something real to build/test (see `.github/workflows/README.md`).

## Alternatives considered

| Alternative | Why rejected (for now) |
|-------------|-------------------------|
| GitFlow (`develop` + release branches) | Overhead for solo/docs-first |
| Merge commits only | Noisy history |
| Labels optional | Agents finish unlabeled; broken skimability |
| **CI jobs for PR title / kind label / docs path inventory** | Not product CI; confuses “process” with “shipping quality”; user explicitly rejected |
| Hard LOC blocker in CI | Punishes legitimate spec prose |
| Require 1–2 human reviewers | Solo maintainer; fake gates worse than honest self-merge |

## Consequences

### Positive

- `main` reads as intentional product history  
- Agents load a real skill for PR authoring  
- Spec/feat split matches DeepSeek-native design (cache, routing)  
- Workflow directory stays honest: empty of fake CI until product exists  

### Negative / costs

- Process quality is not auto-enforced by a green check — depends on agents following `AGENTS.md` / skill and human review  
- Must maintain docs when process evolves  

### Follow-ups

- Product CI at M1+ (build, unit tests, prefix-hash goldens)  
- Optional later: branch protection on *product* checks only  

## References

- [docs/contributing/pull-requests.md](../contributing/pull-requests.md)  
- [docs/contributing/pr-body-standard.md](../contributing/pr-body-standard.md)  
- [skills/pr-authoring/SKILL.md](../../skills/pr-authoring/SKILL.md)  
- [.github/workflows/README.md](../../.github/workflows/README.md)  
