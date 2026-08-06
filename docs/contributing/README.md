# Contributing guides

Deep, **normative** process docs. Skim the root [CONTRIBUTING.md](../../CONTRIBUTING.md) first, then read these before opening a non-trivial PR.

| Guide | What it answers |
|-------|-----------------|
| [pull-requests.md](./pull-requests.md) | What is one unit of work? Titles, labels, body quality, merge, agent rules, anti-patterns |
| [pr-body-standard.md](./pr-body-standard.md) | **Orca-level narrative bar** (Summary/Testing/AI review/Security/Notes) |
| [examples.md](./examples.md) | Filled PR bodies for `spec` / `feat` / `fix` / `docs` (+ counterexamples) |
| [review-checklist.md](./review-checklist.md) | Author/reviewer/self-merge checklist |
| [commits.md](./commits.md) | Conventional Commits on branches vs squash on `main` |
| [branches.md](./branches.md) | Naming, lifecycle, stacking, protection expectations |
| [../maintainers/github-labels.md](../maintainers/github-labels.md) | Label catalog + sync |
| [../adr/0003-pr-process.md](../adr/0003-pr-process.md) | Why this process (alternatives rejected) |
| [../../skills/pr-authoring/SKILL.md](../../skills/pr-authoring/SKILL.md) | Agent skill (harness) for PR authoring |

## Enforcement

**Harness, not process CI:** agents load `pr-authoring`; humans use the checklist.  
Do not reintroduce GitHub Actions that only lint titles/labels/markdown inventories.

## Design sources for process (not product features)

Narrative density is inspired by product repos like **Orca** (detailed PR bodies).  
Commit/label shape draws from common Conventional Commits practice.

Product priorities remain: Grok Build / Reasonix / Deep Code (see [SOURCES](../product/SOURCES.md)).
