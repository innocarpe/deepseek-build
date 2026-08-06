# Implementation gates ledger

**Purpose:** Make G0–G6 **auditable facts**, not self-attestation.  
**Normative definitions:** [architecture/HARNESS_PHILOSOPHY.md](architecture/HARNESS_PHILOSOPHY.md) §11.

| Gate | Requirement | Status | Evidence (PR / path) | Flipped by |
|------|-------------|--------|----------------------|------------|
| **G0** | HARNESS_PHILOSOPHY + layered SOURCES merged | **green** | PR #4 | innocarpe |
| **G1** | Toolchain/config ADR | **green** | `docs/adr/0004-toolchain.md` (this preflight PR) | innocarpe |
| **G1b** | DeepSeek provider contract ADR (pinned ids) | **green** | `docs/adr/0005-deepseek-provider-contract.md` | innocarpe |
| **G2** | Specs **10, 15, 20, 30** ready-for-impl | **green** | `docs/specs/10-cache-contract.md`, `15-tool-call-repair.md`, `20-model-routing.md`, `30-thinking-effort.md` | innocarpe |
| **G3** | Specs **45** + **90 minimum** ready | **red** | — | — |
| **G4** | Spec **50** ready | **red** | — | — |
| **G5** | Spec **60** ready | **red** | — | — |
| **G6** | Specs **70, 80, 100, 110** ready | **red** | — | — |

## Rules

1. **No runtime feature PR** may claim a gate is green without updating this table in the same PR (or a prior merged PR).  
2. **ready-for-impl** for specs **10, 15, 45, 50, 90** requires **automated** golden/negative tests in the test plan (manual-only is not enough). UX-only specs (e.g. 30 display polish, 110 plan UX) may use manual checks.  
3. `crates/` **directory placeholder** (README only) is **not** G1 violation. Adding real package code / Cargo workspace members **is** allowed only after G1 green (now).  
4. **Process-police CI** stays forbidden. Artifact existence checks are allowed.  
5. Who may flip a gate: maintainer on merge of the evidence PR; record login + PR number.

## Current product implication

**M1 runtime implementation may start** once this ledger is green for G0–G2 (after this PR merges).  

Still **blocked for M2+** until G3+ as listed.  

**Ultragoal recommendation:** implement M1 against specs 10/15/20/30 + ADR 0004/0005 only; do not start snippet edit/shell (45/90) until those specs exist.