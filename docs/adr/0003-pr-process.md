# ADR 0003 — Pull request process

- **Status:** Accepted
- **Date:** 2026-08-06

## Context

Work after the foundation should land in reviewable units. Without explicit conventions, direct pushes to `main` and mixed mega-diffs will erode history quality and agent consistency.

## Decision

1. **All meaningful changes go through pull requests** into `main`.  
2. **Conventional Commits** titles; types align with kind labels.  
3. **Exactly one kind label** on non-draft PRs (CI enforced).  
4. **Default merge method: squash and merge**; PR title becomes the `main` commit subject.  
5. Normative docs live under `docs/contributing/`.  
6. Soft size guidance (prefer S/M); optional `size/*` labels.

## Consequences

- Agents and humans share one process (`AGENTS.md` + contributing guides).  
- CI gains `pr-title` and `pr-kind-label` jobs.  
- Branch protection / required reviews may be tightened later without changing the human contract.
