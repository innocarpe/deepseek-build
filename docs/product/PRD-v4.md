# PRD v4 — DeepSeek Build **4.x** (L3 productization)

| Field | Value |
|-------|--------|
| **SemVer line** | **`4.0.0` – `4.x.y`** (planned; after 3.x heart fusion) |
| Status | **Planned** — not active train |
| Owner | @innocarpe |
| Last updated | 2026-08-07 |
| Index | [versions/README.md](./versions/README.md) |
| Depends on | [PRD-v3.md](./PRD-v3.md) heart fusion P0 |

---

## 1. Problem

After 3.0.0, L1/L2 hearts should hold under the Grok-derived shell. Remaining gap vs owner “speed champion” intent:

- Grok **L3 mechanisms** (parallel tools, bg shell, subagents, worktrees) may still be under-used or under-documented as **DeepSeek Build product defaults**.  
- 2.x/3.x may still feel like “single agent session” rather than **orchestrated throughput**.

## 2. Why a separate major

| If mixed into 3.0.0 | Risk |
|---------------------|------|
| L1/L2 contracts + L3 fleet UX same cut | Mega scope; incomplete contracts ship with “speed” features |
| Harder honesty | Easy to claim “fusion done” while only polishing parallel UI |

**4.0.0** is reserved for **L3 productization** once hearts are true.

## 3. Draft P0 (will refine at 3.0.0 exit)

1. Default agent profiles use **Grok-native** parallel tools / bg wait patterns (not 1.x shims).  
2. Subagent / worktree flows dogfoodable as product features (docs + evidence).  
3. Product docs teach “fast multi-step” workflows as first-class.  
4. Tag **`v4.0.0`** only with evidence; full SemVer.

## 4. Non-goals (4.0.0 draft)

- Replacing DeepSeek identity  
- Multi-vendor as core  
- Re-opening L1/L2 regressions “for speed” (fail-close HARNESS rule)

## 5. Status

Do **not** start a `4.x` ultragoal train until:

1. [PRD-v3.md](./PRD-v3.md) `3.0.0` P0 is green or explicitly amended, and  
2. A `WAVE_4x_PR_DAG.md` (or equivalent) lands on main.
