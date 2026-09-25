---
name: session-unit
description: "Use when a session starts, drifts off the opening ask, a follow-up unit appears, or the user says to focus or finish: do that one unit through the PR, then hand the next to a new Orca tab."
---

# Session unit

The rules live in [AGENTS.md](../../AGENTS.md) §One session, one unit. If this
file and that section disagree, that section wins. This file is only the
order. Do not copy commands out of `worktree-dispatch` or `pr-authoring`.

## Order

1. **Pin.** Before the first edit, write one sentence: the done-condition, in
   the opening's words. If the opening has no finish line, use the smallest
   fact that makes the opening true.
2. **Look.** `worktree-dispatch` §0. If another worktree already owns this
   unit, stop and name it.
3. **Change** only what the done-condition needs. A defect in a file this
   unit is already changing, unowned by another session, is its own commit
   and still this unit. Anything the opening did not name waits for step 5.
4. **Ship,** unless the opening said to stop earlier.
   - Run the checks the change needs. The report names the command and what
     it printed.
   - One concern per commit: `git add <path>` then
     `git commit -F <message-file> -- <path>`.
   - Open the PR with `pr-authoring`, which includes the push. From the
     control tower, use that skill's control-tower mode (`--repo`, `--head`,
     per-command token).
   - Merge it: the merge is part of the unit, not a separate permission
     ([AGENTS.md](../../AGENTS.md) §One session, one unit). Stop short only
     when the opening said so — a review-only unit, a stacked child whose
     parent is unmerged, or a PR the user asked to look at first.
   - Clean up the worktree with `worktree-dispatch` §4.
5. **Hand off** each next unit with `worktree-dispatch` §1–§3b. One worktree,
   one `deepseek-build` tab. The brief file lives outside the repo, and its
   first line is that unit's done-condition. Do not ask first. After the
   receipt shows `turn_started`, report the handle and that sentence, then
   leave the unit alone.
6. **Close.** The first sentence of the report is that this unit is done.
   Then: whether the done-condition holds, or which part does not; commits;
   PR URL; the check you ran; handles you opened. Stop. Do not offer another
   unit here.

## Anti-patterns

| Don't | Why |
|-------|-----|
| Rewrite the done-condition into something smaller mid-session | The unit becomes whatever was easy |
| Start the next unit in this session | It never gets its own brief, and this session never closes |
| Ask whether to open the follow-up tab | The handoff is the tab. Asking keeps the work here |
| Offer another unit after you are done | The offer is the next task, and it lands in this session |
| Say the checks passed without naming the command | The report is then only a claim |
| `git commit` with no path | A shared index takes another session's files with it |

## Done means

- [ ] The done-condition was written before editing and still matches the opening
- [ ] Every check the report cites was run
- [ ] The PR is open, or the opening explicitly stopped earlier
- [ ] Next units have their own tab and were not started here
- [ ] The report's first sentence says this unit is done
