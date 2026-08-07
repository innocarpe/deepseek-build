# 03 — Sessions

**Product version:** `0.5.0`+  
**Commands:** `deepseek-build` · `dsb`

## Persist multi-turn chat

```bash
deepseek-build --session my-feature --dogfood chat
```

Each turn appends the volatile transcript to:

```text
~/.deepseek-build/sessions/my-feature.jsonl
```

Override home with `DEEPSEEK_BUILD_HOME`.

## Resume by id

Same flag resumes and **repairs tool-call pairs** on load (spec 15): any assistant `tool_calls` without a matching tool result gets a `tool_result_interrupted` placeholder so the next API call is valid.

```bash
deepseek-build --session my-feature chat
dsb sessions show my-feature
```

## Manage

| Command | Effect |
|---------|--------|
| `sessions list` | Recent sessions + message counts |
| `sessions show <id>` | Message count + repaired hole count |
| `sessions delete <id>` | Remove JSONL file |

Session ids: `A–Z a–z 0–9 _ -` only, max 128 chars.

## Full-screen TUI sessions (bare `dsb`)

The product TUI (`dsb` / `deepseek-build` on a TTY, or `dsb agent`) keeps its
own UUID sessions under:

```text
~/.deepseek-build/sessions/<url-encoded-workspace>/<uuid>/
```

Each session directory holds the prompt history and search index. These UUID
sessions are separate from the line-mode JSONL sessions above — `sessions
list` / `show` / `delete` cover the JSONL ones only.

### Resume

When you quit the TUI it prints a pasteable hint:

```text
Resume this session with:
  dsb --resume 019fda93-767a-7181-a90c-e0327af18dd2
```

The wrapper forwards `--resume` to the TUI:

| Command | Effect |
|---------|--------|
| `dsb --resume <session-id>` | Resume a specific TUI session |
| `dsb -r <session-id>` | Short flag (same as above) |
| `dsb --resume` | Resume the most recent TUI session |
| `dsb agent --resume <session-id>` | Forward extra TUI args plus resume |
| `dsb --minimal --resume <id>` | Resume in minimal mode |

`--resume` conflicts with line-mode `--session`: TUI sessions resume the
full-screen UI, while `--session` persists/resumes JSONL line-mode runs. The
TUI-only flags (`--resume` / `--minimal` / `--fullscreen`) are rejected on
`run` / `chat` / `repl` with a pointer to `--session`.

## Limits

- Stable prefix (system/tools) is **not** stored in the session file — rebuilt each process from current code/config (cache epoch may change across upgrades).
- Snippet store is process-local; resume does not restore snippet ids (re-`read` after resume for edits).
- No fork/branch UX yet.
