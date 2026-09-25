# Bundled skills

Agent-loadable skills (`SKILL.md` directories) live here.

| Skill | When |
|-------|------|
| [`grok-sync/`](./grok-sync/SKILL.md) | Syncing the vendored Grok Build tree forward: pin vs upstream, adoption matrix, three-way merge, gates, ledger + changelist |
| [`pr-authoring/`](./pr-authoring/SKILL.md) | Opening or writing PRs; enforcing Orca-level narrative bar |
| [`release/`](./release/SKILL.md) | Cutting a release: bump, CHANGELOG, tag, prebuilt assets, npm publish |
| [`session-unit/`](./session-unit/SKILL.md) | Finish the opening unit (checks, commits, PR) and hand the next unit to a new Orca tab |
| [`worktree-dispatch/`](./worktree-dispatch/SKILL.md) | From the control-tower checkout: create a worktree per unit of work, hand it to an agent session, merge and clean up |

## How coding agents load these

`.claude/skills` and `.agents/skills` at the repo root are symlinks to this
directory ([ADR 0011](../docs/adr/0011-agent-harness-links.md)). Claude Code,
Codex and DeepSeek Build therefore discover every skill here in the primary
checkout and in every worktree, with no setup step. Add a skill by adding a
directory here; never edit the links.

## Future discovery (runtime)

When the agent runs as a product, discovery should also include Deep Code–compatible paths:

- Project: `.deepseek-build/skills/`, `.agents/skills/`
- User: `~/.deepseek-build/skills/`, `~/.agents/skills/`

Exact precedence → `docs/specs/70-…` (TODO).
