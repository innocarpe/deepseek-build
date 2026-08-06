# Branch conventions

## Default branch

`main` — always releasable enough for the project’s current phase (docs-only today; later: buildable agent).

## Naming

```text
<type>/<short-kebab-description>
```

| Prefix | Typical use |
|--------|-------------|
| `feat/` | User-visible work |
| `fix/` | Bugfix |
| `docs/` | Documentation / process |
| `spec/` | Specs, PRD, ADR |
| `chore/` | Tooling, labels, deps |
| `ci/` | Workflows |
| `refactor/` | Refactors |
| `test/` | Tests |

Optional: issue number for traceability:

```text
fix/123-shell-timeout
spec/10-cache-contract
```

### Avoid

- `update`, `stuff`, `innocarpe-patch-1` as the only name  
- Long sentences in branch names  
- Reusing a merged branch name for unrelated work (prefer a new name)

## Lifecycle

```text
main ──► branch ──► PR ──► CI ──► review ──► squash merge ──► delete branch
```

1. `git fetch origin && git checkout main && git pull`  
2. `git checkout -b docs/my-topic`  
3. Commit; push; open PR  
4. After merge: update local `main`, delete local branch  

## Protected expectations (process)

Even if GitHub branch protection is relaxed for early solo work:

- Do not force-push `main`  
- Do not commit directly to `main` for multi-file work  
- Rebase or merge `main` into your branch to resolve conflicts **on the branch**, not by rewriting `main`  

## Remote

```text
origin → nina.v@example.com:innocarpe/deepseek-build.git
```

Forks: open PRs against `innocarpe/deepseek-build` `main`.
