# PRD v4 — DeepSeek Build **4.x** (L3 productization)

| Field | Value |
|-------|--------|
| **SemVer line** | **`4.0.0` – `4.x.y`** (after 3.x heart fusion) |
| Status | **`4.0.0` cut shipped** (`fleet-4x` complete; patches `4.0.1`+ install/UX) |
| Owner | @innocarpe |
| Last updated | 2026-08-07 |
| Index | [versions/README.md](./versions/README.md) |
| Depends on | [PRD-v3.md](./PRD-v3.md) heart fusion P0 (`v3.0.0`) |
| Parallel ops | [PARALLEL_3X_4X_PLAN.md](./PARALLEL_3X_4X_PLAN.md) |
| PR DAG | [WAVE_4x_PR_DAG.md](./WAVE_4x_PR_DAG.md) |
| Board | [FLEET_4X_GOALS.md](./FLEET_4X_GOALS.md) (`fleet-4x` — complete) |

---

## 1. Problem

After 3.0.0, L1/L2 hearts should hold under the Grok-derived shell. Remaining gap vs owner “speed champion” intent:

- Grok **L3 mechanisms** (parallel tools, bg shell, subagents, worktrees) may still be under-used or under-documented as **DeepSeek Build product defaults**.  
- 2.x/3.x may still feel like “single agent session” rather than **orchestrated throughput**.

---

## 2. Why a separate major

| If mixed into 3.0.0 | Risk |
|---------------------|------|
| L1/L2 contracts + L3 fleet UX same cut | Mega scope; incomplete contracts ship with “speed” features |
| Harder honesty | Easy to claim “fusion done” while only polishing parallel UI |

**4.0.0** is reserved for **L3 productization** once hearts are true.

**Prep is not the code train:** during heart-3x, only docs/research/evidence (WAVE_4x **4x-P0-***) may land on `main`. See [PARALLEL_3X_4X_PLAN.md](./PARALLEL_3X_4X_PLAN.md).

---

## 3. Product definition of done — **`4.0.0`**

### P0 (ship blockers)

1. **Capability matrix** of L3 surfaces (parallel, bg, subagent, worktree) with product-facing paths.  
2. **Product defaults / profiles** use Grok-native parallel + bg wait patterns (not 1.x shims), **without** weakening L1/L2.  
3. **Subagent** and **worktree** flows dogfoodable as product features (docs + evidence).  
4. User-facing docs teach multi-step throughput as first-class.  
5. Heart regression + pre-3x live still green.  
6. Tag **`v4.0.0`** only when above hold. Full SemVer only.

### Explicit non-goals

- Replacing DeepSeek identity  
- Multi-vendor as core  
- Re-opening L1/L2 regressions “for speed” (HARNESS fail-close)  
- Greenfield agent replacing Grok base  
- Claiming 4.0.0 from documentation alone  

---

## 4. Ultragoal shape

| When | Plan | What |
|------|------|------|
| During 3.0 | *no fleet ultragoal* | Parallel plan + WAVE_4x draft + gap inventory on `main` |
| After `v3.0.0` | **`fleet-4x`** | G001–G008 → tag **`v4.0.0`** — **complete** |

Cold start (historical): [ULTRAGOAL_PROMPT_COLD_START_4.0.md](./ULTRAGOAL_PROMPT_COLD_START_4.0.md).

---

## 5. Status

| Item | State |
|------|-------|
| 4.x ultragoal train | **Complete** `fleet-4x` → tag **`v4.0.0`** (PR #85) |
| Prep on main | **Complete** (Lane B + 4x-P0-1..5) |
| Code defaults for fleet | **Shipped** at `4.0.0` — keep `yolo = false` / Path A hearts |
| Install DX patches | **`4.0.1`** prebuilt npm (ADR 0009) · **`4.0.2`** setup → bare `dsb` |
| Evidence | [CUT_4_0_0_2026-08-07.md](./evidence/CUT_4_0_0_2026-08-07.md) |

### Residual after 4.0.0 (not blockers for the cut)

| Topic | Notes |
|-------|--------|
| Live L3 extended smoke | Env-gated (API key) |
| Prebuilt platforms beyond darwin-arm64 | CI `release-prebuilt.yml` + assets on tag |
| Full Spec 45 snippet_id mint | 3.x residual (file_version equivalent) |
