# Pull request conventions

**Audience:** anyone opening or reviewing a PR (humans and coding agents).  
**Normative:** yes — treat this as the process contract unless an ADR supersedes it.

Inspired by patterns used across mature OSS (Conventional Commits, small vertical slices, squash-merge, labeled PRs) and tuned for a docs-first, milestone-driven project.

---

## 1. Why PRs (even for solo work)

- History stays reviewable (`git log` / GitHub UI).
- CI runs on the change set, not only on `main`.
- Specs, product, and code stay aligned per unit of work.
- Future collaborators inherit the same shape.

Direct pushes to `main` are reserved for emergencies (broken `main`, secret leak hotfix). Prefer a PR even then when possible.

---

## 2. One meaningful unit per PR

A PR should deliver **one coherent outcome** that can be described in a single sentence.

| Good unit | Bad unit |
|-----------|----------|
| “Add cache-contract spec (10)” | “Write all M1 specs + scaffold Rust + rename repo” |
| “Wire Flash/Pro routing flags” | “Routing + subagents + MCP” |
| “Fix CI path check for NOTICE” | Unrelated chore + feature + reformat entire tree |

### Size guidance (soft)

| Size | Rough bound | Notes |
|------|-------------|--------|
| **S** | ≤ ~200 lines net, or docs-only ≤ ~400 | Preferred default |
| **M** | ~200–600 lines | OK if single concern |
| **L** | > ~600 lines or > ~12 files of code | Split unless mechanical rename/generated |

Optional labels: `size/S`, `size/M`, `size/L` (see label catalog).

If a change is large because it is **purely mechanical** (rename, format, license headers), say so in the PR body and keep it pure.

### Vertical slice > horizontal layer dump

Prefer end-to-end thin slices:

```text
✅ spec 10 draft → (later PR) provider prefix builder → (later) tests
❌ every file under crates/ “started” with no shippable behavior
```

---

## 3. PR title (required)

