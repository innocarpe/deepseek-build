---
name: worktree-dispatch
description: >
  From the control-tower (primary) checkout of DeepSeek Build, create an Orca
  worktree for one unit of work, give its branch the repo's name, work it by
  path or open an agent session in it and hand over a brief without losing it
  to a first-run trust prompt, then remove the worktree after merge. Use when
  starting any change from the primary checkout, dispatching work to a new
  agent session, running units in parallel, or cleaning up merged worktrees.
---

# Worktree dispatch (control-tower checkout)

The primary checkout is the control tower: it stays on `main`, clean, and
directs work ([AGENTS.md](../../AGENTS.md) §Control-tower checkout). Code
changes happen in Orca worktrees — **one worktree = one branch = one PR.**

This skill is the procedure. The version-matched reference for `orca` syntax is
`orca skills get orca-cli` plus `orca <command> --help`; the commands below were
checked against Orca CLI `1.4.210`. If a flag is rejected, trust the CLI's help
over this file.

Variables used below:

| Name | Value |
|------|-------|
| `TOWER` | Primary checkout: `git rev-parse --show-toplevel` run from the tower session |
| `WT_ID` | Orca worktree id `<repoId>::<path>` — always the whole value |
| `WT` | Worktree path |
| `H` | Orca terminal handle of the agent tab |

## 0. Before you create one

- **Is someone already on it?**
  `orca worktree list --repo path:"$TOWER" --json` shows every worktree with its
  branch and card comment. Do not open a second worktree for a unit that has
  one, and do not touch another session's worktree.
- **Does this unit build vendored Grok?** It does if it touches
  `third_party/grok-build/**` or `patches/grok-build/**`, or needs
  `scripts/build-grok-pager.sh`, `scripts/install.sh` or
  `scripts/package-release-binaries.sh`. Those run **one at a time across all
  worktrees** (cold build 30–60+ min per tree). A quick check for a build in
  flight: `pgrep -fl 'grok-build/target'` (the rustc and build-script
  processes carry that output path; the parent `cargo` does not). It is a
  hint, not a lock — when in doubt, ask the other sessions.

## 1. Create the worktree

```sh
orca worktree create --repo path:"$TOWER" --name <slug> --no-parent --json
```

- `<slug>` is a short kebab description **without** the type prefix
  (`provider-retry-backoff`). Orca turns `/` into `-`, so `feat/x` would become
  `feat-x`; the type goes on the branch in step 2.
- `--no-parent` marks an independent unit. Omit `--base-branch` so the repo
  default (`origin/main`) is used. Stack on an open PR only on purpose:
  `--base-branch <that PR's branch>` and `Depends on #N` in the PR body.
- From the JSON result keep `result.worktree.id` → `WT_ID` and
  `result.worktree.path` → `WT`.
- A bare create also opens a plain shell tab in the worktree. Leave it, or
  close it after `orca terminal read` shows it idle. Never close a tab you have
  not read.

## 2. Name the branch

Orca names the branch after the worktree (plus any branch prefix in the user's
Orca settings). Rename it to [branches.md](../../docs/contributing/branches.md)
form **before the first push**:

```sh
git -C "$WT" branch -m "<type>/<slug>"    # feat | fix | docs | spec | chore | ci | refactor | test
git -C "$WT" branch --show-current
git -C "$WT" config user.email             # the identity you commit to this repo with
```

Orca can keep showing the old branch name for a moment. Address the worktree by
`id:` or `path:`, not `branch:`.

## 3a. Work it from the tower, by path

A short unit can be done by the tower session itself. Never `cd` into the tree
and stay there — target it per command:

| Tool | Target |
|------|--------|
| `git` | `git -C "$WT" …` |
| `cargo`, `scripts/*.sh`, `npm` | subshell: `(cd "$WT" && cargo test -p dsb-cli)` |
| `gh` | `--repo innocarpe/deepseek-build`, token per command (AGENTS.md) |
| file edits | absolute paths under `$WT` |

## 3b. Or open an agent session in it

```sh
orca terminal create --worktree "id:$WT_ID" --title "<slug>" --command "<agent launcher>" --json
# H = result.terminal.handle
orca terminal wait --terminal "$H" --for tui-idle --timeout-ms 60000 --json   # need result.wait.satisfied: true
orca terminal read --terminal "$H" --screen --json                           # look before you type
```

