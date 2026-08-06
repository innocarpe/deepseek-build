# DeepSeek Build

**DeepSeek-native terminal coding agent.**

Combines three first-class references:

| Source | What we take |
|--------|----------------|
| **Grok Build** | Wall-clock speed: parallel tools, subagents, background tasks, worktree isolation, native runtime patterns |
| **[Reasonix](https://github.com/esengine/DeepSeek-Reasonix)** | Prefix-cache-first loop, Flash/Pro cost control, tool-call repair |
| **[Deep Code CLI](https://github.com/lessweb/deepcode-cli)** | Official DeepSeek-oriented surface: thinking, reasoning effort, Skills, MCP, permissions, plan mode |

**Not in v1 scope:** Gajae-code multi-stage planning/team harness (too slow for our north star).

> Status: **scaffolding + product docs.** Runtime stack is reserved (see `docs/architecture/REPO_LAYOUT.md`) but not locked in code yet.

## Product docs

| Doc | Description |
|-----|-------------|
| [PRD v1](docs/product/PRD-v1.md) | Problem, goals, scope, journeys, success |
| [Milestones](docs/product/MILESTONES.md) | M0–M6 development plan (GitHub Milestones) |
| [Vision](docs/product/VISION.md) | North star and pillars |
| [Sources](docs/product/SOURCES.md) | Grok / Reasonix / Deep Code priorities |
| [Non-goals](docs/product/NON_GOALS.md) | Explicit v1 exclusions |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). **All meaningful work lands via PR.**

| Doc | Topic |
|-----|--------|
| [PR conventions](docs/contributing/pull-requests.md) | Units of work, titles, labels, merge |
| [PR body standard](docs/contributing/pr-body-standard.md) | Orca-level narrative bar |
| [PR examples](docs/contributing/examples.md) | Filled bodies for spec/feat/fix/docs |
| [Review checklist](docs/contributing/review-checklist.md) | Self-merge / reviewer gates |
| [Commits](docs/contributing/commits.md) | Conventional Commits |
| [Branches](docs/contributing/branches.md) | Branch naming |
| [Labels](docs/maintainers/github-labels.md) | Kind / area / size catalog |

PR title must be Conventional Commits; ready PRs need exactly one kind label (CI enforced).

## Repository map

```text
deepseek-build/
├── docs/                 # Product truth lives here first
│   ├── product/          # Vision, positioning, sources, non-goals
│   ├── specs/            # Behavior contracts (what ships)
│   ├── architecture/     # System + repo structure
│   ├── adr/              # One decision per file (append-only)
│   ├── research/         # Notes on Grok / Reasonix / Deep Code
│   └── user-guide/       # End-user docs (later; Grok-style numbered guides)
├── crates/               # Future implementation packages (Grok-like layout)
├── skills/               # Bundled Agent Skills (SKILL.md)
├── .deepseek-build/      # Project-local agent config surface (committed examples only)
├── scripts/              # Dev / release helpers
└── third_party/          # Vendored / ported code notices
```

Start reading: **[docs/README.md](docs/README.md)**.

## References

- Grok Build — primary structural and orchestration reference (local sibling `OpenSources/grok-build` when developing offline)
- [Reasonix](https://github.com/esengine/DeepSeek-Reasonix) — DeepSeek cache-native agent
- [Deep Code CLI](https://github.com/lessweb/deepcode-cli) — official DeepSeek-oriented CLI surface

## License

[Apache License 2.0](LICENSE). See also [NOTICE](NOTICE).
