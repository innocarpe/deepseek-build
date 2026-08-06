# Single source of truth (priority order)

When documents disagree, **higher wins**. Agents must not invent a third story.

| Priority | Artifact | Owns |
|----------|----------|------|
| 1 | **Git `main` code + `Cargo.toml` version** | What actually ships |
| 2 | **[versions/README.md](./versions/README.md)** + active major **PRD-vN** | **Which major line is product target** and its DoD |
| 3 | **[PRD-v2.md](./PRD-v2.md)** (while 2.x is current ship line) / **[PRD-v3.md](./PRD-v3.md)** (when heart train starts) | Line-specific problem, goals, honesty |
| 4 | **[HARNESS_PHILOSOPHY.md](../architecture/HARNESS_PHILOSOPHY.md)** | L1/L2/L3 layer ownership (never silent override) |
| 5 | **[REPLAN_2.0.md](./REPLAN_2.0.md)** | Historical replan that defined 2.0.0 cut intent |
| 6 | **WAVE / ultragoal boards** (`WAVE_2x_*`, `GROKBASE_2X_*`, future `WAVE_3x_*`) | Fixed PR units for the **active** train |
| 7 | **Ultragoal `goals.json` story status** (local `.omc/…`) | Next story *on this machine* (must not contradict PRD-vN) |
| 8 | **[MASTER_PLAN.md](./MASTER_PLAN.md)** | Historical A–D scaffold + pointers |
| 9 | **Wave PRDs** `prd/PRD-wave-*.md` | Scaffold-era exit criteria (historical only) |
| 10 | Cold-start prompts | Session bootstrap — must name the **major PRD** they serve |

## Gate ledger

[GATES.md](../GATES.md) wins for **whether a runtime feature may land**.  
Subgates: **G6a** sessions(100), **G6b** skills(70), **G6c** MCP(80), **G6d** plan(110).

## Version display fields to update on every release

One release PR should touch (as applicable):

1. `Cargo.toml` workspace version  
2. `package.json` version (if present)  
3. Active **PRD-vN** release log (if minor notes needed)  
4. [CHANGELOG.md](../../CHANGELOG.md)  
5. [KNOWN_LIMITS.md](./KNOWN_LIMITS.md) if behavior honesty changes  
6. user-guide / README if they hardcode SemVer  

Run: `./scripts/check-semver.sh` and `npm run version-check` when npm exists.

## Major-line cheat sheet

| Line | PRD | Meaning |
|------|-----|---------|
| 1.x | [PRD-v1.md](./PRD-v1.md) | Scaffold / legacy |
| 2.x | [PRD-v2.md](./PRD-v2.md) | Grok base + DeepSeek shell (**current ship**) |
| 3.x | [PRD-v3.md](./PRD-v3.md) | Heart fusion L1+L2 (**next major**) |
| 4.x | [PRD-v4.md](./PRD-v4.md) | L3 productization (later) |
