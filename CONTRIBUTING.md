# Contributing to DeepSeek Build

Thanks for your interest. This project is early: **docs and specs land before code**.

## Before you write code

1. Read [docs/README.md](docs/README.md), [docs/product/VISION.md](docs/product/VISION.md), and [docs/product/NON_GOALS.md](docs/product/NON_GOALS.md).
2. Check [docs/specs/](docs/specs/) for a behavior contract. If none exists, open an issue labeled `spec` (or a draft PR that only adds a spec).
3. Prefer an existing [GitHub Milestone](https://github.com/innocarpe/deepseek-build/milestones) over unscoped work.

## Development priorities (do not fight these)

1. **Grok Build** — wall-clock speed, parallel orchestration patterns  
2. **Reasonix** — DeepSeek cache-first cost loop  
3. **Deep Code CLI** — official DeepSeek-oriented CLI surface  

Gajae-code multi-stage planning harnesses are **out of v1 scope**.

## How to contribute

### Issues

- **Bug:** use the Bug report template  
- **Feature / behavior change:** Feature request template; link the relevant spec if any  
- **Spec work:** label `spec`  
- **Docs only:** label `docs`

### Pull requests

1. Fork (or branch on a collaborator clone).  
2. Keep PRs small and milestone-aligned.  
3. Use a Conventional Commit style title: `feat:`, `fix:`, `docs:`, `spec:`, `chore:`, `ci:`, `test:`, `refactor:`.  
4. Apply **exactly one primary kind label** (`feat` / `fix` / `docs` / `spec` / `chore` / `refactor` / `test` / `ci`).  
5. Fill the PR template.  
6. Do not commit secrets, API keys, or real session logs.

### Docs-only PRs

Welcome and preferred while the runtime is unbuilt. Spec drafts count as product work.

## Code of conduct

See [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).

## Security

See [SECURITY.md](SECURITY.md). Do not file public issues for vulnerabilities that could harm users if disclosed early.

## License

By contributing, you agree that your contributions are licensed under the project’s [Apache License 2.0](LICENSE).
