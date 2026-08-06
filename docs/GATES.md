# Implementation gates ledger

**Purpose:** Make G0–G6 **auditable facts**, not self-attestation.  
**Normative definitions:** [architecture/HARNESS_PHILOSOPHY.md](architecture/HARNESS_PHILOSOPHY.md) §11.

| Gate | Requirement | Status | Evidence (PR / path) | Flipped by |
|------|-------------|--------|----------------------|------------|
| **G0** | HARNESS_PHILOSOPHY + layered SOURCES merged | **green** | PR #4 (`docs/architecture/HARNESS_PHILOSOPHY.md`) | innocarpe |
| **G1** | Toolchain/config ADR (language, binary, state dir, secrets) | **red** | — | — |
| **G1b** | DeepSeek provider contract ADR (model ids pinned, stream, thinking/effort, cache usage fields) | **red** | — | — |
| **G2** | Specs **10, 15, 20, 30** = ready-for-impl | **red** | only `docs/specs/00-overview.md` exists | — |
| **G3** | Specs **45** + **90 minimum** ready; shell not mutating without 90 | **red** | — | — |
| **G4** | Spec **50** ready (parallel semantics) | **red** | — | — |
| **G5** | Spec **60** ready (worker cache law measurable) | **red** | — | — |
| **G6** | Specs **70, 80, 100, 110** ready | **red** | — | — |

## Rules

1. **No runtime feature PR** may claim a gate is green without updating this table in the same PR (or a prior merged PR).  
2. **ready-for-impl** for specs **10, 15, 45, 50, 90** requires **automated** golden/negative tests in the test plan (manual-only is not enough). UX-only specs (e.g. 30 display polish, 110 plan UX) may use manual checks.  
3. `crates/` **directory placeholder** (README only) is **not** G1 violation. Adding real package code / Cargo workspace members **is** G1.  
4. **Process-police CI** (PR title regex, kind-label count, random markdown inventory) stays forbidden.  
   **Artifact existence checks** (gate file, required specs present when claiming G2) are **allowed** later as product hygiene—not “process theater.”  
5. Who may flip a gate: maintainer on merge of the evidence PR; record login + PR number.

## Current product implication

**Runtime / ultragoal coding must not start** until **G1 + G1b + G2** are green.
