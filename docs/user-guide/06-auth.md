# 06 — Authentication

**Product version:** `0.16.0`+

DeepSeek Build calls the DeepSeek API. Credentials are **never** committed to git.

## Load order

1. Environment: `DEEPSEEK_API_KEY`
2. File: `~/.deepseek-build/credentials.json` (mode ideally `0600`)

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

## Missing key

If neither source provides a key, CLI exits with a clear error pointing at env + credentials path.

## Related

- ADR 0005 (provider contract)
- `crates/dsb-config`
