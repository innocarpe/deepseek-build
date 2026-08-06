# Pull request conventions

**Audience:** humans and coding agents.  
**Normative:** yes — process contract unless a later ADR supersedes it.  
**Companion docs:** [commits.md](./commits.md) · [branches.md](./branches.md) · [examples.md](./examples.md) · [review-checklist.md](./review-checklist.md)

This is not a generic “please open a PR” blurb. It is the **operating system** for how DeepSeek Build changes land: what counts as one unit of work, how titles/labels encode intent, what a mergeable description looks like, and how that interacts with specs, milestones, and DeepSeek cache discipline.

---

## 0. Why this process exists (for *this* repo)

DeepSeek Build is being built by **composing ideas** from Grok Build (speed/orchestration), Reasonix (cache/cost), and Deep Code (V4 surface)—not by dumping a monorepo rewrite in one shot.

That product strategy fails if the git process allows:

| Failure mode | What goes wrong here |
|--------------|----------------------|
| Mega-PRs (“M1 entire”) | Review becomes theater; agents hide incomplete specs behind code |
| Direct pushes to `main` | No CI gate on title/labels; history becomes “update files” |
| Spec-less features | Runtime drifts from PRD; Flash/Pro/cache rules get reinvented per PR |
| Ceremony without progress | Process docs that only restate Conventional Commits without project rules |

So the process optimizes for three outcomes:

1. **Reviewable slices** that match milestones (M1→M6).  
2. **Traceability**: PR → kind → area → spec/ADR → milestone.  
3. **Agent-proof defaults**: a coding agent can follow this file and produce an acceptable PR without asking “what does good look like?”

PR #1 established scaffolding + CI gates. **This document (and its companions) is the substance those gates protect.**

---

## 1. Design decisions (and what we rejected)

| Decision | Choice | Rejected alternatives | Why |
|----------|--------|----------------------|-----|
| Change vehicle | **PR into `main`** for all meaningful work | Direct push; long-lived `develop` | Solo + early stage: one trunk, reviewable units |
| History on `main` | **Squash merge**; PR title = commit subject | Merge commits always; rebase-merge only | `main` stays one-intent-per-commit; greppable |
| Title grammar | **Conventional Commits** | Free-form; Angular-only without `spec` | Need a first-class `spec` type for docs-first work |
| Kind encoding | **Exactly one kind label** on ready PRs | Labels optional; many kind labels | CI + human skim; forbids unlabeled “done” |
| Size | Soft S/M/L guidance | Hard line caps as merge blockers | Docs/spec PRs legitimately larger in prose |
| Review | Self-merge OK early when checklist met | Require 2 reviewers day one | Single maintainer; don’t fake process |
| Spec before code | Implementation PRs **link a ready-enough spec** | Code-first, document later | Product is cache/routing sensitive; silent invention is costly |
| Issue required? | **Not always** | Issue for every keystroke | Specs/docs can be the artifact; use issues for bugs/discussion/tracking |

Full ADR: [0003-pr-process.md](../adr/0003-pr-process.md) (expanded in the depth pass).

---

## 2. Taxonomy: what kind of PR is this?

Pick **one** kind before writing code. Kind drives title type, label, and body expectations.

| Kind | Title type | When to use | Must include | Must not include |
|------|------------|-------------|--------------|------------------|
| **spec** | `spec` | Behavior contract, PRD delta that locks behavior, ADR that freezes a product rule | Acceptance criteria, non-goals, how we’ll test later | Large runtime implementation |
| **docs** | `docs` | Guides, README, process, research notes that are **not** shipping contracts | Who the doc is for; what’s normative vs advisory | Quietly changing shipped behavior without `spec`/`feat` |
| **feat** | `feat` | User-visible or agent-visible behavior | Link to spec/ADR; test plan that exercises the behavior | Unrelated refactors; drive-by format |
| **fix** | `fix` | Correctness regression | Repro / expected / actual (or link issue) | “While here” features |
| **refactor** | `refactor` | Structure change, same behavior | Why safer/faster; how you checked behavior unchanged | Intentional behavior change (that’s `feat`/`fix`) |
| **test** | `test` | Tests only | What risk the tests lock | Production code changes beyond test hooks |
| **ci** | `ci` | Actions/workflows only | What gate changed | App logic |
| **chore** | `chore` | Labels, ignore files, deps, mechanical housekeeping | Why now | Mixed feature work |

