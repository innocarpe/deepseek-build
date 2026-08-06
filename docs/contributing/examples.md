# Worked PR examples

Bodies below are the **quality bar**. They intentionally resemble detailed product-repo PRs (Orca-style narrative: problem, evidence, testing honesty, AI review, security, notes)—scaled to DeepSeek Build.

Copy structure and density, not the fake commands if your tree differs.

Full rules: [pr-body-standard.md](./pr-body-standard.md).

---

## Example A — `spec` PR (M1 cache contract)

**Title:** `spec(cache): define byte-stable system prefix rules`  
**Labels:** `spec`, `area/cache`, `size/M`, `milestone-aligned`  
**Milestone:** M1 — Provider + cache + routing  
**Branch:** `spec/10-cache-contract`

```markdown
## Summary

### Problem

DeepSeek long sessions only stay cheap if the **prefix** (system + tools + skills
index + standing memory) is byte-stable across turns. Without a written contract,
implementation PRs will each invent different “helpful” dynamic injections
(timestamps, full tree listings, reordered tool JSON) and silently destroy
prefix-cache hit rates—the opposite of the Reasonix lesson in SOURCES.md.

### What changed

- Add `docs/specs/10-cache-contract.md` as the normative split:
  - **Stable:** system prompt body, tool schemas (canonical JSON), skills index
    lines, standing project memory files loaded at session start
  - **Tail / unstable:** user turn, dynamic reminders, volatile absolute paths
    that must be normalized or excluded
- Acceptance criteria are falsifiable (hash equality across two turns with
  identical stable inputs; schema key sort order pinned).
- Explicit non-goals: mid-session tool schema rewrite, stuffing the full
  workspace walk into the stable prefix every turn.

### Out of scope

- Implementing `PrefixBuilder` in code (follow-up `feat` PR against this spec)
- Flash/Pro routing rules (spec 20)
- Compaction algorithm details beyond “must not thrash stable prefix”

## Screenshots / evidence

No visual change. Reviewer should read end-to-end:

1. `docs/specs/10-cache-contract.md`
2. Cross-check `docs/product/SOURCES.md` (Reasonix row)
3. Cross-check `docs/product/PRD-v1.md` goal G2

## Testing

- [x] Spec-only: no runtime entrypoints changed (`git diff --stat` shows docs only)
- [x] Acceptance criteria each map to a future unit-test name listed in the spec
- [x] Consistency walkthrough: SOURCES + PRD + NON_GOALS — no contradiction found
- [ ] Runtime tests — N/A (no code)

## Kind

- [x] `spec`

## Related

- **Milestone:** M1 — Provider + cache + routing
- **Spec / ADR:** docs/specs/10-cache-contract.md (this PR)
- **Issues:** n/a

## Cache impact

`high` — this document *is* the cache contract; later feats must not violate it

## AI review report

Self-review + agent pass focused on:

- **Falsifiability** — rejected two earlier criteria that only said “should be
  cache friendly” without a measurable check; replaced with hash/sort rules.
- **Over-constraint** — allowed volatile paths on the *tail* so we do not force
  impossible absolute-path stability across machines.
- **Source priority** — confirmed Reasonix-aligned; did not import Gajae planning
  language into the contract.

## Security audit

No runtime surface. Spec forbids embedding secrets into the stable prefix and
requires redaction guidance for memory files that might contain keys (called out
in §Secrets of the spec).

## Notes

- Follow-up PR should implement `PrefixBuilder` with golden tests that fail on
  `main` without the sort/hash rules.
- If DeepSeek API documents additional cache headers later, extend this spec
  rather than inventing headers only in code.
```

---

## Example B — `feat` PR implementing a merged spec

**Title:** `feat(provider): build cache-stable system prefix`  
**Labels:** `feat`, `area/provider`, `area/cache`, `size/M`  
**Milestone:** M1

```markdown
## Summary

### Problem

Spec 10 is merged, but nothing constructs the stable prefix yet. Without a
single builder, every call site will assemble prompts differently and break the
byte-stability rules on day one.

### What changed

- Add `PrefixBuilder` that assembles stable sections per
  `docs/specs/10-cache-contract.md` §Stable sections.
- Canonicalize tool schema JSON with sorted object keys before hashing/shipping.
- Unit tests: identical inputs → identical bytes across two builds; turn-tail
  reminder must not change the stable hash.

### Why one PR

Builder + its golden tests are one review lens (“does this implement §Stable?”).
Routing (Flash/Pro) stays out so this PR cannot be blocked on model policy.

### Out of scope

- Streaming HTTP client
- Tool execution loop
- Compaction

## Screenshots / evidence

No TUI yet. Evidence is test output:

| Check | Result |
| --- | --- |
| `prefix_stable_across_turns` | pass |
| `schema_key_order_canonical` | pass |
| Intentional mutation: drop key sort | test fails (verified once) |

## Testing

- [x] `cargo test -p dsb-provider-deepseek prefix::` (example name)
- [x] Mutation check: removed key sort → golden test failed
- [ ] Full workspace test suite — not run; package-local only (reason: no other
      packages depend on this yet)
- [x] Spec §Acceptance criteria each have a named test

## Kind

- [x] `feat`

## Related

- **Milestone:** M1
- **Spec / ADR:** docs/specs/10-cache-contract.md (merged)
- **Issues:** Closes #N

## Cache impact

`high` — constructs the cached prefix; bugs here multiply cost every session

## AI review report

- **False stability** — flagged wall-clock timestamps in a draft debug header;
  removed from stable section.
- **Hash algorithm** — review asked for an explicit documented hash (blake3 vs
  sha256); chose sha256 for zero extra dep in MVP, noted in Notes.
- **Permissions** — builder only reads configured memory paths; no shell.

## Security audit

- Memory file reads are path-confined to project + user config roots (list in
  code constants); no user-controlled path join from model output in this PR.
- Stable prefix must not include env vars that hold `API_KEY` (test asserts
  redaction hook is invoked).
- No new network surface.

## Notes

- Hash choice may move to blake3 later if we want faster large schemas; not a
  user-visible break if we version the hash label in logs only.
```

