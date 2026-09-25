# What changes in DeepSeek Build `6.0.0`

**The one file.** If you read nothing else about this major, read this: what
`6.0.0` improves, what it fixes, what it newly does, and what it deliberately
does not take — against `5.6.0`.

**Why there is a major at all.** The product is built on open-source **Grok
Build**, vendored under `third_party/grok-build/` and pinned to a revision. The
pin had been sitting on Grok Build **`1.0.0`** (2026-08-09) while upstream
reached **`1.0.41`** (2026-09-22) — **41 releases**, 25 sync commits, 3,587
files. `6.0.0` moves the base across that whole gap and re-derives this
product's own overlay on top of it.

**Measured, not asserted:** the gap was 472 upstream changelog bullets, of which
36 were performance work, 5 were breaking changes, and 19 described xAI-hosted
surface with no meaning here. Everything below is a dsb-facing reading of that
set; the ledger of verdicts is
[UPSTREAM_SYNC_LEDGER.md](./UPSTREAM_SYNC_LEDGER.md), and the per-release matrix
is [research/grok-build.md](../research/grok-build.md).

---

## 1. Faster

The largest cluster of upstream work in this window was latency, and almost all
of it is felt on every session.

| What | Before → after |
|---|---|
| **Starting a session** | Remote settings were fetched up to twice per boot, then MCP servers were connected before the prompt was usable. They now cache, connect in the background, and a new session returns before that work finishes. |
| **First message in a large repository** | Waited on a full repository status scan. That scan moved off the critical path. |
| **Resuming a large session** | Replay of a long transcript blocked the UI — you could watch it fill in and it was not usable until done. Now significantly faster. |
| **Git status and diff on large repos** | High CPU and memory use, to the point of unresponsiveness on big histories. Both are bounded now. |
| **Subagent spawn** | Spawning stalled when many sessions existed, froze the parent while child history loaded, and triggered rate-limit bursts when several started at once. All three are addressed. |
| **Long sessions with many subagents** | History search leaked background threads; finished child transcripts were held in memory forever. Threads are contained and transcripts are evicted, then rebuilt from disk when reopened. |
| **Syntax highlighting** | Uses far less memory and runs faster on large files. |
| **`tmux` panes** | Lagged from per-frame synchronized-update wrapping, which is now omitted. |

## 2. Fixed — the ones you were most likely to hit

**Losing work or state**

- A session interrupted by a crash dropped the turn silently. It now shows a clear marker.
- Long sessions that hit an expired token during network trouble could lose work; auth recovery no longer fails fast on transient errors.
- Session data is written more reliably after prompts and on power loss.
- Long sessions no longer lose older compaction checkpoints or prompt offloads to the 30-day cleanup sweep, and `/rewind` no longer fails when an older checkpoint is gone.
- Prompt text typed while the agent was starting, or while a plan was generating, was lost. It is preserved.

**Turns failing when they should not**

- Length-truncated responses now continue automatically instead of failing the turn.
- Transient inference failures (stalls, drops, 5xx) retry instead of ending the turn, and retry status now says *why* in the composer and title.
- Truncated tool calls execute when their arguments are complete, instead of failing the turn.
- Tool timeouts no longer hang the agent when a child process is stuck or holding pipes open.
- Compaction reports the real error instead of a generic one, degrades input on context overflow, and retries TPM rate-limit errors rather than mistaking them for overflow.

**Real bugs that made the UI wrong**

- Pasted images could attach the *wrong* image on macOS.
- Images larger than 2000px could brick a session on a many-image request, and a server-rejected image could poison a session permanently. Both are healed.
- A shell command was *cancelled* when you sent a new message mid-run; it now moves to the background.
- Permission rules for Bash commands using quoted filename variables always prompted instead of respecting the configured rule.
- Auto mode instantly ran destructive `git checkout --` commands; those now go through the model.
- Esc used to cancel a running turn — one keystroke away from losing work. It now reminds you to use Ctrl+C.
- Worktree sessions lost their branch/status in the status bar, on directory switch, on resume, and when opening the dashboard.
- Copy-paste broke on wrapped table cells, wrapped URLs, CJK edges, and `--minimal` mode.
- Markdown tables, strikethrough, headings, and wrapped tables now render correctly; heading colors were missing entirely.
- `/copy` now preserves markdown formatting instead of flattening it.

**Startup failures**

- Session startup could hang on large or unhealthy git repositories, and could freeze forever when `.envrc` evaluation blocked.
- Startup hangs on slow networks while fetching remote settings are gone.

## 3. New at your fingertips

**The status line.** A configurable bottom row can show live session info or the
output of your own script, refreshed on a timer (`refresh_interval`). It appears
in minimal mode as well as fullscreen.

