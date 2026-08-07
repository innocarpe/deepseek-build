# Implementation gates ledger

**Purpose:** Make G0–G6 **auditable facts**, not self-attestation.  
**Normative definitions:** [architecture/HARNESS_PHILOSOPHY.md](architecture/HARNESS_PHILOSOPHY.md) §11.  
**SSOT priority:** [product/SSOT.md](product/SSOT.md).

| Gate | Requirement | Status | Evidence (PR / path) | Flipped by |
|------|-------------|--------|----------------------|------------|
| **G0** | HARNESS_PHILOSOPHY + layered SOURCES merged | **green** | PR #4 | innocarpe |
| **G1** | Toolchain/config ADR | **green** | `docs/adr/0004-toolchain.md` | innocarpe |
| **G1b** | DeepSeek provider contract ADR (pinned ids) | **green** | `docs/adr/0005-deepseek-provider-contract.md` | innocarpe |
| **G2** | Specs **10, 15, 20, 30** ready-for-impl | **green** | specs 10/15/20/30 | innocarpe |
| **G3** | Specs **45** + **90 minimum** ready | **green** | specs 45/90 | innocarpe |
| **G4** | Spec **50** ready | **green** | `docs/specs/50-parallelism-background.md` + agent parallel readonly (**0.12.0**) | innocarpe |
| **G5** | Spec **60** ready | **green** | `docs/specs/60-subagents.md` + in-process workers (**0.14.0**) | innocarpe |
| **G6a** | Spec **100** sessions ready | **green** | `docs/specs/100-sessions.md` + runtime **0.5.0** (#24) | innocarpe |
| **G6b** | Spec **70** skills ready | **green** | `docs/specs/70-skills.md` + runtime **0.6.0** (#25) | innocarpe |
| **G6c** | Spec **80** MCP ready | **green** | `docs/specs/80-mcp.md` + `dsb-tools` mcp catalog/fingerprint (**0.11.0**) | innocarpe |
| **G6d** | Spec **110** plan light ready | **green** | `docs/specs/110-plan-mode.md` + `plan` tool (**0.11.0**) | innocarpe |

**Legacy label G6:** means “all of G6a–G6d green.” Partial progress is tracked per subgate.

## Rules

1. **No runtime feature PR** may claim a gate is green without updating this table in the same PR (or a prior merged PR).  
2. **ready-for-impl** for specs **10, 15, 45, 50, 90, 100, 70** (and others marked automated) requires **automated** golden/negative tests in the test plan.  
3. Adding package code is allowed after G1 green.  
4. **Process-police CI** stays forbidden. Product CI (build/test/smoke) is allowed and encouraged.  
5. Who may flip a gate: maintainer on merge of the evidence PR; record login + PR number.  
6. **Sessions runtime requires G6a**; **skills runtime requires G6b**; **MCP requires G6c**; **plan product requires G6d**. Spec-only PRs may land while red; runtime must flip the subgate.

## Current product implication

- **Wave A dogfood** through **`0.7.0` npm package** shipped on `main` (install + tools + sessions + surface min + npm wrappers).  
- **Registry `npm publish`** remains **human-gated** ([ADR 0007](adr/0007-npm-packaging.md)).  
- Spec **40** is **ready-for-impl** (`docs/specs/40-core-tools-surface.md`); it is **not** a G-number gate (G3 remains 45+90).  
- **2.x shell** shipped. **3.x / 4.x tags exist** as heart/L3 *attempts* — **owner-bar NOT MET** (Path A fusion incomplete).  
- Spec-ready (this table) ≠ Path A enforced. Historical heart evidence is archive only: [HEART_3X_SPEC_BINDING.md](architecture/HEART_3X_SPEC_BINDING.md) · [WAVE_3x_PR_DAG.md](product/WAVE_3x_PR_DAG.md).  
- **Owner-bar complete product** is **`5.0.0`** only when [OWNER_BAR_P0_LEDGER.md](product/OWNER_BAR_P0_LEDGER.md) all PASS on Path A ([OWNER_BAR_ACCEPTANCE.md](product/OWNER_BAR_ACCEPTANCE.md)).

**Ultragoal (product):** **`owner-bar-5x`** → tag **`v5.0.0`** ([ULTRAGOAL_CHAIN.md](product/ULTRAGOAL_CHAIN.md) · [OWNER_BAR_5X_GOALS.md](product/OWNER_BAR_5X_GOALS.md)).  
Do **not** resume `heart-3x` / `fleet-4x` as product SSOT. Gate: `./scripts/test-owner-bar.sh` (RED until fusion).
