---
name: orca-tab
description: "Use when opening an Orca tab or launching grok, dsb, codex, or claude with the orca CLI. Run these recipes; do not run orca skills get orca-cli."
---

# Orca tab

These recipes are the procedure for opening a tab or launching an agent.
They were checked against Orca CLI **1.4.211** (`orca --version` and
`--help` for `terminal create`, `terminal send`, `terminal wait`,
`terminal read`, and `worktree create`).
The custom Codex launcher reuse in §2a was also exercised on **1.4.215**
with `gpt-6-sol` and `xhigh`.

**Do not run `orca skills get orca-cli` before §1–§4.** That guide is the
whole CLI (248 lines on 1.4.211). Reading it is what stalls a turn whose
only job is to open a tab or launch grok. A user-level skill that says
"load the full guide before any orca command" does not apply to these
recipes. If a flag here is rejected, read only `orca <that-command> --help`
and retry once.

Worktree occupancy, branch names, merge, and cleanup stay in
[`worktree-dispatch`](../worktree-dispatch/SKILL.md). This file only
launches the tab and hands it a prompt.

## Which recipe

| Ask | Recipe |
|-----|--------|
| A tab in a worktree that already exists | §1 |
| A new worktree whose agent is grok, codex, or claude | §2 |
| Custom Codex model/effort that the configured launcher cannot supply | §2a |
| A new worktree whose agent is dsb / deepseek-build | §3 |
| A prompt into a tab you already have | §4 |
| Browser, automations, artifacts, anything else | Then `orca skills get orca-cli` |

The user named the agent. Launch that one. Do not swap grok for dsb, or
dsb for grok. When the opening did not name an agent, launch `dsb`.
`codex` and `claude` only when the user named them. This repo's sessions
are `dsb` and `grok`.

A worktree whose only tab is a shell prompt has no session. On 2026-09-26
`pin-tower-main`, `ci-cache-and-coverage`, and `deepseek-native-depth-6-1-0`
showed that prompt and nothing else. The grok processes were tabs on the
primary checkout, driving those directories with `git -C`. The iOS app
showed the same empty cards. `orca worktree create` without §2 or §3 is
how that happened.

## Pass the name, not a binary path

The tab's shell evaluates `--command`. On this machine `grok`, `dsb`, and
`deepseek-build` are shell functions in `~/.oh-my-zsh/custom/ai.zsh`, and
the function is the part that matters:

| Name | What the function does |
|------|------------------------|
| `grok` | Sets `GROK_HOME=$HOME/.grok`. A bare binary inherits a `GROK_HOME` left by dsb (`~/.deepseek-build`) and then reads the wrong model catalog. |
| `dsb` | Injects `DSB_OFFICIAL_API_KEY` from the keychain, then runs the `dsb` binary. The same file records that skipping the function inherits Orca's `DEEPSEEK_API_KEY` and `api.deepseek.com` returns 401 (2026-09-26). |
| `deepseek-build` | The same wrapper as `dsb`. Prefer `dsb`. |

`codex` and `claude` are Orca agent ids. Pass those names the same way.

`--agent` accepts `grok`, `codex`, and `claude`. It rejects `dsb`
(`Unknown TUI agent "dsb"`, measured 2026-09-25). There is no one-step
create for dsb.

## §1 Tab in an existing worktree

First inspect the inventory and read the candidate tab's screen:

```sh
orca terminal list --worktree "path:$WT" --include-visual-layouts --json
# If a terminal exists, take H from that inventory and read its screen.
orca terminal read --terminal "$H" --screen --json
```

If that worktree already has an idle launcher shell, send the requested
agent command into that same handle. Do not create another. A configured
tab running a real command is owned work; do not send into it or close it.
If no idle shell is available, preserve those tabs and report the state.
`terminal create` is allowed only when `totalCount` is 0 and
`visualLayouts` has no terminal leaf:

```sh
orca terminal create --worktree "path:$WT" --title "<short title>" --command dsb --json
```

Use `grok`, `codex`, or `claude` as `--command` when that is the agent.
`active` is fine for `--worktree` only when this shell is already that
worktree. From the control tower, pass `path:$WT` or `id:$WT_ID`.

