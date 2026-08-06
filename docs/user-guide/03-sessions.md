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

## Limits

- Stable prefix (system/tools) is **not** stored in the session file — rebuilt each process from current code/config (cache epoch may change across upgrades).
- Snippet store is process-local; resume does not restore snippet ids (re-`read` after resume for edits).
- No fork/branch UX yet.
