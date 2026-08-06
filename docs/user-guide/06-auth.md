# 06 — Authentication

**Product version:** `1.1.0`+ (interactive setup; load order since 0.1)

DeepSeek Build calls the DeepSeek API. Credentials are **never** committed to git.

## First-time (preferred)

See **[00-setup.md](./00-setup.md)**. Short path:

```bash
deepseek-build setup
# or: deepseek-build chat  → wizard if no key on TTY
```

## Load order

1. Environment: `DEEPSEEK_API_KEY`
2. File: `~/.deepseek-build/credentials.json` (mode **0600**)

```json
{
  "api_key": "sk-..."
}
```

Optional base URL (proxies/tests):

```bash
export DEEPSEEK_BASE_URL=https://api.deepseek.com
# or
deepseek-build --base-url https://api.example.com chat
```

Config home override:

```bash
export DEEPSEEK_BUILD_HOME=/path/to/home
```

## Commands

| Command | Effect |
|---------|--------|
| `setup` / `auth login` | Interactive or `--api-key` save |
| `auth status` | Configured? masked key |
| `auth logout` | Delete credentials file |

## Missing key (headless)

If neither source provides a key **and** stdin is not a TTY, CLI exits with guidance to run `setup` or set the env.

## Related

- [00-setup.md](./00-setup.md)
- ADR 0005 (provider contract)
- `crates/dsb-config`
