# ADR 0011 — Agent harness entry points as committed links

- **Status:** Accepted
- **Date:** 2026-09-25
- **Normative companions:** [AGENTS.md](../../AGENTS.md) §Control-tower checkout · [skills/README.md](../../skills/README.md) · [REPO_LAYOUT.md](../architecture/REPO_LAYOUT.md)

## Context

The agent contract lives in `AGENTS.md` and the agent skills live in `skills/`
(`pr-authoring`, `release`, …). The coding agents that work on this repo do not
look in those places on their own:

| Agent | Reads instructions from | Discovers project skills in |
|-------|-------------------------|-----------------------------|
| Claude Code | `CLAUDE.md`; since `2.1.277`, `AGENTS.md` when the project has no `CLAUDE.md` (on by default, switchable in `/config`, not yet on Bedrock, Vertex or Foundry) | `.claude/skills/<name>/SKILL.md` |
| Codex CLI | `AGENTS.md` | `.agents/skills/<name>/SKILL.md` |
| DeepSeek Build (vendored Grok runtime) | `AGENTS.md`, `CLAUDE.md` (deduplicated by canonical path) | `.grok/skills`, `.agents/skills`, `.claude/skills` (deduplicated by canonical path) |

With none of `CLAUDE.md`, `.claude/`, `.agents/` in the tree, a session could
start without `pr-authoring` or `release` in its skill list, and a Claude Code
older than `2.1.277`, on a provider without the fallback, or with the fallback
switched off, without the contract itself. That weakens the repo's harness,
which by design is made of docs and skills, not process CI (ADR 0003).

It matters more now that the repo is worked on by many sessions in parallel,
each in its own Orca worktree: whatever wiring exists has to show up in every
new worktree without a setup step.

## Decision

Commit three symlinks at the repo root:

| Link | Target | For |
|------|--------|-----|
| `CLAUDE.md` | `AGENTS.md` | Claude Code instructions |
| `.claude/skills` | `../skills` | Claude Code skills (and DeepSeek Build via its `.claude` compat) |
| `.agents/skills` | `../skills` | Codex skills (and DeepSeek Build) |

`AGENTS.md` and `skills/` stay the only sources. The links are never edited;
a new skill directory under `skills/` is picked up by all three agents without
touching the links.

`.claude/` and `.agents/` are new top-level directories and track only these
links. Everything else under them is gitignored (`/.claude/*` with
`!/.claude/skills`, same for `.agents`): Claude Code writes per-user state
there — `settings.local.json`, `worktrees/`, `scheduled_tasks.json`,
`launch.json`, `agent-memory-local/` — and none of it belongs in the repo.

## Alternatives considered

| Option | Why not |
|--------|---------|
| Generate the links with a script, gitignore them | Every new worktree needs the script run first; a session opened before that runs without the harness. Committed links are there on checkout. |
| `CLAUDE.md` containing `@AGENTS.md` (import) instead of a link | Survives checkouts without symlink support, but DeepSeek Build has no import syntax and would load the literal `@AGENTS.md` line as a second instruction file; a link is deduplicated by canonical path. The skill directories need links anyway, so a checkout without symlinks loses the skills either way. |
| One link per skill (`.claude/skills/pr-authoring → ../../skills/pr-authoring`, …) | Two new links for every new skill, and a forgotten one silently hides a skill from one agent. The directory link has no per-skill upkeep. |
| Move skills into `.claude/skills/` and link the other way | `skills/` is the product seam in ADR 0001 and REPO_LAYOUT; tool-specific dot directories should point at it, not own it. |

## Consequences

- Claude Code, Codex and DeepSeek Build sessions opened anywhere in this repo
  (primary checkout or any worktree) load `AGENTS.md` and every skill under
  `skills/` with no setup.
- DeepSeek Build sessions run inside this repo now list the repo's skills in
  their skills index. That changes the stable prefix of those sessions only
  (dogfooding in this repo), not the product's defaults.
- **Regression for checkouts without symlink support** (typically Windows
  without Developer Mode and `core.symlinks=true`). Git writes each link as a
  small text file holding the target path. Such a checkout loses the repo
  skills (as before this ADR). Worse, Claude Code finds a `CLAUDE.md` that
  contains only the text `AGENTS.md`, and that file also suppresses the
  `2.1.277` `AGENTS.md` fallback. Remedy: enable symlinks and check out again.
  Nothing built or released depends on the links; releases target macOS
  (ADR 0009).
- A later need for a real `.claude/settings.json` or Codex config in the repo
  lives beside the links in the same directories; it needs its own review
  because it changes agent permissions, not just discovery.
