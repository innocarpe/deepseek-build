# ADR 0001 — Repository layout

- **Status:** Accepted
- **Date:** 2026-08-06

## Context

DeepSeek Build starts as a greenfield sibling of `grok-build`. Folder structure will harden into product structure; a messy tree becomes a messy product.

We considered:

1. Flat repo with ad-hoc markdown
2. Top-level `prds/` + `specs/` + `src/`
3. Docs-centric tree + reserved implementation roots (`crates/`, `skills/`, …)

## Decision

Use:

- **`docs/{product,specs,architecture,adr,research,user-guide}`** as the only documentation root
- **`crates/`** reserved for implementation packages (Grok-like)
- **`skills/`**, **`.deepseek-build/`**, **`scripts/`**, **`third_party/`** as first-class product/runtime seams

Do **not** create top-level `prds/`. PRD material lives in `docs/product/`.

## Consequences

- Onboarding is “read `docs/README.md`”
- Implementation language can still change with a later ADR that may rename `crates/`
- Agents are instructed (via `AGENTS.md`) not to invent top-level folders without an ADR
