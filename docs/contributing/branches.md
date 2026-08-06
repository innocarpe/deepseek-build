# Branch conventions

---

## Default branch

`main` is the single integration branch.

| Phase | What “healthy main” means |
|-------|---------------------------|
| Now (docs-first) | Docs/specs consistent; process harness followed; no secrets |
| Later (runtime) | Buildable agent for the claimed milestone; no known broken defaults |

There is **no long-lived `develop`** unless a future ADR introduces one.

---

## Naming

```text
<type>/<short-kebab-description>
```

| Prefix | Use |
|--------|-----|
| `feat/` | User/agent-visible behavior |
| `fix/` | Bugfix |
| `docs/` | Documentation and process |
| `spec/` | Specs, PRD/ADR behavior locks |
| `chore/` | Tooling, labels, deps |
| `ci/` | Workflows |
| `refactor/` | Refactors |
| `test/` | Tests |

Optional issue number:

```text
fix/42-shell-timeout
spec/10-cache-contract
feat/17-provider-stream
```

### Good names

```text
spec/10-cache-contract
feat/provider-stable-prefix
docs/pr-conventions-depth
fix/check-pr-title-scope-digits
```

### Bad names

```text
update
patch-1
innocarpe-patch-2
wip
agent-stuff
final-final-v2
```

---

## Lifecycle

```text
main ──checkout -b──► branch ──push──► PR ──CI──► review ──squash merge──► delete branch
         ▲                                         │
         └──────── pull --rebase / merge main ─────┘  (keep branch current)
```

### Commands

```bash
git fetch origin
git checkout main && git pull origin main
git checkout -b docs/my-topic
# … work …
git push -u origin HEAD
gh pr create …   # see pull-requests.md
# after merge:
git checkout main && git pull origin main
git branch -d docs/my-topic
git push origin --delete docs/my-topic   # if not auto-deleted
```

---

## Keeping a branch current

Prefer:

```bash
git fetch origin
git rebase origin/main
# or: git merge origin/main
```

Resolve conflicts **on the branch**. Never “fix history” by force-pushing `main`.

If a PR is stacked on another PR’s branch, retarget base to `main` after the parent merges.

---

## Protection expectations

Even when GitHub branch protection is light (solo early phase):

| Rule | Level |
|------|--------|
| No force-push to `main` | Hard |
| No multi-file direct commits to `main` | Hard (process) |
| PR required for meaningful work | Hard (process + culture) |
| Required status checks | Soft now / tighten later |
| Required human review count | Not required early |

---

## Remote

```text
origin → nina.v@example.com:innocarpe/deepseek-build.git
```

External contributors: fork → PR against `innocarpe/deepseek-build` `main`.
