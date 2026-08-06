# Contributing to DeepSeek Build

Thanks for your interest. This project is early: **docs and specs land before code**, and **all meaningful work lands via pull requests**.

## Quick start

1. Read [docs/README.md](docs/README.md), [docs/product/PRD-v1.md](docs/product/PRD-v1.md), and [docs/product/NON_GOALS.md](docs/product/NON_GOALS.md).  
2. Pick a [milestone](https://github.com/innocarpe/deepseek-build/milestones) (see [docs/product/MILESTONES.md](docs/product/MILESTONES.md)).  
3. For behavior changes, ensure a [spec](docs/specs/) exists or open a `spec` PR first.  
4. Branch → PR → CI → squash-merge.  

**Process guides (normative):**

| Guide | Topic |
|-------|--------|
| [docs/contributing/pull-requests.md](docs/contributing/pull-requests.md) | PR size, titles, labels, review, merge |
| [docs/contributing/commits.md](docs/contributing/commits.md) | Conventional Commits |
| [docs/contributing/branches.md](docs/contributing/branches.md) | Branch naming |
| [docs/maintainers/github-labels.md](docs/maintainers/github-labels.md) | Labels |

## Development priorities (do not fight these)

1. **Grok Build** — wall-clock speed, parallel orchestration patterns  
2. **Reasonix** — DeepSeek cache-first cost loop  
3. **Deep Code CLI** — official DeepSeek-oriented CLI surface  

Gajae-code multi-stage planning harnesses are **out of v1 scope**.

## Issues

| Kind | How |
|------|-----|
| Bug | [Bug report](https://github.com/innocarpe/deepseek-build/issues/new?template=bug_report.yml) |
| Feature | [Feature request](https://github.com/innocarpe/deepseek-build/issues/new?template=feature_request.yml) |
| Spec / design | [Spec work](https://github.com/innocarpe/deepseek-build/issues/new?template=spec.yml) |

Set a **milestone** when the work is planned. Use labels for triage (`bug`, `enhancement`, `needs-design`, `area/*`, …).

## Pull requests (summary)

Full rules: **[docs/contributing/pull-requests.md](docs/contributing/pull-requests.md)**.

1. **One meaningful unit** per PR (prefer small vertical slices).  
2. Branch name: `<type>/<short-kebab>` (e.g. `docs/pr-conventions`).  
3. Title: Conventional Commit — `feat|fix|docs|spec|chore|refactor|test|ci(scope)?: summary`.  
4. **Exactly one kind label** matching the title type.  
5. Fill the PR template (summary, related, test plan).  
6. Set milestone when known; link issues with `Closes #N` / `Refs #N`.  
7. Default merge: **squash and merge** (PR title becomes the commit on `main`).  
8. No secrets, API keys, or raw private session logs.

### Docs-only and spec PRs

Welcome. Spec drafts are product work (`spec` kind). Process/docs use `docs` or `chore`.

## Code of conduct

See [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).

## Security

See [SECURITY.md](SECURITY.md). Do not file public issues for sensitive vulnerabilities.

## License

By contributing, you agree that your contributions are licensed under the project’s [Apache License 2.0](LICENSE).
