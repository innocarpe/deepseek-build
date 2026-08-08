# Product version lines (major targets)

**Status:** Normative index for major-line PRDs
**SemVer:** Always full `MAJOR.MINOR.PATCH` ([versioning.md](../../contributing/versioning.md))
**Code truth:** root `Cargo.toml` / `package.json` on `main`

When agents or humans start a train, **pick the major line first**, then the PRD for that line. Do not invent a new major without updating this index + a `PRD-vN.md`.

**Product completeness gate (fail-close):**
[OWNER_BAR_ACCEPTANCE.md](../OWNER_BAR_ACCEPTANCE.md) — *Is this the real DeepSeek Coding Agent + TUI?*
Tags and line PRDs do **not** override that file.

---

## Line map (current)

| Line | PRD | Status | One-line identity |
|------|-----|--------|-------------------|
| **1.x** | [PRD-v1.md](../PRD-v1.md) | **Shipped / legacy scaffold** | Thin clap agent + contracts — **not** final product DoD |
| **2.x** | [PRD-v2.md](../PRD-v2.md) | **Shipped shell cut** | Grok-derived full-screen agent + DeepSeek entry/UI/npm |
| **3.x** | [PRD-v3.md](../PRD-v3.md) | **Tagged `3.0.0` — owner-bar NOT MET** | Heart fusion *attempt*; Path A fusion incomplete (library / dead wiring) — [OWNER_BAR_ACCEPTANCE.md](../OWNER_BAR_ACCEPTANCE.md) |
| **4.x** | [PRD-v4.md](../PRD-v4.md) | **Tagged `4.0.0`–`4.0.2` — owner-bar NOT MET** | L3 productization *attempt*; machinery + docs, not full product identity |
| **5.x** | [PRD-v5.md](../PRD-v5.md) | **Owner-bar MET (`5.0.0`)** · **vision-complete `5.5.0` merged on `main`** · release publish pending | Owner-bar cut [CUT_5_0_0](../evidence/CUT_5_0_0_2026-08-07.md) · completed vision board [VISION_COMPLETE_5X_GOALS.md](../VISION_COMPLETE_5X_GOALS.md) |

Historical scaffold waves (A–D) remain under [prd/](../prd/) and are **not** product major PRDs.

---

## Rules

1. **One PRD per major line** (`PRD-v1`, `PRD-v2`, …). Minors (`2.0.1`) are changelog + release notes, not new PRDs unless behavior identity shifts.
2. **Honesty table required** in each PRD: *claimed vs shipped* for that line.
3. **L1/L2/L3 layers** ([HARNESS_PHILOSOPHY.md](../../architecture/HARNESS_PHILOSOPHY.md)) must appear in every major PRD’s architecture section.
4. **Never unpublish** older npm majors; mark legacy in messaging only.
5. **Owner-bar cut** (planned `5.0.0`): every P0 in [OWNER_BAR_ACCEPTANCE.md](../OWNER_BAR_ACCEPTANCE.md) / [OWNER_BAR_P0_LEDGER.md](../OWNER_BAR_P0_LEDGER.md) green on **Path A** only; Path B unit tests alone = fail.
6. **Next train** for the complete product is **5.x / owner bar**, not another documentation-only 3.x/4.x re-cut.

---

## Decision log (major targets)

