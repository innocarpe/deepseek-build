# ADR 0008 — Grok Build base strategy for product 2.0.0

- **Status:** Accepted  
- **Date:** 2026-08-06  
- **Gate:** Wave 2.x W0 (`2x-W0-1`) / ultragoal `grokbase-2x` G002  
- **Normative companions:** [REPLAN_2.0.md](../product/REPLAN_2.0.md) §3–§4 · [WAVE_2x_PR_DAG.md](../product/WAVE_2x_PR_DAG.md) · [0007-npm-packaging.md](./0007-npm-packaging.md)

## Context

Owner intent for DeepSeek Build **2.0.0** is a **Grok Build–class** full-screen coding agent with **DeepSeek** as the default provider — not a polished 1.x thin clap REPL.

Open-source **Grok Build** (`xai-org/grok-build`, Apache-2.0) already provides:

- Full-screen TUI (`xai-grok-pager` / `xai-grok-pager-bin`)
- Agent runtime (`xai-grok-agent`, lifecycle, chat state)
- Tools / shell / sandbox / MCP / subagent / worktree surfaces

This repository already has:

- Published **1.x scaffold** (`@innocarpe/deepseek-build` 1.0.0 / 1.1.0) — keep installable; freeze product expansion
- Overlay crates: `dsb-provider-deepseek`, `dsb-config`, `dsb-tools`, `dsb-context`, …
- Dual CLI names `deepseek-build` + `dsb` (ADR 0006) and npm packaging (ADR 0007)

We must pick **one** integration strategy so later waves do not invent overnight layouts.

### Options considered

| ID | Strategy | Pros | Cons |
|----|----------|------|------|
| **A** | Fork: replace product tree with Grok as the main workspace; layer DeepSeek branding/auth | Cleanest “base is Grok”; single mental model | Disrupts this repo’s 1.x history/layout; high merge risk with existing crates/npm files; hard to keep scaffold packages side-by-side |
| **B** | **Git subtree (or equivalent vendor pin) of Grok under a fixed path** + workspace/bin overlay | Clear upstream pin via `SOURCE_REV`; preserves 1.x crates as overlay; NOTICE/`third_party` path already anticipated | Dual-root CI complexity; large tree size; must document which Cargo workspace is product vs vendor |
| **C** | Path deps only to sibling `../grok-build` | Fast local spike | **Not shippable** for npm/release (ADR 0007 consumers lack sibling) |
| **D** | Continue greenfield “Grok vibes” | — | **Rejected** — fails owner intent (REPLAN §4) |

## Decision

### 1. Chosen strategy: **B — Git subtree pin of Grok Build**

**Product 2.0.0 base runtime is open-source Grok Build, vendored into this repository as a pinned tree.**

| Field | Value |
|-------|--------|
| **Strategy** | **B** (subtree / vendor pin) — not A, C, or D |
| **Vendor root** | `third_party/grok-build/` |
| **Upstream** | `https://github.com/xai-org/grok-build` (Apache-2.0, SpaceXAI) |
| **Pin file** | `third_party/grok-build/SOURCE_REV` (full monorepo/sync SHA from upstream; same semantics as upstream `SOURCE_REV`) |
| **Refresh** | Explicit PR: update subtree + `SOURCE_REV` + NOTICE attribution; never silent copy |
| **1.x crates** | Remain under `crates/dsb-*` as **overlay** (provider, config, L1/L2 policy, legacy REPL entry) |
| **Spike-only** | Sibling `../grok-build` path deps allowed **only** for W0 research (G003); not a release dependency |

**Rationale for B over A**

1. **1.x overlay clarity** — REPLAN prefers B when keeping 1.x crates as overlay is clearer; we already ship scaffold + contracts that must not be deleted.
2. **History & npm continuity** — `@innocarpe/deepseek-build` 1.x remains a honest legacy line; 2.x is a major product change, not a history rewrite of this GitHub repo into a pure Grok fork.
3. **License hygiene** — Apache-2.0 obligations live next to the vendored tree (`LICENSE`, `THIRD-PARTY-NOTICES`, root `NOTICE` update).
4. **Release story** — npm/postinstall and CI can build from **this** repo without requiring a sibling clone.

**Rejected**

- **A** as default for 2.0 — may be revisited only if subtree CI proves unmaintainable; requires ADR amendment.
- **C** for product/release — W0 spike only.
- **D** — permanent reject for 2.0 product identity.

### 2. How the `dsb` / `deepseek-build` binary is produced

