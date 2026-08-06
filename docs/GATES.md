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
| **G4** | Spec **50** ready | **red** | — | — |
| **G5** | Spec **60** ready | **red** | — | — |
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
- **Wave B** (`native-0x`): Spec **40** tools surface (`0.8.0`); theme, full permissions UX, expand skills, MCP/plan — needs G6c/G6d for those runtimes.  
- Spec **40** is **ready-for-impl** (`docs/specs/40-core-tools-surface.md`); it is **not** a G-number gate (G3 remains 45+90).  
- **Wave C**: G4 then G5.  

**Ultragoal:** after `dogfood-0x` complete → `native-0x` ([ULTRAGOAL_CHAIN.md](product/ULTRAGOAL_CHAIN.md)).
