# Single source of truth (priority order)

When documents disagree, **higher wins**. Agents must not invent a third story.

| Priority | Artifact | Owns |
|----------|----------|------|
| 1 | **Git `main` code + `Cargo.toml` version** | What actually ships |
| 2 | **[REPLAN_2.0.md](./REPLAN_2.0.md)** | **Product direction from 2026-08-06:** Grok base, 2.0.0 DoD, 1.x scaffold honesty |
| 3 | **[WAVE_2x_PR_DAG.md](./WAVE_2x_PR_DAG.md)** | Fixed product PR units (W0–W4) — no overnight invention |
| 4 | **Ultragoal `goals.json` story status** (local `.omc/…/goals.json`) | Which story is next *in this machine’s ledger* (must not contradict REPLAN) |
| 5 | **[MASTER_PLAN.md](./MASTER_PLAN.md)** | Historical A–D scaffold + pointer to replan |
| 6 | **Wave PRD** `prd/PRD-wave-*.md` | Scaffold-era exit criteria (historical) |
| 7 | **Fixed PR DAG** `WAVE_A_PR_DAG.md` / `WAVE_B_PR_DAG.md` | Scaffold unit splits (historical) |
| 8 | **RELEASE_TRAIN_0x.md** | Wave A narrative only |
| 9 | Cold-start prompts | Session bootstrap — prefer `ULTRAGOAL_PROMPT_COLD_START_2.0.md` for product work |

## Gate ledger

[GATES.md](../GATES.md) wins for **whether a runtime feature may land**.  
Subgates: **G6a** sessions(100), **G6b** skills(70), **G6c** MCP(80), **G6d** plan(110).

## Version display fields to update on every minor release

One release PR should touch (as applicable):

1. `Cargo.toml` workspace version  
2. `package.json` version (if present)  
3. `MASTER_PLAN.md` §2 / §4 checklists / §8 log  
4. `RELEASE_TRAIN_0x.md` §2 / §7 (Wave A only)  
5. user-guide version lines  
6. README status blurb if it hardcodes SemVer  

Run: `./scripts/check-semver.sh` and `npm run version-check` when npm exists.
