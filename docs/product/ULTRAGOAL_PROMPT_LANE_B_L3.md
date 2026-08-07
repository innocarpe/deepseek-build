# Ultragoal cold-start — **`l3-prep-lane-b`** (parallel L3 prep)

Paste the fenced block into a session whose cwd is the **L3 worktree**  
(`deepseek-build-l3-prep`), **not** the heart-3x dirty tree.

---

```text
# ROLE
You execute plan l3-prep-lane-b only. Parallel-safe 4.0 prep.
Never git stash or reset the heart-3x working tree.
Work only in this worktree; PR base main; merge with --merge.
Child runtime = parent family (grok only unless user crosses).

# FINAL GOAL
Close all B001–B008 in docs/product/LANE_B_L3_PREP_GOALS.md.
Do NOT start fleet-4x. Do NOT change product L3 defaults.
Do NOT implement heart-3x L1/L2.

# CONSTRAINTS
- Touch-set: docs/user-guide, docs/research/l3-*, docs/product/evidence/L3_*,
  scripts/test-l3-smoke.sh, PARALLEL/KNOWN_LIMITS honesty links only.
- Avoid: search_replace fusion, permissions matrix, context prefix, SemVer 3/4 bumps.
- Disk: no vendor-full cargo test.

# VERIFY
git status  # clean or only Lane B files
./scripts/test-l3-smoke.sh --offline-only
test -f docs/user-guide/14-l3-throughput.md
omc ultragoal status --plan-id l3-prep-lane-b

# LOOP
complete-goals → implement unit → PR → merge commit → checkpoint → repeat until 8/8.

# END
```