| Date | Decision | Record |
|------|----------|--------|
| 2026-08-06 | 1.x = scaffold; real product cut re-versioned to 2.0.0 | [REPLAN_2.0.md](../REPLAN_2.0.md) |
| 2026-08-06–07 | 2.0.0–2.0.3 shipped (Grok base + DeepSeek entry/UI/npm) | PRD-v2, tags `v2.0.0`…`v2.0.3` |
| 2026-08-07 | Unfinished L1/L2 fusion **not** all stuffed into a single “dump”; **3.0.0 = heart fusion P0**, **4.0.0 = L3 max** | PRD-v3, PRD-v4 |
| 2026-08-07 | **`3.0.0` / tag `v3.0.0`** heart fusion cut (`heart-3x`) — later found **not owner-bar green** | PRD-v3, CUT_3_0_0, adversarial review |
| 2026-08-07 | **`4.0.0` / tag `v4.0.0`** L3 productization (`fleet-4x`) — **not owner-bar green** | PRD-v4, PR #85, CUT_4_0_0 |
| 2026-08-07 | **`4.0.1`** prebuilt npm install (ADR 0009) · **`4.0.2`** setup → bare `dsb` | PR #86, #87 |
| 2026-08-07 | **`4.0.3`** `dsb --resume` surface + hint branding | PR #92 |
| 2026-08-07 | **`5.0.0` / tag `v5.0.0`** owner-bar complete (`owner-bar-5x`) | PRD-v5, CUT_5_0_0, dual adversarial reviews |
| 2026-08-07 | **`5.1.0`** DeepSeek Night v2 measured default theme | (this release) |
| 2026-08-08 | **`5.2.0`** theme classic default + vision complete + theme picker restore | PR #_(fill in)_ |
| 2026-08-08 | **`5.2.1`** DeepSeek Night v2 markdown hierarchy restore (h2/code/command hues) | PR #131 |
| 2026-08-08 | **`5.2.2`** installer self-check + fresh inode (fix silent corrupt install) | PR #_(fill in)_ |
| 2026-08-08 | **`5.3.0`** Spec 45 Path A snippet_id multi-edit R0A Deep Code cut | PR #138 |
| 2026-08-08 | **`5.4.0`** L3 Path A R0A train cut (multi-tool/bg, subagent/worker-cache, worktree dogfood) + optional live L3 matrix | PR #145 |
| 2026-08-08 | **`5.5.0`** vision-complete freeze: V1-V4 Path A criteria + V3-60-3 parent snippet after worker R0A (merged on `main`; release publish pending) | PR #147 |
| 2026-08-07 | **`5.0.1`** widen the DeepSeek whale logo to official terminal proportions | PR #113 |
| 2026-08-07 | **`4.0.4`** Image attachments on text-only DeepSeek endpoints (persist to session assets + OCR hint); DeepSeek status line with account balance & cache hit rate; G003 mint file_version on Path A read_file | PR #_(fill in)_ |
| 2026-08-07 | **release-cycle harness** — `bump-version.sh` + `release.sh` + CI sccache/fast-path + runbook | PR #94 |
| 2026-08-07 | Owner-bar checklist normative; true complete product = **5.x** only when checklist green | [OWNER_BAR_ACCEPTANCE.md](../OWNER_BAR_ACCEPTANCE.md) |
| 2026-08-07 | Dual adversarial plan review (Claude Opus + Codex gpt-5.6-sol); train **`owner-bar-5x`** package | [evidence/OWNER_BAR_5X_PLAN_ADVERSARIAL_2026-08-07.md](../evidence/OWNER_BAR_5X_PLAN_ADVERSARIAL_2026-08-07.md) · PRD-v5 · WAVE_5x |

---

## Related

| Doc | Role |
|-----|------|
| **[OWNER_BAR_ACCEPTANCE.md](../OWNER_BAR_ACCEPTANCE.md)** | **Product done?** Path A P0 checklist |
| **[OWNER_BAR_P0_LEDGER.md](../OWNER_BAR_P0_LEDGER.md)** | Frozen machine P0 list for `5.0.0` |
| [SSOT.md](../SSOT.md) | Priority order including this index |
| [REPLAN_2.0.md](../REPLAN_2.0.md) | Historical replan that defined 2.0.0 intent |
| [KNOWN_LIMITS.md](../KNOWN_LIMITS.md) | Runtime honesty for current SemVer on disk |
| [CHANGELOG.md](../../../CHANGELOG.md) | Per-release notes |
