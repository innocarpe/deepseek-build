# 00 — First-run setup (onboarding)

**Product version:** `1.1.0`+ (provider choice: unreleased, after `5.5.4`)

Install alone is not enough. You need an API key before the agent can answer:
either a **DeepSeek API** key, or an **OpenRouter** key for the same DeepSeek
V4 Flash / Pro models.

## Natural first run

On a TTY, if no key is configured:

```bash
deepseek-build          # no subcommand → starts setup when unconfigured
deepseek-build chat     # missing key → setup wizard, then continues
deepseek-build setup    # explicit setup
dsb auth login          # same as setup
```

The wizard has two steps:

1. **API provider** — `1` DeepSeek API (`https://api.deepseek.com`, default on
   Enter) or `2` OpenRouter (`https://openrouter.ai/api/v1`).
2. **API key** for that provider — create one at
   `https://platform.deepseek.com/api_keys` or `https://openrouter.ai/keys`
   and paste it (not echoed into git).

It saves `~/.deepseek-build/credentials.json` (mode **0600**) with the key and
the provider, then prints next commands.

The full-screen agent's `~/.deepseek-build/config.toml` follows the choice:

| | DeepSeek API | OpenRouter |
|---|---|---|
| `base_url` | `https://api.deepseek.com` | `https://openrouter.ai/api/v1` |
| Flash slot `model` | `deepseek-v4-flash` | `deepseek/deepseek-v4-flash` |
| Pro slot `model` | `deepseek-v4-pro` | `deepseek/deepseek-v4-pro` |
| `env_key` | `DEEPSEEK_API_KEY` | `OPENROUTER_API_KEY` |

A fresh home is seeded for the chosen provider. If `config.toml` already
exists, setup switches only the stanzas that still hold the other provider's
product defaults; a stanza you edited by hand (custom model, proxy URL, own
`env_key`) is left alone and setup prints a note.

Session titles, web search, and image description are pinned to the Flash
stanza (`[models] session_summary` / `web_search` / `image_description`), so
those side requests use the same provider instead of a vendored default model.

## Non-interactive

```bash
# CI / scripted — DeepSeek API
export DEEPSEEK_API_KEY=sk-...
deepseek-build setup --api-key "$DEEPSEEK_API_KEY"
# or just rely on env without writing a file

# OpenRouter
deepseek-build setup --provider openrouter --api-key "$OPENROUTER_KEY"
```

For DeepSeek, `DEEPSEEK_API_KEY` **always wins** over the file when both are
set. It never overrides a saved OpenRouter choice (it is a DeepSeek key), and
`--provider openrouter` never reads it.

## Line mode

`run` / `chat` speak the DeepSeek wire contract to `api.deepseek.com` only.
With OpenRouter saved they stop with a message instead of sending the
OpenRouter key to DeepSeek; use the full-screen agent, or
`deepseek-build setup --provider deepseek` for line mode.

## Status / logout

```bash
deepseek-build auth status   # configured? provider? source? masked key
deepseek-build auth logout   # deletes credentials file (env unchanged)
```

## Hand-written config with your own key variable

If the default model stanza in `config.toml` names its own `env_key` and that
variable is set, the full-screen agent can authenticate without
`credentials.json`, so first-run setup is skipped. `auth status` still says
`not configured` and adds a note.

## Headless / no TTY

Without a key and without a TTY, the CLI fails with a clear message pointing at `setup` / env / file — it does **not** hang waiting for input.
