# Ultragoal board — **`heart-3x`** → **`3.0.0`**

**Plan id:** `heart-3x`  
**DoD:** [PRD-v3.md](./PRD-v3.md) §3 P0  
**PR units:** [WAVE_3x_PR_DAG.md](./WAVE_3x_PR_DAG.md)  
**Cold start:** [ULTRAGOAL_PROMPT_COLD_START_3.0.md](./ULTRAGOAL_PROMPT_COLD_START_3.0.md)  
**Baseline:** [PRE_3X_TEST_MATRIX.md](./PRE_3X_TEST_MATRIX.md)  
**Previous product plan:** `grokbase-2x` → 2.x shell (**complete for product entry**)

---

## Rules

1. **One plate** until `3.0.0` tagged — do not invent a second plan-id mid-train.  
2. **No `--force` wipe** of an in-progress ledger.  
3. **PR plan before code** for each story ([ULTRAGOAL_PR_PLANNING.md](./ULTRAGOAL_PR_PLANNING.md)).  
4. **Child runtime = parent** (Grok session → `grok` only unless user explicitly crosses).  
5. **Full SemVer** only (`3.0.0`, never `3.0`).  
6. **CLI:** `deepseek-build` primary · `dsb` alias.  
7. Everyday tests: `./scripts/test-pre3x-baseline.sh --live` — **not** vendor-full.

---

## Stories (G001 → G008)

| ID | Title | WAVE_3x units | Band | Done when |
|----|-------|---------------|------|-----------|
| **G001** | PrepOnMain | 3x-P0-1 | `2.0.x` | Agent model `base_url` + pre-3x harness on `main`; T4.0 green |
| **G002** | PlanOnMain | 3x-P0-2 | docs | WAVE_3x + this board + cold-start 3.0 on `main`; chain points here |
| **G003** | SpecMap | 3x-H0-1, 3x-H0-2 | docs/spec | [HEART_3X_SPEC_BINDING.md](../architecture/HEART_3X_SPEC_BINDING.md) + [HEART_3X_P0_TEST_PLAN.md](./HEART_3X_P0_TEST_PLAN.md) |
| **G004** | L1-Snippet | 3x-H1-1 | `3.0.0-alpha.N` | Spec 45 spirit enforced on default Grok edit path + tests |
| **G005** | L1-Permissions | 3x-H1-2, 3x-H1-3 | alpha | Spec 90 matrix + dogfood evidence; H1 exit |
| **G006** | L2-Prefix | 3x-H2-1 | `3.0.0-beta.N` | Prefix/epoch under agent context stack + tests |
| **G007** | L2-RepairRoute | 3x-H2-2, 3x-H2-3 | beta | Repair + Flash/Pro under agent; H2 exit |
| **G008** | Cut-3.0.0 | 3x-H3-1..3 | **3.0.0** | Honesty docs + gates + tag **`v3.0.0`** only |

---

## Create ledger (once)

```bash
# After G002 docs are on main — adjust omc CLI if project uses a wrapper
omc ultragoal create-goals --plan-id heart-3x \
  --goal "G001 PrepOnMain::base_url + pre-3x harness on main; T4.0 green" \
  --goal "G002 PlanOnMain::WAVE_3x + HEART_3X + cold-start 3.0 on main" \
  --goal "G003 SpecMap::Spec 45/90/10/15/20 binding under Grok agent path" \
  --goal "G004 L1-Snippet::snippet-safe edit on Grok tools + tests (3.0.0-alpha)" \
  --goal "G005 L1-Permissions::ask/deny/allow + headless fail-closed + dogfood" \
  --goal "G006 L2-Prefix::prefix/epoch under agent context (3.0.0-beta)" \
  --goal "G007 L2-RepairRoute::tool-call repair + Flash/Pro under agent" \
  --goal "G008 Cut-3.0.0::honesty docs + evidence + tag v3.0.0"
```

If the plan already exists with progress: **`omc ultragoal status --plan-id heart-3x`** only — never `--force` recreate.

```bash
omc ultragoal complete-goals --plan-id heart-3x
```

---

## Operator loop

```bash
git fetch origin && git checkout main && git pull origin main
./scripts/test-pre3x-baseline.sh --live   # when API key available
omc ultragoal status --plan-id heart-3x
omc ultragoal complete-goals --plan-id heart-3x
# Active story only → PR units from WAVE_3x → squash-merge → checkpoint → complete-goals again
```

Stop when **8/8 complete** or blocked with written evidence (not calendar wait).

---

## Non-goals (fail-close)

- Claiming 3.0.0 from 2.x shell + paint alone  
- L3 fleet identity (→ 4.x)  
- Restarting `grokbase-2x` / A–D as product SSOT  
- Multi-vendor core identity  
- Everyday `third_party/grok-build` full test (disk bomb)