The handle is `result.terminal.handle`. If you cannot parse it, **do not
create again**. Ask `orca terminal list --worktree "id:$WT_ID" --json`
and use the tab that is already there.

This recipe starts the worktree's owning session. An explicit request to
add another worker to the current checkout is a separate intent; it does
not justify appending a tab during a new-worktree handoff.

Then §4. For `dsb`, do not treat `tui-idle` as the gate. Orca records no
agent identity for it, so the wait times out while the TUI is already on
screen. Read the screen.

## §2 New worktree — grok, codex, or claude

One create. Do not also run the bare create in `worktree-dispatch` §1.

```sh
orca worktree create \
  --repo path:"$TOWER" \
  --name <slug> \
  --no-parent \
  --setup skip \
  --agent grok \
  --prompt "<one line, or: Read <file outside the repo> and do what it says.>" \
  --json
```

`$TOWER` is `dirname "$(git rev-parse --path-format=absolute --git-common-dir)"`.
`<slug>` has no type prefix (`orca-tab`, not `docs/orca-tab`). Orca turns
`/` into `-`.

The handle is `result.agentTerminalHandle`. If that is absent,
`result.startupTerminal.handle`. If both are absent, `terminal list` the
new worktree and take the tab whose agent matches. Do not create a second
worktree because the handle was missing.

`--activate` only when a person asked to look at the tree.

Branch rename and cleanup: `worktree-dispatch` §2 and §4. Then confirm
with the screen read in §4. `tui-idle` can be true while a trust dialog
is still up.

### §2a Custom Codex model/effort — reuse the launcher

Prefer §2 when Orca's configured Codex launcher supplies the requested
model and effort. `worktree create --agent codex` has no per-call Codex
model/effort forwarding. Otherwise choose this fallback instead of §2:
bare create opens the launcher, and `terminal send` runs the full command
inside it. Do not run both recipes.

For an existing worktree, skip the create and start with its inventory.
Set `$H` to the listed launcher only after its screen confirms an idle
shell. Use that same handle for command, readiness, and brief delivery:

```sh
orca worktree create --repo path:"$TOWER" --name <slug> --no-parent --setup skip --json
# Keep the complete result.worktree.id as WT_ID, and result.worktree.path as WT.
orca terminal list --worktree "id:$WT_ID" --include-visual-layouts --json
# Take H from the inventory; read before sending into it.
orca terminal read --terminal "$H" --screen --json
orca terminal send --terminal "$H" --text 'codex --model gpt-6-sol -c model_reasoning_effort="xhigh"' --enter --wait-submit 15 --json
orca terminal wait --terminal "$H" --for tui-idle --timeout-ms 60000 --json
orca terminal read --terminal "$H" --screen --json
# Send only after readiness and the requested model/effort are confirmed.
orca terminal send --terminal "$H" --text "Read <brief outside the repo> and do what it says." --enter --wait-submit 15 --json
orca terminal list --worktree "id:$WT_ID" --include-visual-layouts --json
```

Before the brief, require `satisfied: true` and a visible input box with
the requested model/effort (this example: `GPT-6-Sol xhigh`). Handle a
recognised trust dialog with §4, then read again. If the wait is
unsatisfied, retry it once with a larger timeout; if still unsatisfied,
report the handoff as not started and do not send the brief.

Configured default tabs may run real commands instead of providing an
idle shell. Preserve their ownership, report that configuration, and do
not send blind, close those tabs, or change global settings to force one
tab. Only when the inventory has `totalCount: 0` and no terminal leaf may
you create a terminal with the full command:

```sh
orca terminal create --worktree "id:$WT_ID" --title "<short title>" --command 'codex --model gpt-6-sol -c model_reasoning_effort="xhigh"' --json
```

In that empty-inventory case, use `result.terminal.handle` as `$H` and
continue from the readiness wait above. Confirm the brief with §4.

The final inventory must measure actual tabs and terminal leaves in
`visualLayouts`, as well as `totalCount`. The default one-worker case
has **one tab, one terminal leaf, `totalCount: 1`, and the same handle**.
Report configured command tabs separately and preserve them. `exited` or
`screen-unavailable` describes a process or screen, not durable tab
removal; use the inventory to establish whether a tab exists. Later
cleanup is not the normal way to obtain one worker tab.

