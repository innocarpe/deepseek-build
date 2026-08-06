# Review checklist

Use this when reviewing someone else’s PR **or** self-merging.

---

## A. Process gates (must pass)

- [ ] Title matches Conventional Commits (`type` / `type(scope): summary`)
- [ ] Exactly one **kind** label; matches title type
- [ ] CI green: `docs-hygiene`, `pr-title`, `pr-kind-label` (and future jobs)
- [ ] Milestone set when the work maps to M1–M6
- [ ] PR body Summary is not a file list only — states **what and why**
- [ ] Test plan is falsifiable
- [ ] Cache-impact filled honestly for agent/prompt/tool changes

If any fail → request changes; do not “merge and fix later” on process gates.

---

## B. Unit of work

- [ ] One-sentence outcome is clear
- [ ] Does not mix **spec lock** + **large implementation** without strong reason
- [ ] Does not span multiple milestone exit criteria
- [ ] Revert story is acceptable
- [ ] No unrelated drive-by refactors/formats

---

## C. Product alignment

- [ ] Consistent with [PRD-v1](../product/PRD-v1.md) and [SOURCES](../product/SOURCES.md)
- [ ] Does not violate [NON_GOALS](../product/NON_GOALS.md) (no Gajae multi-stage harness smuggling)
- [ ] If `feat`/`fix` on agent behavior: linked **spec** exists and is cited
- [ ] Speed north star: change does not introduce serial-only ceremony without payoff

---

## D. Spec PRs (`spec`)

- [ ] Behavior is testable (acceptance criteria, not vibes)
- [ ] Non-goals listed
- [ ] Interactions with cache / Flash-Pro / tools called out
- [ ] Naming/paths match `docs/specs/00-overview.md` index when applicable
- [ ] Does not pretend to implement runtime

---

## E. Implementation PRs (`feat` / `fix`)

- [ ] Matches cited spec sections (or documents intentional delta + why)
- [ ] Error paths and permissions considered
- [ ] No secrets in diff or fixtures
- [ ] Tests or manual smoke proportional to risk
- [ ] Stable prefix / tool schema: no accidental non-determinism (timestamps, random IDs, unsorted maps) in cache-stable regions

---

## F. Docs / process PRs (`docs` / `chore` / `ci`)

- [ ] Normative vs advisory language is clear
- [ ] Links work
- [ ] Does not invent product behavior that should be a `spec`
- [ ] CI changes: before/after examples of what fails

---

## G. Security / safety

- [ ] No API keys, tokens, private session logs
- [ ] Shell/file tools: no silent expansion of trust boundaries without permissions discussion
- [ ] Dependency bumps: note supply-chain risk if any

---

## H. Final merge call

Only merge if you can answer **yes**:

1. Would I accept this from an external contributor as-is?  
2. Can a future reader of `git log` understand the intent from the squash title alone?  
3. If this is wrong, can we revert or follow up without a archaeology dig?
