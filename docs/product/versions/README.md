# Product version lines (major targets)

**Status:** Normative index for major-line PRDs  
**SemVer:** Always full `MAJOR.MINOR.PATCH` ([versioning.md](../../contributing/versioning.md))  
**Code truth:** root `Cargo.toml` / `package.json` on `main`

When agents or humans start a train, **pick the major line first**, then the PRD for that line. Do not invent a new major without updating this index + a `PRD-vN.md`.

---

## Line map (current)

| Line | PRD | Status | One-line identity |
|------|-----|--------|-------------------|
| **1.x** | [PRD-v1.md](../PRD-v1.md) | **Shipped / legacy scaffold** | Thin clap agent + contracts (`1.0.0`–`1.1.0`) — **not** final product DoD |
| **2.x** | [PRD-v2.md](../PRD-v2.md) | **Shipped shell cut** | Grok-derived full-screen agent + DeepSeek entry/UI/npm (`2.0.0`–`2.0.3+`) |
| **3.x** | [PRD-v3.md](../PRD-v3.md) | **Shipped heart fusion (`3.0.0`)** | L1+L2 P0 under Path A — [HEART_3X_GOALS.md](../HEART_3X_GOALS.md) · cut [CUT_3_0_0_2026-08-07.md](../evidence/CUT_3_0_0_2026-08-07.md) |
| **4.x** | [PRD-v4.md](../PRD-v4.md) | **Next major (after `v3.0.0`)** | L3 productization — [PARALLEL_3X_4X_PLAN.md](../PARALLEL_3X_4X_PLAN.md) · [WAVE_4x_PR_DAG.md](../WAVE_4x_PR_DAG.md) · [FLEET_4X_GOALS.md](../FLEET_4X_GOALS.md) |

Historical scaffold waves (A–D) remain under [prd/](../prd/) and are **not** product major PRDs.

---

## Rules

1. **One PRD per major line** (`PRD-v1`, `PRD-v2`, …). Minors (`2.0.1`) are changelog + release notes, not new PRDs unless behavior identity shifts.
2. **Honesty table required** in each PRD: *claimed vs shipped* for that line.
3. **L1/L2/L3 layers** ([HARNESS_PHILOSOPHY.md](../../architecture/HARNESS_PHILOSOPHY.md)) must appear in every major PRD’s architecture section.
4. **Never unpublish** older npm majors; mark legacy in messaging only.
5. **Next train** after 2.x polish defaults to **3.x** (PRD-v3), not restarting A–D or inventing overnight plan-ids.

---

## Decision log (major targets)

| Date | Decision | Record |
|------|----------|--------|
| 2026-08-06 | 1.x = scaffold; real product cut re-versioned to 2.0.0 | [REPLAN_2.0.md](../REPLAN_2.0.md) |
| 2026-08-06–07 | 2.0.0–2.0.3 shipped (Grok base + DeepSeek entry/UI/npm) | PRD-v2, tags `v2.0.0`…`v2.0.3` |
| 2026-08-07 | Unfinished L1/L2 fusion **not** all stuffed into a single “dump”; **3.0.0 = heart fusion P0**, **4.0.0 = L3 max** | PRD-v3, PRD-v4 |
| 2026-08-07 | **`3.0.0` / tag `v3.0.0`** heart fusion cut (`heart-3x` G001–G008) | PRD-v3, CUT_3_0_0 evidence |

---

## Related

| Doc | Role |
|-----|------|
| [SSOT.md](../SSOT.md) | Priority order including this index |
| [REPLAN_2.0.md](../REPLAN_2.0.md) | Historical replan that defined 2.0.0 intent |
| [KNOWN_LIMITS.md](../KNOWN_LIMITS.md) | Runtime honesty for current SemVer on disk |
| [CHANGELOG.md](../../../CHANGELOG.md) | Per-release notes |
