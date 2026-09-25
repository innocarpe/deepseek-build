# Commit conventions

We follow [Conventional Commits](https://www.conventionalcommits.org/).  
On `main`, history is **merge commits with their branch commits preserved** (see [pull-requests.md](./pull-requests.md) §9). Squash and rebase are disabled, so every commit you write on a branch lands in `main` — write them atomic.

---

## Format

```text
<type>(optional-scope): <summary>

[optional body]

[optional footer(s)]
```

### Types

| Type | Meaning | Typical kind label on the PR |
|------|---------|------------------------------|
| `feat` | New behavior | `feat` |
| `fix` | Bug fix | `fix` |
| `docs` | Docs / guides / research notes | `docs` |
| `spec` | Behavior contracts, PRD/ADR locking behavior | `spec` |
| `chore` | Housekeeping | `chore` |
| `refactor` | Same behavior, better structure | `refactor` |
| `test` | Tests only | `test` |
| `ci` | CI only | `ci` |
| `perf` | Performance only | usually `feat` or `chore` until dedicated label |
| `build` | Build system | `chore` |

### Summary rules

- Imperative: `add`, `fix`, `define` — not `added` / `adds`  
- No trailing period  
- ≤ ~72 characters  
- Scope optional but encouraged when area is clear: `cache`, `provider`, `tools`, `contributing`

### Body

Write a body when the **why** is not obvious from the summary. Good body content:

- Motivation (user pain, bug report, milestone exit criterion)  
- Approach in 2–5 sentences  
- Tradeoffs / alternatives rejected  
- Cache-impact when relevant  

### Footers

```text
Fixes #123
Closes #123
Refs #123
BREAKING CHANGE: description of break and migration
Cache-impact: low — sorted tool schema keys only
```

---

## Branch commits on `main`

`main` takes **merge commits** (squash and rebase are disabled on this repo),
so branch commits are preserved rather than flattened. That raises the bar on
them, it does not lower it.

| Location | Expectation |
|----------|-------------|
| Feature branch | **Atomic** Conventional Commits (one logical concern each). Compiles/tests when feasible. Ultragoal: see [ULTRAGOAL_PR_PLANNING.md](../product/ULTRAGOAL_PR_PLANNING.md) |
| `main` after merge | Every branch commit survives under the merge commit, so **each one must stand on its own as a changelog line** |

Do **not** rely on a merge commit to hide a PR that mixed three features — split
the PR instead.
Do **not** use the merge as an excuse for a single non-atomic dump commit on the
branch during multi-step work.

---

## Examples (good)

```text
spec(cache): define byte-stable system prefix rules

Stable: system, tools schema, skills index, standing memory.
Unstable/tail: user turn, dynamic reminders, volatile paths.

Cache-impact: high — defines the contract
```

```text
feat(provider): stream DeepSeek chat completions

OpenAI-compatible SSE client for deepseek-v4-flash/pro.
No tool loop yet — that is a follow-up PR against spec 40.

Refs docs/specs/20-model-routing.md
```

```text
fix(tools): preserve executable bit on shell-created files
```

```text
docs(contributing): deepen PR conventions with examples
```

```text
ci: require conventional PR titles
```

---

## Examples (bad)

```text
update
WIP
fix stuff
address review
asdf
implemented a lot of the agent
```

---

## Amending and force-push

| Action | On feature branch | On `main` |
|--------|-------------------|-----------|
| `commit --amend` + force-push | OK if you own the branch and no one else builds on it (or coordinate) | **Forbidden** |
| Rewrite shared history | Avoid for long-lived stacked branches | **Forbidden** |

Prefer new commits responding to review unless history is pure noise (typo-only) and the PR is still open.
