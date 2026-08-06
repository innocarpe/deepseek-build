# Dogfood notes

Short, honest records of using **DeepSeek Build** on real work (this repo first).

| Date | Note | SemVer |
|------|------|--------|
| 2026-08-06 | [Live smoke + agent write](./2026-08-06-live-smoke.md) | **0.4.0** |

## How to add a note

Prefer producing the note **with** `deepseek-build --dogfood` (or `dsb --dogfood`) so the entry proves the product path, not only unit tests.

```bash
export PATH="$HOME/.deepseek-build/bin:$PATH"
# Auth: ~/.deepseek-build/credentials.json or DEEPSEEK_API_KEY
deepseek-build run "Reply with exactly: pong"
deepseek-build --dogfood run "Create docs/dogfood/YYYY-MM-DD-topic.md describing …"
```

Never put API keys in these notes.
