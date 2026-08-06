# W1 entry smoke — G005 / G006

**Date:** 2026-08-06  
**Band:** 2.0.0-alpha.2  
**Host:** macOS arm64

## Install

```bash
./scripts/build-grok-pager.sh release   # ~20m cold release
./scripts/install.sh                     # wrapper + deepseek-build-agent
export PATH="$HOME/.deepseek-build/bin:$PATH"
```

## Evidence

| Check | Result |
|-------|--------|
| `dsb --version` | Product SemVer from dsb-cli |
| `test -x ~/.deepseek-build/bin/deepseek-build-agent` | PASS (Grok pager composition root) |
| `dsb` no-args non-TTY | Exit 2 with guidance (not thin REPL) |
| `dsb` no-args TTY | Execs agent (full-screen Grok-class UI) — manual dogfood |
| `dsb repl-legacy` | Thin 1.x REPL still available |
| `dsb setup` / credentials | `~/.deepseek-build/credentials.json` mode 0600 |
| Product `config.toml` seed | DeepSeek models + `api.deepseek.com` + `chat_completions` |

## Branding

- CLI about / help: **DeepSeek Build**, not Grok as product name
- Agent UI may still contain upstream Grok strings until deeper chrome patch; product chrome and docs use DeepSeek Build
- Config seed header states product name

## Auth

- Missing key on TTY → setup wizard before agent launch
- Credentials 0600 via `dsb-config::Credentials::save`
- Agent child receives `GROK_HOME=~/.deepseek-build` and `DEEPSEEK_API_KEY` when available
