# Ultragoal cold-start — product **`fleet-4x`** → **`4.0.0`**

**Use after `v3.0.0` is on `main`.**  
Workspace = **`deepseek-build` git root** (prefer clean worktree; not dirty heart WIP).

Paste **everything inside the single fenced `text` block below** into a **new** agent session.

Optional mid-plan:

> Resume `fleet-4x`. Run `omc ultragoal status --plan-id fleet-4x`. Continue active story only — no `--force` recreate.

---

```text
# ROLE

You ship DeepSeek Build **4.0.0** (L3 productization) after heart fusion.
Cold start — load truth from repo, git, ultragoal ledger only.
Child runtime = parent family (**grok** only unless user explicitly crosses).

# FINAL GOAL (immutable)

Normative: docs/product/PRD-v4.md · docs/product/WAVE_4x_PR_DAG.md · docs/product/L3_CAPABILITY_MATRIX.md

1. L3 capability matrix published (parallel, bg, subagent, worktree).
2. Product defaults enable subagents; keep **yolo = false** (hearts).
3. Worktree/subagent/bg documented as product features + smoke.
4. Heart path_a tests still green; pre3x offline green.
5. Honesty docs: 3.0.0 hearts; 4.0.0 L3 productization.
6. Tag **v4.0.0** only when P0 green. Full SemVer MAJOR.MINOR.PATCH only.

# NON-GOALS

- Weakening L1/L2 for speed
- Multi-vendor core
- Greenfield agent
- Restarting heart-3x as product SSOT
- Everyday vendor-full cargo test disk bomb
- npm publish without human if OTP required (ADR 0007)

# WHERE WE ARE

- 3.0.0 heart fusion shipped (tag v3.0.0, npm 3.0.0)
- Plan id: **fleet-4x** — G001→G008
- Board: docs/product/FLEET_4X_GOALS.md
- Ops: docs/product/PARALLEL_3X_4X_PLAN.md
- Smoke: ./scripts/test-l3-smoke.sh
- Chain: docs/product/ULTRAGOAL_CHAIN.md active = fleet-4x

Verify:
```bash
git fetch origin && git checkout main && git pull
git tag -l v3.0.0 | grep -q . || exit 1
test -f docs/product/WAVE_4x_PR_DAG.md
rg -n 'ready-for-impl' docs/product/WAVE_4x_PR_DAG.md
./scripts/check-semver.sh
omc ultragoal status --plan-id fleet-4x || true
```

# STORIES

| # | Story | Done when |
|---|-------|-----------|
| G001–G004 | Prep assets | already on main from Lane B |
| G005 | TrainStart | WAVE ready-for-impl + this cold-start |
| G006 | L3-Matrix | L3_CAPABILITY_MATRIX + heart regression |
| G007 | L3-Defaults | product seed subagents + docs + smoke |
| G008 | Cut-4.0.0 | SemVer 4.0.0 + tag + evidence + npm human/agent |

# PROCESS

PR plan before code; atomic commits; English GH text; kind labels;
merge with **merge commit** (squash disabled).
complete-goals → unit → PR → merge → checkpoint → next until 8/8.

# CREATE (if missing)

See FLEET_4X_GOALS.md create-goals block. No --force if progress exists.

# END
```