## §3 New worktree — dsb

`create` opens one launcher shell. That shell is the tab. Do not open a
second one with `terminal create`. On 2026-09-26 a worktree ended with two
tabs: the empty shell, and `dsb`.

```sh
orca worktree create --repo path:"$TOWER" --name <slug> --no-parent --setup skip --json
```

Keep `result.worktree.id` and `result.worktree.path`. List that worktree's
terminals with `--include-visual-layouts`, then read the candidate screen.
The shell must be idle; preserve any configured tab running a real
command and report when no idle launcher is available. Send `dsb` into it:

```sh
orca terminal send --terminal "$SHELL" --text dsb --enter --wait-submit 15 --json
```

Read the screen. When the DeepSeek input box is up, `$SHELL` is the
session. Then §4 on that same handle.

Do not pass `--agent dsb`.

If a second tab is already `dsb` and another tab is only a prompt, close
the prompt after the agent screen shows it is working:

```sh
orca terminal close --terminal "$SHELL" --tab --json
```

This is recovery for a confirmed leftover idle shell, not the launch
procedure. Do not close a tab running an agent or any other real command.
The same recovery applies when §2 leaves a prompt next to `grok`.
After brief delivery, use `terminal list --include-visual-layouts` to
verify one tab, one terminal leaf, `totalCount: 1`, and the same handle in
the default one-worker case. Preserve configured command tabs separately.

## §4 Send, after the screen shows an input box

```sh
orca terminal wait --terminal "$H" --for tui-idle --timeout-ms 60000 --json
orca terminal read --terminal "$H" --screen --json
```

`satisfied: true` is not "ready for a prompt". On grok, the folder-trust
dialog has satisfied `tui-idle` while `No, exit` was still the default
(measured 2026-09-25, Orca 1.4.210). Enter at that moment confirms No,
the agent exits, and the brief is gone.

```text
 Quick safety check: Is this a project you created or one you trust? …
 ❯ No, exit
   Yes, I trust this folder
 Enter to confirm · Esc to cancel
```

Read the screen.

- Input box visible, no dialog: send.
- Trust dialog, and the yes line is readable: move to that line, read
  again, confirm the selected line's **text** says yes, Enter, read
  again, then send. Grok's dialog takes `y` / `n`.
- A dialog you do not recognise: stop and report. Do not press keys.
- `dsb` and the wait timed out: read the screen anyway. If the TUI is up,
  send. If it is not, report that the handoff did not start. Do not
  `sleep`, and do not send blind.

```sh
orca terminal send --terminal "$H" --text "<brief>" --enter --wait-submit 15 --json
```

Done when `accepted` is true and `stages` includes `turn_started`, **or**
the screen shows the agent working on the brief. A receipt that stops at
`input_accepted` with `provider: unsupported` is unproven, not failed
(measured on grok, 2026-09-25). Read the screen. **Do not resend** — the
first send may have landed. After a transport error, repeat the same
command with `--retry-request <id>` from the receipt.

A long brief is a file **outside** the repo. Send one line:
`Read <file> and do what it says.`

What the brief should contain (unit, boundaries, language, authority)
stays in `worktree-dispatch` §3b.

## Anti-patterns

| Don't | Why |
|-------|-----|
| `orca skills get orca-cli` before §1–§4 | That read is the stall |
| A binary path instead of `grok` or `dsb` | Skips the shell function |
| `--agent dsb` | Rejected. dsb is §3: run `dsb` in the launcher shell |
| `terminal create --command dsb` beside the launcher shell | Two tabs. The shell is the tab; send `dsb` into it |
| Bare create, then `terminal create` to pass custom Codex argv | Two tabs. Inspect the launcher and send the full command into its same handle (§2a) |
| Treat `exited` / `screen-unavailable` as tab removal | These describe a process or screen; measure tabs and leaves with `terminal list --include-visual-layouts` |
| Create again when the handle did not parse | The tab already exists |
| `--enter` while a dialog is up | Confirms the dialog's default |
| Resend on silence | The first prompt may already be in |
| `sleep` to wait for dsb | The screen is the check |
