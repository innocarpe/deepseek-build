# 07 — Chat and one-shot run

**Product version:** `0.16.0`+

## Commands

| Command | Mode |
|---------|------|
| `deepseek-build chat` / `dsb chat` | Multi-turn REPL |
| `deepseek-build repl` | Alias of `chat` |
| `deepseek-build run "…"` | One-shot message |
| `deepseek-build run` + stdin | One-shot from pipe |

Both public names (`deepseek-build`, `dsb`) are the same binary.

## REPL

```bash
deepseek-build chat
# or with dogfood + session:
deepseek-build --dogfood --session mywork chat
```

Slash-ish routing helpers (same turn text):

| Input | Effect |
|-------|--------|
| `/pro …` | One-shot Pro model for this turn |
| `/flash …` | Flash model |
| `/preset max` | Sticky max preset |
| `/quit` or `/exit` | Leave REPL (session persists if `--session`) |

## Common flags

| Flag | Meaning |
|------|---------|
| `--cwd PATH` | Workspace root |
| `--preset flash\|balanced\|max` | Routing preset |
| `--effort low\|high\|max` | Reasoning effort override |
| `--thinking` / `--no-thinking` | Thinking mode |
| `--show-reasoning` | Stream reasoning to stderr |
| `--quiet-model` | Hide model visibility lines |
| `--session ID` | Persist/resume under `~/.deepseek-build/sessions/` |
| `--dogfood` | Trusted local write + bash execute |
| `--allow-workspace-write` | Allow in-cwd writes without ask |
| `--bash-execute` | Enable bash execution |
| `--ask-permissions` | Force TTY ask even for `run` |
| `--no-ask-permissions` | Never prompt (ask → deny) |

## Theme

On TTY, tool/meta lines use DeepSeek blue accents (`docs/product/DESIGN.md`). Set `NO_COLOR=1` for plain text.