### Mapping to DeepSeek Build milestones (examples)

| Milestone | Typical PR kinds | Example units (each = its own PR) |
|-----------|------------------|-----------------------------------|
| M1 | `spec`, then `feat`/`chore` | `spec(cache): …` → `spec(routing): …` → `feat(provider): stream chat` |
| M2 | `spec`, `feat` | `spec(tools): …` → `feat(tools): parallel dispatch` |
| M3 | `spec`, `feat` | `spec(skills): discovery paths` → `feat(skills): load SKILL.md` |
| M4 | `spec`, `feat` | `spec(subagents): cache rules for workers` → `feat(orchestrator): spawn explore` |
| M5 | `spec`, `feat` | `spec(sessions): resume/fork` → `feat(mcp): list tools` |
| M6 | `docs`, `chore`, `ci` | user-guide pages, release notes, install script |

**Rule of thumb:** if M1 has no `spec` PRs merged for cache/routing/thinking, do not open a giant `feat(provider)` that invents those rules in code comments.

---

## 3. One meaningful unit (definition)

A PR is **one unit** when all of the following hold:

1. **One sentence outcome** — you can finish: “After merge, ___.”  
2. **One primary kind** — not “half spec, half production loop.”  
3. **One review lens** — a reviewer can ask one main question (“Is this contract right?” *or* “Does this implement the contract?”).  
4. **Independently mergeable** — does not require an unmerged sibling branch to compile/make sense (stacked PRs are OK if each step is reviewable).  
5. **Revertable** — `git revert` of the squash commit should not leave the tree half-migrated without a follow-up plan called out in the body.

### Decision tree: split or keep?

```text
Does the diff change both a behavior contract AND production code?
  YES → split: spec PR first (or same milestone, two PRs), then feat PR
  NO ↓
Does the diff touch two milestones’ exit criteria?
  YES → split by milestone
  NO ↓
Can a reviewer understand it without reading an external essay?
  NO → either shrink scope or improve the PR body (examples!)
  YES ↓
Is >~600 LOC of non-generated code or >~12 code files?
  YES → split unless mechanical rename (call out in body)
  NO → single PR is fine
```

### Concrete good / bad units for this repo

| Good | Bad |
|------|-----|
| Draft `docs/specs/10-cache-contract.md` only | Cache spec + Flash/Pro routing + thinking + half a Rust crate |
| Implement prefix builder **against** merged spec 10 | “Start crates/” with empty modules and no behavior |
| Fix `pr-title` regex false negative | Fix CI + reword entire PRD + add skills |
| Add `/model` UX for effort flags | `/model` + MCP + permissions in one PR |

See [examples.md](./examples.md) for full filled PR bodies.

---

## 4. Size guidance (soft)

| Label | Rough bound | When it’s OK |
|-------|-------------|--------------|
| `size/S` | ≤ ~200 LOC net; docs/spec prose ≤ ~400 lines changed | Default target |
| `size/M` | ~200–600 LOC; larger docs OK if single topic | One subsystem |
| `size/L` | above that, or many files | Mechanical only, or explicitly justified |

Size labels are **signals**, not CI hard-fails. Prefer splitting an `L` feature PR over inventing a novel process exception.

---

## 5. Branch, title, labels

### Branch

See [branches.md](./branches.md). Pattern: `<type>/<short-kebab>`  
Examples: `spec/10-cache-contract`, `feat/provider-stream`, `docs/pr-conventions-depth`.

### Title (required; CI-enforced)

```text
<type>(optional-scope): <imperative summary>
```

Rules:

1. Type ∈ `feat|fix|docs|spec|chore|refactor|test|ci|perf|build`  
2. Optional scope: kebab-case product area (`cache`, `provider`, `contributing`, `tools`)  
3. Imperative summary, no trailing period, ≤ ~72 chars  
4. Breaking: `feat(provider)!: …` + `BREAKING CHANGE:` in body  
5. **Title type must match kind label**

