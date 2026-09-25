# Bundled skills

Agent-loadable skills (`SKILL.md` directories) live here.

| Skill | When |
|-------|------|
| [`pr-authoring/`](./pr-authoring/SKILL.md) | Opening or writing PRs; enforcing Orca-level narrative bar |
| [`release/`](./release/SKILL.md) | Cutting a release: bump, CHANGELOG, tag, prebuilt assets, npm publish |

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
