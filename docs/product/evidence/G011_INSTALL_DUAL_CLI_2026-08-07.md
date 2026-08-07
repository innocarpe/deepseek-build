# G011 InstallDualCLI — evidence (owner-bar-5x)

| Field | Value |
|-------|--------|
| **Story** | G011 InstallDualCLI |
| **WAVE** | **5x-H4-1** |
| **Date** | 2026-08-07 |

## Done criteria

| ID | Check | Result |
|----|-------|--------|
| **S1** | Dual CLI same SemVer | **PASS** `deepseek-build` / `dsb` → `4.0.4` |
| **S2** | Bare TTY → full-screen agent | **PASS** public entry agent path (prior G002; re-proved) |
| **S3** | Product home + GROK_HOME bridge | **PASS** stamps under `DEEPSEEK_BUILD_HOME` |
| **S4** | Model-level DeepSeek `base_url` | **PASS** config seed both models |
| **S5** | Readable DeepSeek theme | **PASS** `theme = "deepseeknight"` |
| **S7** | Clean-prefix install CLI+agent | **PASS** hermetic prefix + agent hash |
| **OB-1** | Installed CLI → agent without override | **PASS** `DEEPSEEK_BUILD_AGENT_BIN` unset |

## What shipped

1. **`scripts/test-install-dual-cli.sh`** — hermetic clean-prefix dual CLI + agent smoke + public entry.
2. **`install.sh`** — also installs `xai-grok-pager` fallback name; debug agent fallback when release missing.
3. Package inventory artifact (npm dual `bin` + installed hashes).

## Commands

```bash
./scripts/test-install-dual-cli.sh
# S1 SemVer match, S3 stamps, S5 theme, OB-1 wire, PASS
```

## Artifacts

| Path | Role |
|------|------|
| [`PATH_A_R0_G011_INSTALL_META_last.txt`](./PATH_A_R0_G011_INSTALL_META_last.txt) | Meta + agent hash |
| [`PATH_A_R0_G011_PACKAGE_INVENTORY_last.txt`](./PATH_A_R0_G011_PACKAGE_INVENTORY_last.txt) | Package + install inventory |

## Explicit non-claims

- Does not publish npm or claim multi-platform prebuilt matrix beyond primary.
- Full cargo-from-zero install of agent still slow (dev path); hermetic uses built artifacts.
- Cut / dual review = G012.
