# 00 — First-run setup (onboarding)

**Product version:** `1.1.0`+

Install alone is not enough. You need a **DeepSeek API key** before `chat` / `run` work.

## Natural first run

On a TTY, if no key is configured:

```bash
deepseek-build          # no subcommand → starts setup when unconfigured
deepseek-build chat     # missing key → setup wizard, then continues
deepseek-build setup    # explicit setup
dsb auth login          # same as setup
```

The wizard:

1. Explains where to create a key (`https://platform.deepseek.com/api_keys`)
2. Prompts for the key (not echoed into git)
3. Saves `~/.deepseek-build/credentials.json` with mode **0600**
4. Prints next commands (`chat`, `auth status`)

## Non-interactive

```bash
# CI / scripted
export DEEPSEEK_API_KEY=sk-...
deepseek-build setup --api-key "$DEEPSEEK_API_KEY"
# or just rely on env without writing a file
```

Env **always wins** over the file when both are set.

## Status / logout

```bash
deepseek-build auth status   # configured? source? masked key
deepseek-build auth logout   # deletes credentials file (env unchanged)
```

## Headless / no TTY

Without a key and without a TTY, the CLI fails with a clear message pointing at `setup` / env / file — it does **not** hang waiting for input.
