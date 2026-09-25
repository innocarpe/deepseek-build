---
name: worktree-dispatch
description: >
  From the control-tower (primary) checkout of DeepSeek Build, create an Orca
  worktree for one unit of work, open an agent session in it in one step when
  the agent supports it, hand over a brief without losing it to a first-run
  trust prompt, then merge the PR and clean up — including the launcher and
  setup tabs the worktree comes with. Use when starting any change from the
  primary checkout, dispatching work to a new agent session, running units in
  parallel, or merging and cleaning up finished worktrees.
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

## 1. Create the worktree — and know the tabs it comes with

```sh
orca worktree create --repo path:"$TOWER" --name <slug> --no-parent --setup skip --json
```

- `<slug>` is a short kebab description **without** the type prefix
  (`provider-retry-backoff`). Orca turns `/` into `-`, so `feat/x` would become
  `feat-x`; the type goes on the branch in step 2.
- `--no-parent` marks an independent unit. Omit `--base-branch` so the repo
  default (`origin/main`) is used. Stack on an open PR only on purpose:
  `--base-branch <that PR's branch>` and `Depends on #N` in the PR body.
- **`--setup skip` unless the unit needs the repo bootstrap.** Passing `run`
  (or omitting the flag where the repo defines hooks) starts an `npm install`
  in a tab of its own; it is slow, and it **fails loudly when the current
  version's prebuilt asset is not published yet** — the run we measured
  fetched `deepseek-build-6.0.0-darwin-arm64.tar.gz` and got a 404. The failure
  is harmless but the tab stays.
- From the JSON result keep `result.worktree.id` → `WT_ID` and
  `result.worktree.path` → `WT`.

### `create` opens tabs you did not ask for — count them, then decide

**Measured 2026-09-25, Orca `1.4.210`.** `create` always opens the launcher
shell, and running the repo hooks opens a second tab:

| `--setup` | Tabs the new worktree gets | Titles |
|---|---|---|
| `skip` | **1** | the launcher shell |
| `run` (or omitted where hooks exist) | **2** | the launcher shell + `Setup` (the hook's run) |

```sh
orca terminal list --worktree "id:$WT_ID" --json    # before you add your own
```

**Read a tab before closing it.** `orca terminal close --terminal <handle>` is
not undoable, and a `Setup` tab that failed still holds the error text worth
reporting.

**This is the whole reason to prefer `--setup skip` plus the one-step path in
§3b.** Creating the worktree, letting hooks run, and then adding an agent tab by
hand gives three tabs where one was needed — which is exactly the mistake this
section exists to prevent.

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

## 3b. Open the agent session — prefer one step

### The one-step path (use this when the agent id exists)

`create` takes the agent and the brief itself, so the worktree never exists
without its session:

```sh
orca worktree create \
  --repo path:"$TOWER" \
  --name <slug> \
  --no-parent \
  --setup skip \
  --agent <claude|codex|grok> \
  --prompt "<brief, or one line pointing at a brief file>" \
  --json
```

`--activate` only when a **human** asked to look at that tree. It is not a
signal for where the session lives.

With `--agent --json`, read the head handle from `result.agentTerminalHandle`;
older runtimes return only `result.startupTerminal.handle`, and a folder-based
repo may return neither — then `terminal list` the new worktree and find the
one tab whose `agentIdentity` matches what you asked for.

### `--agent` does **not** accept every agent — measure before assuming

**`--agent` is validated against Orca's `TuiAgent` list** (34 ids:
`claude`, `codex`, `grok`, `gemini`, `cursor`, `droid`, …). An id outside it
fails the whole create:

```text
$ orca worktree create … --agent dsb
Unknown TUI agent "dsb"
```

**Measured 2026-09-25.** So for **this repo's own product (`dsb` /
`deepseek-build`) the one-step path is unavailable**, and the two-step path
below is the only route. For a unit worked by **Grok Build** or **Codex** —
which is most units in this repo, and what `--agent grok` is for — the one-step
path works and is preferred.

### The two-step path (only when the launcher needs custom argv)

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

**A parse failure is not a create failure.** If you cannot read the handle out
of the JSON, do **not** re-run `create` — the side effect already happened. Ask
`orca terminal list --worktree "id:$WT_ID" --json` whether the tab exists. This
is how a second tab gets created by mistake.

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

**Measured 2026-09-25 (`grok`, Orca `1.4.210`):** `tui-idle` returned
`satisfied: true` **while this dialog was still up**, so the wait agreeing is
not evidence the agent is ready. Reading the screen is the only check that
works. Grok Build's dialog offers `y` / `n`.

### Send the brief

```sh
orca terminal send --terminal "$H" --text "<brief>" --enter --wait-submit 15 --json
```

The handoff is done when the receipt says `accepted: true` and its `stages`
include `turn_started`. **A receipt that stops at `input_accepted` is unproven,
not failed** — some providers report `provider: unsupported` and can never
advance the stage. Read the screen and confirm the agent is working; **never
resend on silence**, because the first send may already have landed. After a
transport error, repeat the exact command with `--retry-request <id>` from the
receipt.

**Measured 2026-09-25 (`grok`):** the receipt stopped at `input_accepted` with
the warning *"input was accepted, but this provider cannot report delivery"*,
and the screen showed the brief was submitted and the agent had started.

For a long brief, write it to a file **outside** the repo (the worktree stays
clean) and send one line: `Read <file> and do what it says.`

