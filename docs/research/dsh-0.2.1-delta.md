# DeepSeek Harness (dsh) 0.2.1 delta — what moved since 2026-10-01

**Nature:** Non-binding evidence ([SOURCES.md](../product/SOURCES.md) precedence).
A judgment table, not a product commitment. Promote an item into
`docs/specs/`, `docs/product/`, or an ADR before building it.

**Question asked:** the earlier dsh note read `0.1.7-rc.2` and the
[upstream sync ledger](../product/UPSTREAM_SYNC_LEDGER.md) read `0.2.0-rc.2`.
What changed in `0.2.1`, and which changes matter to DeepSeek Build?

**Subject:** [`deepseek-ai/deepseek-harness`](https://github.com/deepseek-ai/deepseek-harness)
at `687f282e86` (= `dsh-v0.2.1-rc.2`, commit date 2026-10-17 15:22 +0800),
read 2026-10-22 from the shallow clone at `OpenSources/_upstream/deepseek-harness`.
The earlier notes are [dsh-deepseek-harness.md](dsh-deepseek-harness.md)
(`477b4f4` = `0.1.7-rc.2`, read 2026-09-25) and the ledger's Sync 3 entry
(`639ed015` = `dsh-v0.2.0-rc.2`, commit date 2026-09-29, read 2026-10-01).

**Method.** The window is `git log 687f282e86 --since=2026-10-01`, filtered by
package path to count commits. The design notes and diffs behind the items in
the two judgment tables were read at the subject commit; the rest is the
package-level log. This is **not** a file-by-file survey: the window changes
8,431 files, and the `.agents/notes` tree alone contributed 1,983 of them.
Every number below was re-run for this note; where a figure could not be
reproduced, the reproduced value is used.

---

## 1. Scale

| Measurement | Value |
|---|---|
| Non-merge commits since 2026-10-01 | 1,019 |
| Merge commits in the same window | 325 |
| Total commits reached | 1,344 |
| Files changed, `639ed015..687f282e86` | 8,431 |
| Lines, same range | +320,407 / −84,704 |

Top directories by changed files in `639ed015..687f282e86`:

| Directory | Files |
|---|---|
| `.agents/notes` | 1,983 |
| `packages/client` | 1,157 |
| `packages/experimental` | 817 |
| `snapshots/session` | 526 |
| `apps/web` | 219 |

Non-merge commits touching the core packages (a commit can touch several):

| Package | Commits |
|---|---|
| `packages/boot` | 63 |
| `packages/llm` | 58 |
| `packages/core` | 54 |
| `packages/session` | 47 |
| `packages/subagent` | 47 |
| `packages/fs` | 22 |
| `packages/shell` | 22 |
| `packages/plan` | 12 |

Commands (run in the clone, 2026-10-22):

```bash
git rev-list --count 687f282e86 --since=2026-10-01                     # 1344
git rev-list --count --no-merges 687f282e86 --since=2026-10-01         # 1019
git rev-list --count --merges 687f282e86 --since=2026-10-01            # 325
git diff --shortstat 639ed015 687f282e86                               # 8431 files ...
git diff --name-only 639ed015 687f282e86 | cut -d/ -f1-2 | sort | uniq -c | sort -rn
git log 687f282e86 --since=2026-10-01 --no-merges --format=%H -- packages/<name> | wc -l
```

The changed-file mass is concentrated in notes and client code. The eight core
packages carry the commit counts above; those counts overlap where one commit
touches several packages, so they do not add up to the 1,019 non-merge commits.
That shape is why this note samples design notes and package logs instead of
reading the diff.

---

## 2. Judgment table

| Item | Upstream evidence | dsb today | Verdict |
|---|---|---|---|
| **DeepSeek Search and Fetch APIs** | `fda98dadd2` (2026-10-17) | `web_search` is deferred in [spec 40](../specs/40-core-tools-surface.md); no fetch tool | **Take** — a provider behind an existing contract, not a new tool |
| **Minimum reasoning for auxiliary requests** | `ffb88d331e`, `9e36f20675`, `80fd1f0b19` (2026-10-06..07) | Session-title request sends no effort; the route default applies | **Take (small)** — set the floor, then measure |
| **Subagent preset selection and restoration** | `a9afe76200`, `648ab5167e` (2026-10-09) | Spec 60 has two fixed kinds; no preset concept | **Take (later)** — needs a spec 60 revision |
| **Exhausted delegation depth in runtime context** | `42db4b8541` (2026-10-11) | No depth cap at all; only `max_rounds` | **Take (conditional)** — depends on the row above |
| **End the turn instead of sleeping; read line ranges** | `7a881e944a` (2026-10-12) | Vendored prompt has no equivalent sentence | **Take (prompt overlay)** — smallest change here |

### 2.1 DeepSeek Search and Fetch APIs

Evidence: `fda98dadd2` and its design note
`.agents/notes/implemented/feature/2026-10-15-deepseek-search-fetch-api.md`.

dsh adds direct retrieval as an opt-in setting, **off by default**. Turning on
"Use the new Web Search API" writes both `web.searchProvider: deepseek-search-api`
and `web.fetchProvider: deepseek-fetch-api`; turning it off restores Messages
search with local HTTP fetch. The two APIs share one credential and one base,
`https://api.deepseek.com/v1/tools`, and append `/search` or `/fetch`. Messages
search keeps a different base (`.../anthropic/v1`) and stays the default until
the launch is announced. The routing change sits behind the existing
`web_search` and `web_fetch` tools, so the model-facing schema does not change.
Public routes started accepting DeepSeek account tokens on 2026-10-16.

The design note records the costs and limits: CNY 5 per 1,000 successful
requests with results, independent of result count; each API is limited to 5 QPS
per account, shared across that account's API keys; Search sends `query`,
`count`, `need_content: false`, and `ori_question`. `ori_question` is the
initiating session's latest human prompt, whitespace-collapsed and cut to the
API's 128-character limit.
The note's own probes found that different `ori_question` values left result
order, scores, and summaries unchanged. Fetch returns Markdown without a second
Harness summarization request, and the provider reports HTTP 200 for a
successful retrieval because the shared result has no target-status field.

Live wire check (2026-10-22, run by the supervising session with the
maintainer's key; two calls, key not recorded, not re-run for this note):

| Request | Result |
|---|---|
| `POST https://api.deepseek.com/v1/tools/search` | 200; `results[]` carried `title`, `url`, `snippet`, `date`, `summary` |
| `POST https://api.deepseek.com/v1/tools/fetch` | 200; `{title, content}` with Markdown content |

The raw Fetch response carries `title`; the dsh provider drops it because its
shared fetch result has no title field. That is a Harness mapping limit, not an
API limit.

dsb today has no web retrieval tool. [Spec 40](../specs/40-core-tools-surface.md)
lists `web_search` as deferred and does not list a fetch tool. The vendored xAI
`web_search` is a backend-hosted tool on the Responses API — the hidden default
resolver sets `api_backend: ApiBackend::Responses`
(`third_party/grok-build/crates/codegen/xai-grok-shell/src/agent/config.rs:4774`,
asserted in `agent/config_tests.rs:447`) — so the Chat Completions path that dsb
uses does not expose it.

**Recommendation for a port:** keep `ori_question` off by default. It sends up
to 128 characters of the user's prompt to the search service on every query,
and upstream measured no result difference. Everything else maps onto the
existing tool contract, which makes this a spec change first (un-defer
`web_search`, define `web_fetch`) rather than a new tool.

### 2.2 Minimum reasoning for auxiliary requests

Evidence: `ffb88d331e`, `9e36f20675`, `80fd1f0b19` (2026-10-06..07).

dsh added a `reasoning.floorEffort` to exact-model metadata and lets the caller
select it. The session-title service now dispatches at the route's floor instead
of the route's default. Two adapters share the rule: `llm-pi-ai` and `llm-deepseek`
send the effort they receive and no longer special-case the `session-title`
purpose. The motivation is concrete: title calls carry a 64-token output cap,
and on an adaptive-thinking route whose default is high, thinking consumed the
cap and the session kept its fallback title.

dsb's title request builds a `ConversationRequest` without setting
`reasoning_effort`
(`third_party/grok-build/crates/codegen/xai-grok-shell/src/session/helpers/session_summary.rs:196`),
while the field exists and is optional
(`third_party/grok-build/crates/codegen/xai-grok-sampling-types/src/conversation.rs:658`).
The route or model default therefore applies. A floor would be a small change
in the same file.

**The cost saving is not measured.** dsh's note reports tests and a snapshot
showing a low effort selected from a high-default route; it reports no token or
latency delta. Treat "cheaper" as a hypothesis until a billed comparison runs.

### 2.3 Subagent preset selection and restoration

Evidence: `a9afe76200` and `648ab5167e` (2026-10-09), with notes
`.agents/notes/implemented/feature/2026-09-20-spawn-agent-preset-selection.md`
and `.agents/notes/implemented/simplification/2026-10-09-preset-owned-subagent-composition.md`.

A spawned child can select a named Agent Preset that owns its persona and tool
plugins. Omission joins the parent's live preset revision. An independent
selection records an `agent-preset/selected` event before the child's first
`subagent/descriptor`; cold restoration uses that ordering to tell an
independent selection from an inherited one, and rejects a missing projection
instead of silently joining the parent. In the same window, new requests stopped
accepting per-child `persona` and `toolFilter` fields — those compose through
the preset — while released descriptors keep their fields for cold resume.

dsb's [spec 60](../specs/60-subagents.md) defines two worker kinds
(`explore`, `implement`) and an in-process synchronous worker
(`crates/dsb-agent/src/subagent.rs`). There is no composition object to select.
The reusable idea is "a child is a named composition", not the exact preset
machinery; adopting it requires a spec 60 revision that names what a composition
owns.

### 2.4 Exhausted delegation depth in runtime context

Evidence: `42db4b8541` (2026-10-11).

When the calling agent reaches a visible tool's numeric depth cap, dsh appends a
required runtime-context notice — "current 1, maximum 1, remaining levels 0; do
the work directly". The cap and depth are resolved per request, so a resumed or
reconfigured agent stays correct. The system prompt and tool schemas do not
change, so the cached prefix survives.

dsb has no depth cap: [spec 60](../specs/60-subagents.md) §2 makes unbounded
nested trees a non-goal, and the worker tool exposes `max_rounds` only
(`crates/dsb-tools/src/tools.rs:874`). This row only becomes actionable together
with §2.3.

### 2.5 End the turn instead of sleeping; read line ranges

Evidence: `7a881e944a` (2026-10-12).

Two prompt edits in one commit:

- Delegation guidance gained: "When only waiting for their results remains, end
  your turn instead of sleeping or polling; the next result starts a new turn."
- Read guidance now names `sed`, `head`, and `tail` alongside `cat`, and says
  `offset`/`limit` read a line range rather than only continuing a large file.

dsb's full-screen agent runs the vendored prompt at
`third_party/grok-build/crates/codegen/xai-grok-agent/templates/prompt.md`;
that file instructs launching `task` calls but carries no sentence about ending
the turn while waiting. dsb's native `read` tool already accepts a line range
(`start_line`/`end_line`, `crates/dsb-tools/src/tools.rs:709`), so the read half
is a wording change.

This is the smallest item in the table and the only one that can ship as a
carried prompt patch without a spec change.

---

## 3. Hold

| Item | Evidence | Why hold |
|---|---|---|
| Working-directory centralization | `79bd3d8da7` (2026-10-06; 497 files, note `2026-09-13-session-working-directory`) | dsh moved the session's working directory and directory context into a central service with its own persistence record. dsb's cwd is part of the stable prefix ([spec 10](../specs/10-cache-contract.md)) and one session belongs to one worktree; moving it is a prefix-contract decision, not a port |
| Session format status and deferred migration | `a2dd724c60` (2026-10-08; note `2026-08-31-released-session-format-migrations`) | Exposes format status to the frontend and defers migration triggers. dsb's session replay is its own format ([spec 100](../specs/100-sessions.md)); a status surface is a compatibility project with its own migration story |
| Turn/shutdown ownership | `bff2d40dee`, `54c13688a8` (2026-10-08) | Fixes async ownership in dsh's agent loop and job runner. dsb's loop is `crates/dsb-agent/src/loop_.rs`, so there is no shared code to fix; read the dsh notes only if a matching symptom appears |
| Process-specific values out of the system prompt | `36779dcf09` (2026-10-11) | Moves checkout path and web URL to runtime context so a process restart cannot rewrite the logged prompt. The dsb analogue would move `EnvironmentSummary.cwd` out of the prefix (`crates/dsb-context/src/prefix.rs:57`), which contradicts the current prefix design; needs a decision first |
| PTC runtime and sandbox policy | `244ca73755` (2026-10-09) and the `packages/ptc-runtime` log; `packages/sandbox` saw release commits only in this window | PTC (programmatic tool calling) has no counterpart in dsb's snippet contract ([spec 45](../specs/45-snippet-edit.md)); porting the runtime is a different product shape |

---

## 4. N/A

| Area | What moved | Why it does not map |
|---|---|---|
| Web, desktop, browser UI | `apps/web` 219 files; `packages/client` 1,157 files | dsb ships a terminal UI; the web client is dsh's product surface |
| Computer use and browser use | `packages/computer-use` 7 commits, `packages/browser-use` 7 | No such capability in dsb, and none is planned |
| Office document skills | `packages/document` 8 commits; `office`-scoped fixes in the log | Desktop skills for Word, Excel, and PowerPoint; dsb has no desktop skill surface |
| Voice input | `voice-input` and speech-recognition commits in the web client | Microphone selection and speech-to-text live in dsh's browser UI |
| i18n | Every note and doc gained a Chinese pair | Repository artifacts here stay English |
| Agent teams and other experimental packages | `packages/experimental` 182 commits, 817 files | Experimental tree with no stable contract; not evidence for a stable design |
| Account notifications | `packages/feedback` 11 commits | Tied to the dsh account product |
| Plugin marketplace and HMR | `packages/extensions` 104 commits | dsb extends by overlay crates and vendor patches, not a plugin tree |
| Remote access and RBAC | `packages/host` 19 commits, `packages/acp` 21 | dsb has no remote server or ACP surface |
| pi-ai / Anthropic transport changes | `packages/llm` 58 commits | dsh's provider layer is Anthropic Messages; dsb is Chat Completions ([ADR 0005](../adr/0005-deepseek-provider-contract.md)) |
| Flash 1017 model naming | Model-ID churn in the window | Already handled in dsb 6.12.0 by [ADR 0015](../adr/0015-deepseek-v41-flash-1017.md) |

---

## 5. What this does not establish

- **Commit subjects are not behavior.** The fourteen commits named in §2 and §3
  had their messages read; the Take rows also had their diffs or design notes
  read. A subject like "centralize working directories" describes intent.
- **No live-wire verification beyond Search and Fetch.** Those two calls are
  recorded in §2.1. Every other dsh claim is read from source and docs.
- **The auxiliary-request saving is unmeasured.** §2.2 sets a hypothesis, not a
  result.
- **This is not a file-by-file audit.** 8,431 changed files were counted, not
  read; the package-level log and the notes tree carried the survey.
- **The subject is a release candidate.** `dsh-v0.2.1-rc.2` states no
  compatibility promise, and anything adopted must be re-read against the final
  release.
- **dsh-side quality is out of scope.** Whether the search results are good, or
  the presets are the right abstraction, is not answered here.

---

## 6. Next candidates

Ordered by cost. Completion conditions are one line each.

1. **DeepSeek Search and Fetch provider.** A separate session already holds the
   implementation. Done when `web_search` is no longer deferred in spec 40, the
   fetch contract is written, `ori_question` defaults off, and a live call
   through dsb returns sources with no credential in the log.
2. **Minimum reasoning for the title request.** Done when the title request
   selects the route's floor effort and a billed before/after on one model
   records input/output tokens and latency.
3. **Subagent composition and depth.** Done when spec 60 names a child
   composition and a depth cap, a resumed child restores the same composition,
   and the cap is announced through runtime context rather than the system
   prompt.
