# Bundled skills

Agent-loadable skills (`SKILL.md` directories) live here.

| Skill | When |
|-------|------|
| [`grok-sync/`](./grok-sync/SKILL.md) | Syncing the vendored Grok Build tree forward: pin vs upstream, adoption matrix, three-way merge, gates, ledger + changelist |
| [`pr-authoring/`](./pr-authoring/SKILL.md) | Writing or opening a PR body. Opening the PR is not the end of the unit; the close stays in `session-unit` |
| [`release/`](./release/SKILL.md) | The user asked to ship a version. A child brief cannot drop this skill. Then bump, CHANGELOG, tag, assets, npm |
| [`orca-tab/`](./orca-tab/SKILL.md) | Open an Orca tab or launch grok, dsb, codex, or claude. Run the recipes; do not read the full `orca` manual |
| [`session-unit/`](./session-unit/SKILL.md) | Default end of asked work: PR, CI, merge commit, report. Stop earlier only when that turn said so |
| [`worktree-dispatch/`](./worktree-dispatch/SKILL.md) | One worktree per unit, then merge and clean up. A brief cannot shrink the user's turn |

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
