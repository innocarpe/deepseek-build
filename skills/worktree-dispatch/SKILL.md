---
name: worktree-dispatch
description: "From the control-tower checkout: one Orca worktree per unit, then merge and clean up. Launch grok or dsb with orca-tab, not the orca manual."
---

# Worktree dispatch (control-tower checkout)

The primary checkout is the control tower: it stays on `main`, clean, and
directs work ([AGENTS.md](../../AGENTS.md) §Control-tower checkout). Code
changes happen in Orca worktrees — **one worktree = one branch = one PR.**

This skill is the worktree lifecycle. Launching the agent tab — grok, codex,
claude, or dsb — is [`orca-tab`](../orca-tab/SKILL.md). Do not run
`orca skills get orca-cli` to open a tab or launch an agent. If a flag in
either skill is rejected, read only that command's `--help` and retry once.

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

- **Never decide another worktree is abandoned from a tool that only sees Orca
  tabs.** `orca terminal list --worktree <id>` and the whole-list `worktreeId`
  match both returned **0 terminals** for a worktree whose agent was mid-build
  at that moment (2026-09-25, Orca `1.4.210`): that session ran in a plain
  terminal on another tty, not an Orca tab. A zero there means *you cannot see
  it*, not *nobody is there*. Reading activity from it has already produced a
  wrong report that another session's in-flight work was abandoned.

  Use a signal that reads the machine, in this order:

  | Signal | Reads | Weakness |
  |---|---|---|
  | `pgrep -fl "/deepseek-build/<slug>"` | live processes whose argv or cwd names the tree | an idle agent waiting on the model may hold no matching child |
  | `git -C "$WT" log -1 --format=%ad` + `git -C "$WT" status --short` | last commit time and uncommitted work | a session can be thinking for minutes with no new commit |
  | `orca worktree list … .workspaceStatus` | Orca's own card state | reads `in-progress` even for trees nobody has touched since creation |
  | `orca terminal list --worktree <id>` | Orca tabs only | **misses sessions outside Orca** |

  **Treat "no signal" as unknown, not as idle.** When two of the four disagree,
  or when the last commit is recent, **ask the owning session** — a question
  costs a minute; deleting or rebasing under a live session destroys work with
  no undo. This is the same fail-close rule as everywhere else in the harness:
  an unmeasurable state is not a passing one.

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

**This create does not launch an agent.** To open grok, codex, claude, or dsb,
follow [`orca-tab`](../orca-tab/SKILL.md) — its §2 or §3 includes the create.
Do not run the command above and then add an agent tab on top. For grok,
codex, and claude that one-step launch is one tab. Creating here, letting
hooks run, and then adding an agent by hand is three. dsb cannot use
`--agent`, so `orca-tab` §3 is the create and §1 is the tab.

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

## 3b. Open the agent session

Follow [`orca-tab`](../orca-tab/SKILL.md). Do not restate its commands here,
and do not run `orca skills get orca-cli` to fill them in.

- grok, codex, claude: `orca-tab` §2. One create. Do not also run §1.
- dsb / deepseek-build: `orca-tab` §3, then §1 with `--command dsb`.
  `--agent dsb` is rejected.
- A tab in a worktree that already exists: `orca-tab` §1.

What a brief carries:

- **Unit** — the outcome in one sentence, PR kind and title, branch name,
  `Depends on #N` if stacked
- **Boundaries** — which worktree is theirs; the primary checkout and other
  worktrees are off-limits; the one-Grok-build-at-a-time rule
- **Harness** — follow `AGENTS.md`, `skills/session-unit`,
  `skills/orca-tab`, and `skills/pr-authoring`; `gh` with `--repo` and a
  per-command token; push to `origin`
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
it looks finished — ask its session first. If "it looks finished" came from
`orca terminal list` showing nothing, re-read §0: that command returned 0 for a
worktree whose agent was mid-build (2026-09-25).

## Anti-patterns

| Don't | Why |
|-------|-----|
| Edit or commit in the primary checkout | Tower sessions share its index; the change lands in someone else's diff |
| Create the worktree, then add an agent tab by hand when `--agent <id>` would have worked | Leaves `Terminal 1` and `Setup` tabs the unit never needed — three tabs for one job |
| Re-run `create` because you could not parse the handle | The side effect already happened; ask `terminal list` instead |
| `orca skills get orca-cli` before launching grok or dsb | The recipes are in `orca-tab`. The full guide is what stalls the turn |
| `terminal send … --enter` right after `tui-idle` without reading the screen | A first-run dialog swallows the brief and exits the agent (`orca-tab` §4) |
| `cd "$WT"` and keep working | The next command meant for the tower runs in the tree, or the reverse |
| `gh auth switch` | Flips the active account for every session on the machine |
| Two vendored Grok builds at once | 30–60+ min cold each; parallel builds starve each other |
| `--name feat/x` | Becomes `feat-x`; name the worktree by slug and rename the branch |
| Report a worktree as "no active session" from `orca terminal list` alone | It returned 0 for a live, mid-build worktree (§0); that report tells the owner to abandon in-flight work |
| `worktree rm --force` on a tree you did not read | Deletes uncommitted work with no undo |