What a brief carries:

- **Unit** — the outcome in one sentence, PR kind and title, branch name,
  `Depends on #N` if stacked
- **Boundaries** — which worktree is theirs; the primary checkout and other
  worktrees are off-limits; the one-Grok-build-at-a-time rule
- **Harness** — follow `AGENTS.md`, `skills/session-unit`, and
  `skills/pr-authoring`; `gh` with `--repo` and a per-command token; push
  to `origin`
- **Language** — a delegated run answers in the language its brief *asks for*,
  and `AGENTS.md` alone is not reliable there: subagent runs from the same batch
  on 2026-09-25, with near-identical English briefs, came back 0%, 0%, 1%, 98%.
  Put the language instruction where that run cannot miss it — the brief itself.
  Write the brief in Korean with the technical detail inline, or open it with
  `보고·진행 메모는 한국어로` before switching to English. A delegated run's
  report is human-facing session text too.
- **Authority** — merging is part of the unit when the opening asked for the work
  to be carried through; say so in the brief, and say when it is withheld.
- **Report** — PR URL, test evidence, what was left out

Then mark the card so other sessions can see the state:
`orca worktree set --worktree "id:$WT_ID" --comment "<one-line state>" --json`

### Watching a dispatched unit — the spinner glyphs are not a contract

If you wait for the agent to finish, **do not decide it from a spinner
character**. Every glyph used here (`⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏`) is a frame index, not a state, and a
pattern that lists a few of them silently reports "idle" on the frames it did
not list.

**Measured 2026-09-25.** A monitor watched a `grok` tab and declared it done
because its glyph list missed the frame on screen — while the tab showed
`⠦ Thinking… 5.2s ⇣141k`, mid-task, and the agent went on to write its report.
The false positive arrived 10 minutes into the run.

What works instead:

```sh
orca terminal read --terminal "$H" --json | python3 -c "
import json,sys
d=json.load(sys.stdin)
tail = '\n'.join(d['result']['terminal']['tail'][-6:])
busy = ('Thinking' in tail) or ('Waiting for response' in tail) or ('stop]' in tail)
print('busy' if busy else 'idle')
"
```

Match the **words** the agent prints (`Thinking…`, `Waiting for response…`) and
the presence of the `[stop]` affordance — not the glyph. Then require the idle
reading twice in a row before declaring the unit finished, because a single
sample lands between frames.

**Better still: watch the artifact, not the screen.** A unit that writes files
is finished when the files change and stop changing; `git -C "$WT" status` is
harder to misread than a TUI. Use the screen to confirm, not to decide.




## 4. Merge and clean up

**This step is part of the unit** (AGENTS.md §One session, one unit). Once the
checks pass and the body meets the bar, merge and clean up in the same session —
do not leave a green PR for a later prompt.

```sh
GH_TOKEN="$(gh auth token --user <account>)" gh pr merge <n> --repo innocarpe/deepseek-build --merge --delete-branch
gh pr view <n> --repo innocarpe/deepseek-build --json state,mergeCommit   # state must read MERGED
git -C "$WT" status --short                          # must print nothing
orca terminal list --worktree "id:$WT_ID" --json      # see what is actually open
orca terminal close --worktree "id:$WT_ID" --all --json
orca worktree rm --worktree "id:$WT_ID" --json
```

`orca worktree rm` also deletes the local branch, but only when Orca can prove
it is merged. GitHub deletes the remote branch on merge (repo setting).
`--force` removes a dirty tree; use it only after reading what is dirty.

**`--all` closes the launcher and setup tabs too**, which is usually what you
want at the end of a unit — but read the list first if any tab might hold
unsent work.

Then refresh the tower — only when it is on `main` with no changes to tracked
files:

```sh
if [ "$(git -C "$TOWER" branch --show-current)" != main ]; then
  echo "tower is not on main; not pulling"
elif [ -n "$(git -C "$TOWER" status --porcelain --untracked-files=no)" ]; then
  echo "tower has uncommitted changes; not pulling:"
  git -C "$TOWER" status --short --untracked-files=no
else
  git -C "$TOWER" pull --ff-only origin main
fi
```

Untracked files do not block the pull, and they should not: OS or tool
markers (for example a Spotlight `.metadata_never_index`) would otherwise
stop every refresh. `pull --ff-only` already refuses by itself if an incoming
file would overwrite an untracked one.

Never remove a worktree another session created or is still using, even when
it looks finished — ask its session first.

## Anti-patterns

| Don't | Why |
|-------|-----|
| Edit or commit in the primary checkout | Tower sessions share its index; the change lands in someone else's diff |
| Create the worktree, then add an agent tab by hand when `--agent <id>` would have worked | Leaves `Terminal 1` and `Setup` tabs the unit never needed — three tabs for one job |
| Re-run `create` because you could not parse the handle | The side effect already happened; ask `terminal list` instead |
| `terminal send … --enter` right after `tui-idle` without reading the screen | A first-run dialog swallows the brief and exits the agent |
| `cd "$WT"` and keep working | The next command meant for the tower runs in the tree, or the reverse |
| `gh auth switch` | Flips the active account for every session on the machine |
| Two vendored Grok builds at once | 30–60+ min cold each; parallel builds starve each other |
| `--name feat/x` | Becomes `feat-x`; name the worktree by slug and rename the branch |
| `worktree rm --force` on a tree you did not read | Deletes uncommitted work with no undo |
