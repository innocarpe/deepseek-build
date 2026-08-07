# Single source of truth (priority order)

When documents disagree, **higher wins**. Agents must not invent a third story.

| Priority | Artifact | Owns |
|----------|----------|------|
| 1 | **Git `main` code + `Cargo.toml` version** | What actually ships (bytes) |
| 2 | **[OWNER_BAR_ACCEPTANCE.md](./OWNER_BAR_ACCEPTANCE.md)** | **Is the product done?** Owner-bar P0 checklist, Path A-only evidence, anti-game rules. Supersedes false “3.0/4.0 complete” claims for the true product. |
| 3 | **[versions/README.md](./versions/README.md)** + active major **PRD-vN** | **Which major line is in train** and line-scoped DoD (must not contradict owner bar for a “complete product” cut) |
| 4 | Active **PRD-vN** (e.g. planned **PRD-v5** for owner-bar major) | Line-specific problem, goals, honesty table |
| 5 | **[HARNESS_PHILOSOPHY.md](../architecture/HARNESS_PHILOSOPHY.md)** | L1/L2/L3 layer ownership (never silent override) |
| 6 | **[HEART_3X_SPEC_BINDING.md](../architecture/HEART_3X_SPEC_BINDING.md)** | Path A vs Path B; hearts only count on default agent path |
| 7 | **WAVE / ultragoal boards** | Fixed PR units for the **active** train (status ≠ product done) |
| 8 | **Ultragoal `goals.json` story status** (local `.omc/…`) | Next story *on this machine* (must not contradict owner bar / PRD-vN) |
| 9 | **[MASTER_PLAN.md](./MASTER_PLAN.md)** | Historical scaffold + pointers |
| 10 | Cold-start prompts | Session bootstrap — must name **owner bar + major PRD** they serve |

**Hard rule:** Ultragoal “all goals complete” or a SemVer tag **does not** mean owner-bar green. Only [OWNER_BAR_ACCEPTANCE.md](./OWNER_BAR_ACCEPTANCE.md) §8 / §12 does.

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
| 2.x | [PRD-v2.md](./PRD-v2.md) | Grok base + DeepSeek shell |
| 3.x | [PRD-v3.md](./PRD-v3.md) | Tagged heart-fusion *attempt* — **not owner-bar green** (see owner bar) |
| 4.x | [PRD-v4.md](./PRD-v4.md) | Tagged L3 productization *attempt* — **not owner-bar green** |
| **5.x** | [PRD-v5.md](./PRD-v5.md) | **Owner-bar complete product** (`owner-bar-5x`) — only when [OWNER_BAR_P0_LEDGER.md](./OWNER_BAR_P0_LEDGER.md) all PASS |
