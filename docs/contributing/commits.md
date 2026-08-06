# Commit conventions

We follow [Conventional Commits](https://www.conventionalcommits.org/) so history stays greppable and squash-merge titles stay consistent with PR titles.

## Format

```text
<type>(optional-scope): <summary>

[optional body]

[optional footer(s)]
```

### Type

Same set as PR titles: `feat` `fix` `docs` `spec` `chore` `refactor` `test` `ci` `perf` `build`.

### Summary

- Imperative, present tense: `add` not `added`  
- No trailing period  
- ~72 characters or less  

### Body

Use when the *why* is non-obvious. Wrap at ~72–100 chars. Bullets are fine.

### Footers

```text
Fixes #123
Closes #123
Refs #123
BREAKING CHANGE: description
Cache-impact: low — tool schema field order stabilized
```

## Local history vs squash

Default merge is **squash**. That means:

- Intermediate commits on a branch may be informal **only if** the branch is short-lived and the **PR title** is conventional.  
- Prefer still writing conventional commits on the branch — they become the PR discussion trail and help `git bisect` before merge.  
- After squash, `main` history is essentially **one conventional commit per PR**.

## Examples

```text
spec(cache): define byte-stable system prefix rules

Lock tools schema and skills index as cache-stable.
Dynamic reminders move to the turn tail.

Cache-impact: high — defines the contract
```

```text
fix(tools): avoid truncating shell output mid-line
```

```text
chore: add size labels to catalog
```

## What to avoid

- `WIP`, `tmp`, `asdf`, `update files`  
- Mixing unrelated concerns in one commit when you still plan multi-commit review  
- Rewriting `main` history  