| Stage | Behavior |
|-------|----------|
| **Composition root** | A product binary crate (name in W1: e.g. `dsb-pager-bin` or renamed wrapper) that is the **Grok pager composition root** — same class as `xai-grok-pager-bin`, not the thin 1.x clap REPL as default. |
| **Default entry** | No-args TTY → full-screen Grok-class agent (W1). Thin 1.x REPL may remain as `repl-legacy` or non-default only. |
| **Build inputs** | Vendored Grok crates under `third_party/grok-build/crates/...` (workspace members or path deps declared from product `Cargo.toml`) + overlay crates under `crates/dsb-*`. |
| **Artifact names** | Cargo may emit an internal binary name; **install surface** always exposes **`deepseek-build`** and **`dsb`** (ADR 0006). |
| **npm** | Unchanged identity `@innocarpe/deepseek-build` (ADR 0007): wrappers resolve native bin; postinstall/build story must produce the **agent** binary for 2.x (not only legacy REPL). |
| **Config / credentials** | Product paths stay under `~/.deepseek-build/` (not `~/.grok/`). Wire DeepSeek key/setup into the Grok auth/config injection points (W1–W2). |

### 3. Apache-2.0 attribution (mandatory)

When the vendor tree lands (W1 integrate / G004+):

1. Keep upstream `LICENSE` and `THIRD-PARTY-NOTICES` **inside** `third_party/grok-build/`.
2. Root `NOTICE` must name **Grok Build (SpaceXAI)** and point at the vendored path + `SOURCE_REV`.
3. Do **not** rebrand Grok as “DeepSeek Build” **in copyright headers of vendored files** without a deliberate license-compliant change set; product chrome/branding is an overlay (W1-BrandAuth).
4. Product license remains **Apache-2.0** for DeepSeek Build original work.

### 4. `SOURCE_REV` pin semantics

- At integrate time, copy upstream `SOURCE_REV` (or the git SHA used for the subtree commit) into `third_party/grok-build/SOURCE_REV`.
- Any bump of the pin is a **chore/docs+code PR** with evidence that `cargo check` (or product CI) still builds the composition root.
- Agents must not invent a second pin file at repo root that diverges from the vendor pin without updating this ADR.

### 5. Workspace / CI shape (forward-looking; refine in G004)

| Concern | Direction |
|---------|-----------|
| Cargo | Prefer product root `Cargo.toml` as the **release workspace**; vendor crates as members or path deps. Avoid requiring users to `cd third_party/grok-build` for install. |
| CI | Document in W1 if full Grok tree is not yet in `product-ci`; path-filter heavy lanes; shared Rust cache pattern may extend. |
| Toolchain | Align `rust-toolchain.toml` with what the vendored tree needs (document drift in spike + integrate PRs). |
| DotSlash / protoc | Grok build may require DotSlash + `bin/protoc`; product install docs must list these when building from source. |

## Consequences

### Positive

- Product DoD can be honest: **base runtime is Grok**, not greenfield vibes.
- 1.x scaffold stays installable; L1/L2 contracts remain portable as overlays.
- Clear pin + NOTICE path for compliance.

### Trade-offs / costs

- Large tree under `third_party/grok-build/`; clone and CI time grow.
- Periodic upstream sync PRs required.
- Dual package mental model (vendor Grok vs `dsb-*` overlay) until branding/docs fully rewrite for 2.0.

### What later stories must not do

- Re-open strategy D (“skip Grok, ship REPL as 2.0”).
- Default `dsb` to thin REPL after W1 entry lands.
- Drop Apache attribution when copying crates.
- Rely on unpinned sibling `../grok-build` for npm consumers.

## Implementation order (this ADR does not implement)

| Story | Unit | Work |
|-------|------|------|
| G003 | 2x-W0-2, 2x-W0-3 | Spike doc + `cargo check -p xai-grok-pager-bin` on sibling or pin draft |
| G004 | 2x-W1-1 | Land subtree + workspace/CI so composition root builds |
| G005–G006 | 2x-W1-2..4 | Dual bin entry, branding, auth/setup |
| G007–G008 | 2x-W2-* | DeepSeek default + edit loop |
| G009–G010 | 2x-W3-* | L1/L2 under real shell |
| G011–G012 | 2x-W4-* | Install/docs + **v2.0.0** cut |

## PR unit plan (this story)

### PR unit 1 — `docs(adr): accept ADR-0008 Grok Build base strategy B`

- **Intent:** Lock fork-vs-subtree decision, pin path, binary production, Apache/`SOURCE_REV` rules.
- **Touches:** `docs/adr/0008-grok-build-base.md` only (optional one-line index if present).
- **Depends on:** replan docs on main (G001 / #55 family).
- **Parallelizable with:** none required for G002.
- **SemVer:** none (docs).
- **Tests:** file exists; links resolve; strategy B explicit; D rejected.

## References

- Upstream README / layout: sibling `../grok-build` or `https://github.com/xai-org/grok-build`
- Local pin sample (pre-vendor): `../grok-build/SOURCE_REV` = `4d6d11372ab8f73026a78c45a7b7e7b1310eb39f` as of W0 authoring date (refresh at integrate)
- REPLAN §4 strategy table
- ADR 0006 (CLI names), ADR 0007 (npm)