Use **[Conventional Commits](https://www.conventionalcommits.org/)** form (same as commit style when squash-merging):

```text
<type>(optional-scope): <imperative summary>
```

### Allowed types (match kind labels)

| Type | Use for |
|------|---------|
| `feat` | User-visible behavior |
| `fix` | Bug fix |
| `docs` | Documentation only (including product docs that are not a behavior **spec**) |
| `spec` | Behavior contracts under `docs/specs/`, or PRD/ADR that lock behavior |
| `chore` | Tooling, ignore files, deps, housekeeping |
| `refactor` | No intentional behavior change |
| `test` | Tests only |
| `ci` | CI / GitHub Actions only |
| `perf` | Performance-only (rare early on) |
| `build` | Build system only |

### Rules

1. **Lowercase type**; no trailing period on the summary.
2. Summary ≤ ~72 characters; imperative mood (`add`, `fix`, `document` — not `added` / `adds`).
3. Scope is optional kebab-case: `feat(provider):`, `docs(contributing):`, `spec(cache):`.
4. Breaking change: `feat!:` or `feat(api)!:` plus a footer in the body (`BREAKING CHANGE: …`).
5. **Title must match the primary kind label** (see below).

### Examples

```text
docs(contributing): establish pull request conventions
spec(cache): draft stable prefix contract
feat(provider): stream DeepSeek chat completions
fix(tools): preserve executable bit on write
chore: sync GitHub labels catalog
ci: validate conventional PR titles
```

CI enforces a basic title regex on `pull_request` events (see `.github/workflows/ci.yml`).

---

## 4. Labels (required)

### Kind (exactly one)

`feat` · `fix` · `docs` · `spec` · `chore` · `refactor` · `test` · `ci`

(`perf` / `build` titles map to kind `chore` or `feat` until dedicated labels exist.)

### Recommended secondary

| Label | When |
|-------|------|
| `area/*` | Touches a known product area |
| `milestone-aligned` | Milestone field set |
| `needs-design` | Should not merge until ADR/spec exists |
| `ready` | Maintainer marks implementable / mergeable |
| `size/S` `size/M` `size/L` | Optional size signal |
| `priority/p*` | Maintainer triage |

**Unlabeled PRs are incomplete.** Do not merge without a kind label.

Catalog: [`.github/labels.json`](../../.github/labels.json) · [github-labels.md](../maintainers/github-labels.md)

---

## 5. Branch naming

See [branches.md](./branches.md). Short form:

```text
<type>/<short-kebab>
```

Examples: `docs/pr-conventions`, `spec/cache-contract`, `feat/provider-stream`.

---

## 6. PR body (template)

Use the GitHub template. Minimum quality bar:

1. **Summary** — what and why (2–8 bullets or short prose).  
2. **Related** — milestone, spec/ADR paths, `Closes #N` / `Refs #N`.  
3. **Test plan** — how a reviewer knows it is correct (commands, doc checklist).  
4. **Risks / rollout** — if any (defaults, migrations, cache impact).  

### Product alignment

- Does not violate [NON_GOALS](../product/NON_GOALS.md) without an ADR.  
- Implementation PRs link the **spec** they implement.  
- Spec PRs name the **milestone** they unlock.

### DeepSeek / cache notes

If the change can affect the **stable system/tool/memory prefix**, say so explicitly (Reasonix-style cache impact). Suggested footer:

```text
Cache-impact: none | low | medium | high — <reason>
```

---

## 7. Draft vs ready

| State | Use |
|-------|-----|
| **Draft** | Early feedback, incomplete checklist, CI not green yet |
| **Ready for review** | Title/labels/template complete; CI green or explained |

Convert Draft → Ready only when you want review/merge attention.

---

## 8. Review expectations

### Author

- Self-review the diff in the GitHub UI before requesting review.  
- Keep discussion on the PR; update the branch instead of opening a replacement PR for the same unit (unless abandoned).  
- Respond to review comments or push fixes; resolve threads when addressed.

### Reviewer (when applicable)

- Check **correctness**, **scope creep**, **spec alignment**, and **security** (secrets, permissions).  
- Prefer requesting a follow-up PR over bloating the current unit.  
- Approve only when kind label + CI + template bar are met.

### Solo maintainer mode

Self-merge is allowed after:

1. CI green  
2. Kind label present  
3. Template filled honestly  
4. 24h wait **optional** — not required for docs/chore/spec scaffolding in early milestones  

---

## 9. Merge policy

| Policy | Choice |
|--------|--------|
| Default merge method | **Squash and merge** |
| Squash commit message | Use the **PR title** (+ optional body from GitHub UI) |
| Merge commits | Avoid for feature work; ok for deliberate multi-commit history only if justified |
| Delete branch | On (repo setting) |

After merge:

- Local: `git checkout main && git pull && git branch -d <branch>`  
- Do not continue work on a merged branch.

---

## 10. Milestone & issue linking

1. Set **Milestone** (M1–M6) on the PR when work is planned.  
2. Link issues: `Closes #12` for full completion, `Refs #12` otherwise.  
3. Prefer one primary issue per PR; multi-close only when the unit truly finishes all of them.

---

## 11. What not to do

- Drive-by reformat of unrelated files  
- Mix `spec` + large `feat` in one PR without necessity  
- Empty titles like `update` / `fix stuff`  
- Force-push to `main`  
- Commit API keys, session dumps, or private paths with secrets  
- Open a PR that depends on unmerged unpublished local-only docs without linking them  

---

## 12. Agent / automation checklist

Coding agents opening PRs for this repo **must**:

1. Work on a feature branch (not `main`).  
2. Use a conventional title and matching kind label (`gh pr create --label …`).  
3. Fill Summary + Test plan.  
4. Set milestone when known.  
5. Verify labels after create: `gh pr view --json labels`.  
6. Not mark work “done” if the PR has zero labels or failing required CI.

---

## Related

- [commits.md](./commits.md)  
- [branches.md](./branches.md)  
- Root [CONTRIBUTING.md](../../CONTRIBUTING.md)  
- Product [MILESTONES.md](../product/MILESTONES.md)