`<agent launcher>` is whatever starts the agent with your flags — `claude …`,
`codex …`, or `deepseek-build` (alias `dsb`) to dogfood. A timed-out wait still
prints a result; read `satisfied`, and if it is `false` wait once more with a
longer timeout before giving up on the handoff.

### The trust-prompt trap

`tui-idle` is also satisfied while the agent sits on a **first-run dialog**,
not only at its input box. The first time Claude Code opens in a folder it has
no trust record for, the screen is:

```text
 Quick safety check: Is this a project you created or one you trust? …
 ❯ No, exit
   Yes, I trust this folder
 Enter to confirm · Esc to cancel
```

`No, exit` is listed first and selected, and digit keys do not pick an option.
A brief sent now with `--enter` is typed into the dialog and its Enter confirms
**No, exit**: Claude Code quits and the brief is lost. This has happened on a
real handoff. Codex shows its own first-run trust prompt; handle it the same
way.

Whether the dialog appears depends on the machine's trust records (worktrees
of an already-trusted checkout may skip it; a folder never seen before shows
it). Do not predict it — read the screen every time, and if it is there:

```sh
orca terminal send --terminal "$H" --text $'\e[B' --json   # Down arrow → "Yes, I trust this folder"
orca terminal read --terminal "$H" --screen --json         # confirm ❯ is on the Yes line
orca terminal send --terminal "$H" --enter --json
orca terminal wait --terminal "$H" --for tui-idle --timeout-ms 60000 --json
orca terminal read --terminal "$H" --screen --json         # the agent's input box must show now
```

Only trust a worktree of this repo that you just created. If the screen shows
a dialog you do not recognise, stop and report; do not press keys blind.

### Send the brief

```sh
orca terminal send --terminal "$H" --text "<brief>" --enter --wait-submit 15 --json
```

The handoff is done when the receipt says `accepted: true` and its `stages`
include `turn_started`. A receipt that stops at `input_accepted` is unproven,
not failed: read the screen, and never resend on silence. After a transport
error, repeat the exact command with `--retry-request <id>` from the receipt.

For a long brief, write it to a file **outside** the repo (the worktree stays
clean) and send one line: `Read <file> and do what it says.`

What a brief carries:

- **Unit** — the outcome in one sentence, PR kind and title, branch name,
  `Depends on #N` if stacked
- **Boundaries** — which worktree is theirs; the primary checkout and other
  worktrees are off-limits; the one-Grok-build-at-a-time rule
- **Harness** — follow `AGENTS.md` and `skills/pr-authoring`; `gh` with
  `--repo` and a per-command token; push to `origin`
- **Authority** — whether merging was granted, stated either way
- **Report** — PR URL, test evidence, what was left out

Then mark the card so other sessions can see the state:
`orca worktree set --worktree "id:$WT_ID" --comment "<one-line state>" --json`

## 4. Clean up after merge

Only for a worktree whose PR is merged and whose session has finished:

```sh
GH_TOKEN="$(gh auth token --user <account>)" gh pr view <n> --repo innocarpe/deepseek-build --json state,mergeCommit
git -C "$WT" status --short                          # must print nothing
orca terminal close --worktree "id:$WT_ID" --all --json
orca worktree rm --worktree "id:$WT_ID" --json
```

`orca worktree rm` also deletes the local branch, but only when Orca can prove
it is merged. GitHub deletes the remote branch on merge (repo setting).
`--force` removes a dirty tree; use it only after reading what is dirty.

Then refresh the tower — only when it is on `main` and clean:

```sh
test "$(git -C "$TOWER" branch --show-current)" = main \
  && test -z "$(git -C "$TOWER" status --porcelain)" \
  && git -C "$TOWER" pull --ff-only origin main
```

Never remove a worktree another session created or is still using, even when
it looks finished — ask its session first.

## Anti-patterns

| Don't | Why |
|-------|-----|
| Edit or commit in the primary checkout | Tower sessions share its index; the change lands in someone else's diff |
| `terminal send … --enter` right after `tui-idle` without reading the screen | A first-run dialog swallows the brief and exits the agent |
| `cd "$WT"` and keep working | The next command meant for the tower runs in the tree, or the reverse |
| `gh auth switch` | Flips the active account for every session on the machine |
| Two vendored Grok builds at once | 30–60+ min cold each; parallel builds starve each other |
| `--name feat/x` | Becomes `feat-x`; name the worktree by slug and rename the branch |
| `worktree rm --force` on a tree you did not read | Deletes uncommitted work with no undo |