---

## Example C — `fix` PR (CI title regex)

**Title:** `fix(ci): allow digits in conventional title scopes`  
**Labels:** `fix`, `area/infra`, `size/S`

```markdown
## Summary

### Problem

Titles like `spec(10-cache): define rules` fail the local title helper and
confuse agents following Conventional Commits with numeric scopes. Our spec
index uses numeric prefixes (`10-cache-contract`); the scope regex only allowed
`[a-z]`, so legitimate titles were rejected.

### What changed

- Extend scope pattern to `[a-z0-9][a-z0-9/_-]*` in `scripts/check-pr-title.sh`.
- Add positive/negative examples to contributing docs.

### Out of scope

- Changing allowed types list

## Screenshots / evidence

No visual change.

| Title | Before | After |
| --- | --- | --- |
| `spec(10-cache): define rules` | fail | pass |
| `bad title` | fail | fail |

## Testing

- [x] `./scripts/check-pr-title.sh "spec(10-cache): define rules"` → ok
- [x] `./scripts/check-pr-title.sh "bad title"` → fails
- [ ] Product CI — N/A (no product test surface for this change)

## Kind

- [x] `fix`

## Related

- **Milestone:** n/a
- **Spec / ADR:** docs/contributing/pull-requests.md §Title
- **Issues:** n/a

## Cache impact

`none`

## AI review report

Self-review: confirmed `feat(provider): …` still matches; empty scope still
works; trailing period rule unchanged.

## Security audit

No security-sensitive surface — CI string match only; no shell interpolation of
the title beyond `grep`.

## Notes

If we adopt scopes like `area/cache`, `/` is already allowed.
```

---

## Example D — `docs` process PR (depth + Orca bar)

**Title:** `docs(contributing): adopt Orca-level PR body standard`  
**Labels:** `docs`, `area/docs`, `area/infra`, `size/M`

```markdown
## Summary

### Problem

DeepSeek Build PR #1 shipped process *gates* and short rule lists. Compared to
Orca PRs (multi-thousand-character Summaries with assumption tables, honest
Testing, AI Review Report, Security Audit, Notes), our template and examples
were still checklist-thin. CI could pass while the PR body remained review-useless.

### What changed

- Align `.github/PULL_REQUEST_TEMPLATE.md` with Orca’s section set (Summary with
  Problem / What changed / Out of scope, Screenshots/evidence, Testing, AI
  review report, Security audit, Notes) plus our kind/milestone/cache fields.
- Add `docs/contributing/pr-body-standard.md` explaining the narrative bar and
  Orca → DeepSeek Build mapping.
- Replace thin examples with full Orca-density worked bodies.
- Expand review checklist to grade narrative sections, not only labels.

### Out of scope

- Product feature specs
- Runtime implementation
- Enforcing body length in CI (human/agent standard; not a regex)

## Screenshots / evidence

No visual change. Compare:

- Orca template: `OpenSources/orca/.github/pull_request_template.md`
- Orca example density: stablyai/orca PRs such as #12860 / #12848 (Summary
  narrative + Testing honesty + AI Review + Security)
- This PR’s template + `pr-body-standard.md` + `examples.md`

## Testing

- [x] Links from CONTRIBUTING / docs/README / contributing/README resolve
- [x] CI required paths include new standard doc
- [x] `actionlint` on workflow if touched
- [ ] Runtime tests — N/A

## Kind

- [x] `docs`

## Related

- **Milestone:** n/a
- **Spec / ADR:** docs/adr/0003-pr-process.md
- **Issues:** feedback that PR #1 / early conventions were too thin vs Orca

## Cache impact

`none`

## AI review report

- Checked that we did **not** cargo-cult Electron/IPC wording into a CLI repo.
- Checked that Security/AI sections are required in spirit for non-trivial PRs
  but allow a short escape hatch for pure typos.
- Verified examples still teach one-unit-of-work (no mega-MVP example as “good”).

## Security audit

No security-sensitive surface — documentation and PR template only. Template
reminds authors to redact secrets in evidence.

## Notes

- Body quality is intentionally **not** fully enforceable by CI (like Orca);
  the bar is social + review checklist + agent instructions in AGENTS.md.
- Follow-up: when runtime exists, replace Testing placeholders with real
  workspace commands in the template comments.
```

---

## Example E — intentionally bad (reject)

```markdown
## Summary
- misc improvements

## Testing
- [ ] tests

## AI Review Report
LGTM

## Security Audit
N/A
```

**Why reject:** no problem, no evidence, no commands, no kind, no cache thinking, AI/security theater.
