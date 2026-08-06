# Bundled skills

Agent-loadable skills (`SKILL.md` directories) live here.

| Skill | When |
|-------|------|
| [`pr-authoring/`](./pr-authoring/SKILL.md) | Opening or writing PRs; enforcing Orca-level narrative bar |

## Future discovery (runtime)

When the agent runs as a product, discovery should also include Deep Code–compatible paths:

- Project: `.deepseek-build/skills/`, `.agents/skills/`
- User: `~/.deepseek-build/skills/`, `~/.agents/skills/`

Exact precedence → `docs/specs/70-…` (TODO).

Until then, **coding agents working on this repo** must still honor `pr-authoring` via `AGENTS.md`.
