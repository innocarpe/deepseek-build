# Worked PR examples

Copy structure, not prose. Each example is the **body quality bar** for that kind.

---

## Example A — `spec` PR (M1 cache contract)

**Title:** `spec(cache): define byte-stable system prefix rules`  
**Labels:** `spec`, `area/cache`, `size/M`, `milestone-aligned`  
**Milestone:** M1 — Provider + cache + routing  
**Branch:** `spec/10-cache-contract`

```markdown
## Summary

- Add `docs/specs/10-cache-contract.md` as the normative rules for what must
  stay byte-stable across turns (system prompt, tool schemas, skills index,
  standing project memory) versus what may live on the turn tail.
- Explicitly lock non-goals: mid-session tool schema rewriting, per-turn
  injection of full workspace trees into the stable prefix.
- Align with Reasonix cache-first lessons and PRD v1 G2 (cache discipline).

## Kind

- [x] `spec`

## Related

- **Milestone:** M1 — Provider + cache + routing
- **Spec / ADR:** docs/specs/10-cache-contract.md (this PR)
- **Issues:** Refs #N (if a tracking issue exists)

## Test plan

- [ ] Read §Acceptance criteria in the new spec; each item is falsifiable
- [ ] Cross-check SOURCES.md (Reasonix row) — no contradiction
- [ ] Cross-check PRD §6.1 cache row — language matches
- [ ] Confirm no runtime code claims to implement this yet (spec-only PR)

## Cache impact

`high` — this document *is* the cache contract; later feats must not violate it

## Checklist

- [x] Conventional title
- [x] Kind label `spec` only
- [x] One unit (contract only)
- [x] NON_GOALS respected
- [x] No secrets
```

**Why this is a good unit:** reviewer only judges the contract, not an incomplete client.

---

## Example B — `feat` PR implementing a merged spec

**Title:** `feat(provider): build cache-stable system prefix`  
**Labels:** `feat`, `area/provider`, `area/cache`, `size/M`  
**Milestone:** M1  
**Branch:** `feat/provider-stable-prefix`

```markdown
## Summary

- Implement `PrefixBuilder` that assembles the stable prefix per
  docs/specs/10-cache-contract.md §Stable sections.
- Snapshot tool schema JSON with sorted keys so field order cannot drift.
- Unit tests: two consecutive builds with identical inputs produce identical
  bytes; adding a turn-tail reminder does not change the stable hash.

## Kind

- [x] `feat`

## Related

- **Milestone:** M1
- **Spec / ADR:** docs/specs/10-cache-contract.md (merged)
- **Issues:** Closes #N

## Test plan

- [ ] `cargo test -p dsb-provider-deepseek prefix::` (or project equivalent)
- [ ] Manual: run two-turn smoke; log stable prefix hash equal across turns
- [ ] Grep: no `format!(..., chrono::Utc::now())` inside stable sections

## Cache impact

`high` — constructs the cached prefix; bug here multiplies cost for every session

## Checklist

- [x] Implements named spec sections (list them in review notes if helpful)
- [x] No scope creep into Flash/Pro routing (separate spec/feat)
```

**Why this is a good unit:** one implementable surface of one merged contract.

---

## Example C — `fix` PR

**Title:** `fix(ci): allow scoped PR titles with digits in scope`  
**Labels:** `fix`, `area/infra`, `ci` is wrong if kind is fix — use **`fix` only as kind**; area ok  
**Branch:** `fix/pr-title-scope-digits`

```markdown
## Summary

- PR titles like `spec(10-cache): …` were rejected because the scope regex
  disallowed digits.
- Spec index numbers are intentional in this repo; update regex + script.

## Kind

- [x] `fix`

## Related

- **Milestone:** n/a
- **Spec / ADR:** docs/contributing/pull-requests.md §Title
- **Issues:** Closes #N

## Test plan

- [ ] `./scripts/check-pr-title.sh "spec(10-cache): define rules"` → ok
- [ ] `./scripts/check-pr-title.sh "bad"` → fails
- [ ] CI `pr-title` green on this PR

## Cache impact

`none`
```

---

## Example D — `docs` process PR (this depth pass)

**Title:** `docs(contributing): deepen PR conventions with examples`  
**Labels:** `docs`, `area/docs`, `area/infra`, `size/M`

```markdown
## Summary

- Expand pull-requests.md from generic OSS boilerplate into a project-specific
  operating guide (decisions, taxonomy, unit definition, workflows, anti-patterns).
- Add worked examples and a reviewer checklist.
- Expand ADR 0003 with alternatives considered.
- Does not change CI policy except where docs mention existing jobs.

## Kind

- [x] `docs`

## Related

- **Milestone:** n/a
- **Spec / ADR:** docs/adr/0003-pr-process.md
- **Issues:** follow-up to thin PR #1 substance

## Test plan

- [ ] Read docs/contributing/pull-requests.md end-to-end — no TODOs left as “write later”
- [ ] examples.md contains at least one filled body per primary kind we use early (spec/feat/fix/docs)
- [ ] Links from CONTRIBUTING.md and docs/README.md resolve
- [ ] CI docs-hygiene still green (required paths)

## Cache impact

`none`
```

---

## Example E — bad PR (do not merge)

**Title:** `update`  
**Labels:** (none)  
**Body:** “misc improvements”

Problems:

- Non-conventional title → `pr-title` fails  
- No kind label → `pr-kind-label` fails  
- No unit of work, no test plan, no milestone  
- Cannot tell if cache/routing/product rules changed  

---

## Example F — bad mega-unit (split required)

**Title:** `feat: implement deepseek build mvp`  
**Diff:** specs 10–110 stubs + empty crates + half TUI + README rewrite  

Split into the milestone graph instead (see MILESTONES.md). Even if one person could force it through, it destroys review and agent coordination.
