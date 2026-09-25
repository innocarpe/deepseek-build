# Stack merge runbook

**Normative for ultragoal stacking.** Companion: [ULTRAGOAL_PR_PLANNING.md](../product/ULTRAGOAL_PR_PLANNING.md).

`main` uses **merge commits** — squash and rebase are disabled on this repo, so a
merged parent's commits stay ancestors and a stacked child rebases onto the new
`main` without a repair step.

---

## 1. When to stack vs serial-merge

| Situation | Strategy |
|-----------|----------|
| B needs A’s API and A is not on `main` yet | **Stack:** B base = A branch |
| A is small and will merge in minutes | **Serial:** merge A, `pull main`, branch B from main |
| Units are disjoint | **Parallel** from `main` (no stack) |

**Default for overnight solo agent:** prefer **serial merge** (unit → PR → merge → pull → next).  
Use **stack** only when wall-clock needs concurrent review of A and B.

---

## 2. Open a stack

```bash
# Parent
git checkout main && git pull origin main
git checkout -b feat/unit-a
# atomic commits …
git push -u origin HEAD
gh pr create --base main --title "feat(…): A" --label feat --body "…"

# Child
git checkout -b feat/unit-b   # from feat/unit-a tip
# atomic commits …
git push -u origin HEAD
gh pr create --base feat/unit-a --title "feat(…): B" --label feat \
  --body "Depends on #N (unit A). Stack child."
```

---

## 3. Merge-ready predicate (before `gh pr merge`)

All must hold:

```bash
gh pr view N --json state,mergeable,mergeStateStatus,statusCheckRollup,reviews
# state=OPEN, mergeable=MERGEABLE (or unknown then recheck)
# No failing required checks if any exist
# For stacks: parent N-1 already MERGED if using serial; if stacked, merge parent first
```

Do **not** merge if `mergeStateStatus` is `DIRTY` / `BLOCKED` (unless only failing optional checks — this repo has little CI; still fix conflicts).

---

## 4. Bottom-up merge + child rebase

```bash
# 1) Merge parent (merge commit — squash is disabled on this repo)
gh pr merge <A> --merge --delete-branch

# 2) Update main
git checkout main && git pull --ff-only origin main

# 3) Rebase child B onto new main. A's commits are ancestors of the merge
#    commit, so a plain rebase drops them — no --onto needed.
git fetch origin
git checkout feat/unit-b
git rebase main

git push --force-with-lease origin feat/unit-b

# 4) Retarget PR base to main if still pointing at the (now deleted) parent branch
gh pr edit <B> --base main

# 5) Verify only B diff remains
gh pr diff <B>   # must NOT re-include A changes

# 6) Merge child
gh pr merge <B> --merge --delete-branch
git checkout main && git pull --ff-only origin main
```

---

## 5. Failure ladder (overnight)

| Event | Action | Ultragoal |
|-------|--------|-----------|
| Conflict on rebase | Fix once; if 2nd fail → open `blocked` note, switch to **disjoint** unit if any | `checkpoint --status blocked` with evidence |
| 3 consecutive test failures same unit | Stop unit; write failing test note; do not thrash | blocked or failed |
| `gh` auth / permission | Document exact command for human | blocked-awaiting-human |
| npm publish needs OTP | Package DoD only; never fake publish | blocked-awaiting-human for publish sub-goal |
| API 429 / quota | Backoff; skip live smoke; keep offline tests | evidence note; continue offline units |
| Gate red for needed runtime | Land **spec unit first** or skip runtime story | do not invent specs mid-code |

**Retry budget:** max **3** automated fix attempts per unit failure class, then blocked.

---

## 6. Anti-patterns

- Merge child before parent  
- `git rebase --onto` gymnastics left over from a squash flow — with merge
  commits on `main`, a plain `git rebase main` already drops the parent's commits  
- Force-push `main`  
- Parallel agents both editing `Cargo.lock`  
