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
| [contributing/](contributing/) | PR / commit / branch process (normative) | Contributors |
| [maintainers/](maintainers/) | Labels and maintainer ops | Maintainers |

## Reading order (onboarding)

1. [product/PRD-v1.md](product/PRD-v1.md) — full product requirements
2. [product/MILESTONES.md](product/MILESTONES.md) — M0–M6 plan
3. [product/VISION.md](product/VISION.md)
4. [product/SOURCES.md](product/SOURCES.md)
5. [product/NON_GOALS.md](product/NON_GOALS.md)
6. [architecture/REPO_LAYOUT.md](architecture/REPO_LAYOUT.md)
7. [specs/00-overview.md](specs/00-overview.md)
8. ADRs under [adr/](adr/)

## Naming conventions

- **product/** — stable nouns (`VISION`, `SOURCES`); rarely renumbered
- **specs/** — zero-padded index + kebab feature name (`00-overview`, `10-cache-contract`)
- **adr/** — `NNNN-short-title.md`, append-only; supersede with a new ADR, do not rewrite history silently
- **research/** — tool-named notes (`grok-build.md`); not binding

## Why not top-level `prds/`?

PRD content lives under **`docs/product/`**. Keeping all truth under `docs/` avoids split brains (`prd/` vs `docs/` vs `specs/`). If a single PRD file is needed later, add `docs/product/PRD.md` rather than a new top-level tree.