Local check:

```bash
./scripts/check-pr-title.sh "spec(cache): define stable system prefix rules"
```

### Labels (required kind; CI on non-draft)

| Required | Exactly one of | `feat` `fix` `docs` `spec` `chore` `refactor` `test` `ci` |
|----------|----------------|--------------------------------------------------------------|
| Recommended | Area | `area/cache` `area/provider` `area/docs` `area/infra` … |
| Optional | Size / process | `size/S` `size/M` `size/L` `milestone-aligned` `needs-design` `ready` |

Catalog: [`.github/labels.json`](../../.github/labels.json).

**Draft PRs** may omit kind temporarily; converting to **Ready** without kind fails CI (`pr-kind-label`).

---

## 6. PR body quality bar

The template (`.github/PULL_REQUEST_TEMPLATE.md`) is the skeleton. **Passing the skeleton with empty bullets is not enough.**

### Minimum substance by kind

| Kind | Summary must answer | Test plan must include |
|------|---------------------|------------------------|
| `spec` | What behavior is locked? What is explicitly out of scope? | “Reviewer reads §X and checks consistency with PRD/SOURCES” |
| `docs` | Who is the audience? Normative or advisory? | Link walkthrough; broken-link check if applicable |
| `feat` | User/agent-visible change; which spec paragraph | Command(s) or scenario that fails before / passes after |
| `fix` | Root cause hypothesis; blast radius | Repro steps; regression test if feasible |
| `ci` | What policy is now enforced or relaxed | Example title/label that would fail/pass |
| `chore` | Why this housekeeping unblocks others | Sync/script dry-run output |

### Always fill

1. **Summary** — what *and why* (not only file list).  
2. **Related** — milestone, spec/ADR paths, `Closes`/`Refs`.  
3. **Test plan** — falsifiable. “Looks good” is not a plan.  
4. **Cache-impact** — for anything that can touch prompts, tool schemas, skills index, system memory:

```text
Cache-impact: none | low | medium | high — <reason>
```

If you are unsure, pick **medium** and explain—not `none` by default on agent work.

### Product alignment gates

- Does not violate [NON_GOALS](../product/NON_GOALS.md) without an ADR.  
- Does not smuggle Gajae-style multi-stage planning into “helpful extras.”  
- Implementation claims “done” only when the **spec’s acceptance criteria** are hit (or the PR explicitly narrows them).

---

## 7. End-to-end workflow (maintainer / agent)

```bash
# 0) start from current main
git fetch origin
git checkout main && git pull origin main

# 1) branch
git checkout -b spec/10-cache-contract

# 2) work; commit with conventional messages (see commits.md)
git add -A
git commit -m "spec(cache): draft stable prefix contract"

# 3) push + PR
git push -u origin HEAD
gh pr create \
  --base main \
  --title "spec(cache): draft stable prefix contract" \
  --label spec \
  --label area/cache \
  --label size/M \
  --milestone "M1 — Provider + cache + routing" \
  --body-file - <<'EOF'
## Summary
- …

## Kind
- [x] `spec`

## Related
- **Milestone:** M1
- **Spec / ADR:** docs/specs/10-cache-contract.md
- **Issues:** n/a

## Test plan
- [ ] …

## Cache impact
high — defines the contract

## Checklist
- [x] …
EOF

# 4) verify gates
gh pr view --json title,labels,milestone,url
gh pr checks

# 5) after approval / solo checklist: squash merge (repo default)
gh pr merge --squash
git checkout main && git pull origin main
git branch -d spec/10-cache-contract
```

### Stacked work

Prefer:

```text
main ← PR-A (spec) ← merge
main ← PR-B (feat implementing A)
```

If you must stack before A merges:

```text
main ← PR-A
       ← PR-B (base = A’s branch)  # retarget base to main after A merges
```

Never leave PR-B targeting a deleted branch without retarget.

---

## 8. Review

Deep checklist: [review-checklist.md](./review-checklist.md).

