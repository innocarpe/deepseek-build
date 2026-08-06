# Commit conventions

We follow [Conventional Commits](https://www.conventionalcommits.org/).  
On `main`, history is primarily **one squash commit per PR** (see [pull-requests.md](./pull-requests.md) §Merge). Branch commits still matter for review and bisect *before* merge.

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

## Branch commits vs squash on `main`

| Location | Expectation |
|----------|-------------|
| Feature branch | Prefer conventional commits; small WIP commits OK if PR title is solid and final squash is clean |
| `main` after squash | **PR title** becomes the subject; should stand alone as a changelog line |

Do **not** rely on squash to hide a PR that mixed three features—split the PR instead.

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
