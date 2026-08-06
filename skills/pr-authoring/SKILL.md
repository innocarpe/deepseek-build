---
name: pr-authoring
description: >
  Write and open DeepSeek Build pull requests to the Orca-level narrative bar:
  Problem / What changed / Out of scope, evidence, honest Testing, AI review,
  Security audit, Notes. Use when creating a PR, drafting a PR body, shipping
  a branch, or when the user asks for PR conventions / reviewable units of work.
---

# PR authoring (DeepSeek Build harness)

This skill is the **agent harness** for change delivery. It is not CI.

## Load these docs (in order)

1. `docs/contributing/pr-body-standard.md` — narrative bar (Orca-aligned)
2. `docs/contributing/examples.md` — filled bodies by kind
3. `docs/contributing/pull-requests.md` — unit of work, title, labels, merge
4. `docs/contributing/review-checklist.md` — self-merge gate

## Hard rules

1. **Never push product work straight to `main`.** Branch → PR → squash-merge.
2. **One meaningful unit** per PR (one review lens). Prefer split over mega-PR.
3. **Title:** Conventional Commits  
   `feat|fix|docs|spec|chore|refactor|test|ci|perf|build(scope)?: summary`
4. **Exactly one kind label** matching the title type (`gh pr create --label …`).
5. **Body is the review artifact** (Orca density):
   - Summary: **Problem** + **What changed** + **Out of scope**
   - Screenshots / evidence (or “No visual change” + paths to read)
   - Testing: real commands or honest N/A + reason
   - AI review report (focus areas, flags, fixes)
   - Security audit (or justified “no sensitive surface”)
   - Notes (limits, follow-ups)
   - Kind / Related / Cache-impact / Checklist
6. **Do not** treat empty template checkboxes as done.
7. **Do not** invent “process CI” (title linters, path inventories) as a substitute for product tests.
8. Spec-before-large-feat for agent behavior; cite `docs/specs/…`.
9. Cache-impact honest for prompts / tools / skills / memory / routing.
10. After `gh pr create`, verify labels: `gh pr view --json title,labels,url`.
11. **SemVer only:** version mentions must be full `MAJOR.MINOR.PATCH` (e.g. `1.0.0`), never bare `1.0`. See `docs/contributing/versioning.md`.
12. **CLI names:** public docs prefer `deepseek-build`; `dsb` is the supported alias (ADR 0006).

## Optional local helper (not required)

```bash
./scripts/check-pr-title.sh "spec(cache): define stable prefix rules"
```

Title/label discipline is **process**, not a GitHub Actions gate.

## Workflow sketch

```bash
git fetch origin && git checkout main && git pull
git checkout -b <type>/<short-kebab>
# … work …
git push -u origin HEAD
gh pr create --base main \
  --title "<type>(scope): <summary>" \
  --label <kind> \
  --body-file <path-to-full-narrative-body>
gh pr view --json title,labels,url
```

## Anti-patterns

| Bad | Why |
|-----|-----|
| Summary = file list | Not reviewable |
| Testing all unchecked, no reasons | Unverifiable |
| “AI Review: LGTM” | Theater |
| Mixing M1 provider + M4 subagents | Wrong unit |
| Adding CI that only checks markdown paths / PR titles | Not product CI; rejects user intent for harness |

## Done means

- [ ] Narrative body meets `pr-body-standard.md`
- [ ] Kind label present and matches title
- [ ] Unit of work is coherent and revertable
- [ ] You would accept this PR from a stranger
