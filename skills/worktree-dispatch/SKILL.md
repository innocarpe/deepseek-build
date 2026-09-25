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
| `TOWER` | Primary checkout: `dirname "$(git rev-parse --path-format=absolute --git-common-dir)"` — the same answer from the primary checkout or any worktree (`--show-toplevel` would give a worktree's own path, which `--repo path:` rejects) |
| `WT_ID` | Orca worktree id `<repoId>::<path>` — always the whole value |
| `WT` | Worktree path |
| `H` | Orca terminal handle of the agent tab |

## 0. Before you create one

- **Is someone already on it?**
  `orca worktree list --repo path:"$TOWER" --json` shows every worktree with its
  branch and card comment. Do not open a second worktree for a unit that has
  one, and do not touch another session's worktree.
- **Does this unit build vendored Grok?** It does if anything it runs calls
  `cargo` in `third_party/grok-build` — directly, or through a script
  (`rg -l grok-build scripts/` lists them: `build-grok-pager.sh`,
  `install.sh`, `test-grok-vendor-offline.sh`, the `test-path-a-*` scripts, …).
  Those units run **one at a time across all worktrees** (cold build 30–60+ min
  per tree). A quick check for a build in
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
| `gh` | `--repo innocarpe/deepseek-build`, token per command (AGENTS.md); `gh pr create` also needs `--head <type>/<slug>`, since it reads the head branch from cwd |
| file edits | absolute paths under `$WT` |

## 3b. Or open a session in it

```sh
orca terminal create --worktree "id:$WT_ID" --title "<slug>" --command "deepseek-build" --json
# H = result.terminal.handle
orca terminal wait --terminal "$H" --for tui-idle --timeout-ms 60000 --json   # need result.wait.satisfied: true
orca terminal read --terminal "$H" --screen --json                           # look before you type
```

The launcher is `deepseek-build` (alias `dsb`): work on this repo runs under the
product the repo ships ([AGENTS.md](../../AGENTS.md) §Control-tower checkout).
A timed-out wait still prints a result; read `satisfied`. If it is `false` or
absent, **read the screen before anything else** — a first-run dialog can hold
the agent there. Otherwise wait once more with a longer timeout, and if it is
still unsatisfied, report the handoff as not started.

### A first-run dialog can swallow the brief

An agent opening a folder it has no trust record for may show a trust dialog
before its input box exists, and `tui-idle` does not have to mean "ready for
input":

```text
 Quick safety check: Is this a project you created or one you trust? …
 ❯ No, exit
   Yes, I trust this folder
 Enter to confirm · Esc to cancel
```

A brief sent with `--enter` while such a dialog is up is typed into the dialog,
and the Enter confirms whatever its default is — the agent exits and the brief
is lost. This has happened on a real handoff.

Read the screen after the wait. If a dialog is up, pass it *deliberately*: move
the selection to the "yes" line, read the screen again to confirm the **text**
of the selected line says yes (not the marker glyph, and not a table from a
previous version), then press Enter — and read again before sending the brief.
If the screen shows a dialog you do not recognise, stop and report; do not
press keys blind.

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
if [ "$(git -C "$TOWER" branch --show-current)" != main ]; then
  echo "tower is not on main; not pulling"
elif [ -n "$(git -C "$TOWER" status --porcelain)" ]; then
  echo "tower has local changes; not pulling:"; git -C "$TOWER" status --short
else
  git -C "$TOWER" pull --ff-only origin main
fi
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
