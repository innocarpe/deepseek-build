# Documentation map

Folder structure mirrors product truth. **Write the product here before the binary exists.**

## Tree

| Path | Purpose | Audience |
|------|---------|----------|
| [product/](product/) | Vision, positioning, sources, non-goals | Humans deciding what to build |
| [specs/](specs/) | Behavioral contracts (ship checklist) | Implementers + reviewers |
| [architecture/](architecture/) | System shape + repo layout | Implementers |
| [adr/](adr/) | Architecture Decision Records | Future you |
| [research/](research/) | Analysis of Grok / Reasonix / Deep Code | Context only |
| [user-guide/](user-guide/) | End-user manuals (Grok-style, later) | Users |
| [contributing/](contributing/) | PR process, examples, review checklist (normative) | Contributors |
| [maintainers/](maintainers/) | Labels and maintainer ops | Maintainers |

## Reading order (onboarding)

1. [architecture/HARNESS_PHILOSOPHY.md](architecture/HARNESS_PHILOSOPHY.md) — **design spine** (Deep Code / Reasonix / Grok layers)
2. [product/VISION.md](product/VISION.md) · [product/SOURCES.md](product/SOURCES.md) · [product/NON_GOALS.md](product/NON_GOALS.md)
3. [product/PRD-v1.md](product/PRD-v1.md) · [product/MILESTONES.md](product/MILESTONES.md)
4. [specs/00-overview.md](specs/00-overview.md)
5. [architecture/REPO_LAYOUT.md](architecture/REPO_LAYOUT.md)
6. ADRs under [adr/](adr/)
7. [contributing/](contributing/) before opening PRs

## Naming conventions

- **product/** — stable nouns (`VISION`, `SOURCES`); rarely renumbered
- **specs/** — zero-padded index + kebab feature name (`00-overview`, `10-cache-contract`)
- **adr/** — `NNNN-short-title.md`, append-only; supersede with a new ADR, do not rewrite history silently
- **research/** — tool-named notes (`grok-build.md`); not binding

## Why not top-level `prds/`?

PRD content lives under **`docs/product/`**. Keeping all truth under `docs/` avoids split brains (`prd/` vs `docs/` vs `specs/`). If a single PRD file is needed later, add `docs/product/PRD.md` rather than a new top-level tree.