**Prompt queuing and steering.** Queued messages no longer auto-submit while you
are still editing them; a follow-up behavior setting lets them send immediately
as interjections; interjections during a turn are delivered atomically or not at
all; and sending a message while waiting on a subagent works instead of failing.

**Modal session info.** `/usage`, `/session-info` and `/context` open in a
tabbed modal rather than dumping text into the conversation, with click-to-copy
rows and drag-select.

**Drafts and stashing.** Ctrl+S stashes the current prompt draft so you can send
something else and restore it; Ctrl+Z right after stashing restores it.

**External editing and history.** `/edit-prompt` opens your `$EDITOR` from the
full TUI, not just minimal mode. History shows every command, slash command and
memory note in the order you typed it, and Up-arrow on an empty prompt reaches
queued follow-ups first.

**Subagents you can actually watch.** Subagent lifecycle events survive
out-of-order delivery, `[stop]` in a drill-in view stops *that* child rather
than the root session, background children report real output instead of a
pointer, and a subagent's model and retry settings are respected.

**Workflows.** A Workflows catalog (Ctrl+L or `/plugins`), `/workflows` and
`/workflow runs` for live runs, `--agent-budget` and `--effort` on `/workflow`,
and workflow rows in the command palette.

**Session search and recovery.** The search index can be disabled per host
(`GROK_SESSION_SEARCH`), headless sessions are browsable without polluting
default history, and resume tells the model what loops, subagents and workflows
were still running.

**Config surface.** `GROK_CONFIG` / `GROK_CONFIG_PATH` let launchers override
settings without editing `config.toml`; the default permission mode for new
interactive sessions is configurable; web search can be restricted to allowed or
excluded domains.

**MCP.** Servers can ask for form input or URL consent through the normal
popup, re-authentication states are visible even when no tools are listed,
transient connection failures retry, and built-in tools win tool-name collisions
against user MCP servers.

**Hooks.** Hooks can ask you to confirm a tool call rather than only
allow/deny, can rewrite a tool's input before it runs, and can add context shown
to the model after a tool runs. Each `SessionEnd` hook now has its own timeout
so one slow hook cannot delay session close.

## 4. Changed — behavior that is deliberately different

These come from upstream's breaking changes; each was reviewed for what it means
here rather than absorbed silently.

| Change | Effect in dsb |
|---|---|
| `/rewind` truncates conversation history only, and confirms by default | Inherited. It no longer touches files, which is the safer default. |
| `spawn_subagent` no longer accepts `capability_mode` | **Landed with a docs change.** Tool access is controlled by agent type now; the subagent contract and its brief were updated to match. |
| Scheduled `/loop` tasks always run in the background | Inherited. They no longer inject turns into your conversation. |

## 5. What `6.0.0` does **not** take

Upstream is a different product with a different vendor behind it. These were
read, judged, and left — not overlooked.

| Left out | Why |
|---|---|
| **Telemetry, consent notices, xAI login/OIDC/team policy, billing and credit upsell, `grok` self-update channels** | xAI-hosted surface with no meaning in a DeepSeek product. The local default path stays DeepSeek-seeded and telemetry-off. |
| **Voice, video generation, ZDR storage prompts, image generation** | Tied to xAI media models and endpoints. |
| **Desktop app / Computer Hub daemon** | Not this product's shape; it is a terminal harness. |
| **Enterprise `requirements.toml` model restriction** | A hosted-policy feature; there is no equivalent policy server here. |
| **`grok clone` content-store / projected working tree** | A new git-object storage model. Interesting, but it changes how worktrees are created and stored, which is L3 machinery this product already covers with Orca worktrees. Revisit if upstream's model proves materially faster. |
| **`x.ai/*` RPC method names, `GROK_*` env vars and `~/.grok` paths** | Branding. Where an upstream *behavior* was worth having, the behavior was taken and re-pointed at DeepSeek paths and names. |

**One thing this major does not attempt:** the 472 upstream items include work
that only pays off with a live hosted backend (multi-device remote control,
managed policy, marketplace installs). Those are recorded as such in the ledger
rather than claimed as improvements.

---

## Where to look next

| For | Read |
|---|---|
| The verdict on every upstream item | [UPSTREAM_SYNC_LEDGER.md](./UPSTREAM_SYNC_LEDGER.md) |
| The per-release adoption matrix | [research/grok-build.md](../research/grok-build.md) |
| How a sync is performed | [grok-sync-runbook.md](../contributing/grok-sync-runbook.md) · [skills/grok-sync](../../skills/grok-sync/SKILL.md) |
| Why the base is vendored at all | [ADR-0008](../adr/0008-grok-build-base.md) |
| The release notes | `CHANGELOG.md` and the GitHub release for `v6.0.0` |