### Author (before Ready)

1. Read your own diff in the GitHub UI (not only local IDE).  
2. Ensure PR does not contain secrets, machine-local junk, or unrelated reformats.  
3. Ensure title/label/milestone/body match.  
4. Ensure CI is green or failures explained in the body.

### Reviewer

1. **Intent** — is this the right unit?  
2. **Contract** — specs/PRD/SOURCES alignment.  
3. **Correctness** — for code: edge cases, permissions, cache prefix stability.  
4. **Scope creep** — request follow-up PR rather than expanding this one.  
5. Approve only if kind label + CI + body bar are met.

### Solo maintainer self-merge

Allowed when:

1. CI green (`docs-hygiene`, and on PRs `pr-title` + `pr-kind-label`)  
2. Exactly one kind label  
3. Body meets the quality bar for that kind (not checkbox theater)  
4. You would accept this PR from a stranger without embarrassment  

No mandatory 24h wait in early milestones; still sleep on **high cache-impact** or security-sensitive changes if unsure.

---

## 9. Merge policy

| Setting | Value |
|---------|--------|
| Allowed method on GitHub | **Squash merge only** |
| Squash title | PR title |
| Squash body | PR body (trimmed as needed) |
| Delete branch on merge | yes |

After squash, `main` history should read like a product changelog of intents:

```text
spec(cache): …
feat(provider): …
fix(tools): …
docs(contributing): …
```

not:

```text
WIP
address comments
fix stuff
merge branch 'x'
```

---

## 10. Issues vs PRs

| Situation | Prefer |
|-----------|--------|
| Bug with repro | Issue (`bug`) → then `fix` PR `Closes #N` |
| Open design question | Issue (`needs-design`) or draft `spec` PR |
| Spec ready to lock | `spec` PR (issue optional) |
| Small docs fix | `docs` PR directly |
| Tracking multi-PR epic | Milestone + optional tracking issue |

---

## 11. Agent / automation hard requirements

Coding agents **must not** mark work complete unless:

1. Branch ≠ `main`  
2. `gh pr create` (or update) with conventional title  
3. `--label` includes exactly one kind  
4. Body has non-empty Summary + Test plan appropriate to kind  
5. `gh pr view --json labels` shows the kind label  
6. CI not failing required jobs  

Agents **must not**:

- Open a “foundation” PR that only restates generic OSS advice without project-specific rules (the failure mode of a thin first process PR)  
- Mix M1 provider work with M4 subagents “because it’s related to agents”  
- Claim cache-impact `none` on prompt/tool schema edits  

---

## 12. Anti-patterns (project-specific)

| Anti-pattern | Why it hurts DeepSeek Build |
|--------------|-----------------------------|
| “Implement agent” mega-PR | Hides missing cache/routing specs; unreviewable |
| Spec that only says “should be fast” | Not testable; won’t guide Flash/Pro or prefix rules |
| Feat without linking spec | Diverges from Reasonix/Deep Code source priorities |
| Rewriting CONTRIBUTING in every feature PR | Process thrash; put process changes in `docs` PRs |
| Force-pushing `main` | Breaks the only shared history |
| Using `chore` for user-visible behavior | Breaks release/changelog intent |
| Label spam (3 kind labels) | CI fails; intent unclear |

---

## 13. CI jobs that encode this doc

| Job | When | Enforces |
|-----|------|----------|
| `docs-hygiene` | push/PR | Required paths exist; labels.json valid |
| `pr-title` | PR | Conventional title regex |
| `pr-kind-label` | non-draft PR | Exactly one kind label |

CI is the **floor**, not the ceiling. A green PR can still be rejected for empty Summary or wrong unit of work.

---

## Related

- [examples.md](./examples.md) — filled PR bodies  
- [review-checklist.md](./review-checklist.md)  
- [commits.md](./commits.md) · [branches.md](./branches.md)  
- [MILESTONES.md](../product/MILESTONES.md) · [PRD-v1.md](../product/PRD-v1.md)  
- Root [CONTRIBUTING.md](../../CONTRIBUTING.md)
