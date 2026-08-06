# Releases (draft harness)

## SemVer only

All releases use **full** Semantic Version identifiers: `MAJOR.MINOR.PATCH`
(see [versioning.md](./versioning.md)). Never tag or announce `1.0`.

## CLI names

Shipped commands (see [ADR 0006](../adr/0006-cli-names-and-semver.md)):

| Command | Role |
|---------|------|
| `deepseek-build` | **Primary** public command (parallel to `claude` / `codex` / `grok` style clarity) |
| `dsb` | **Short alias** — same binary behavior |

Both must print the same `--version` string (workspace SemVer).

## npm (planned)

When packaging lands:

- `package.json` `"version"` = workspace SemVer  
- `bin` map exposes **both** `deepseek-build` and `dsb`  
- Install smoke: `deepseek-build --version` and `dsb --version` equal  

## Pre-1.0.0 checklist (minimum)

- [ ] Version fields consistent (`Cargo.toml` / npm / tag)  
- [ ] `deepseek-build --version` and `dsb --version` → `deepseek-build X.Y.Z` / `dsb X.Y.Z` with **X.Y.Z** full triple  
- [ ] README install path works from a clean machine  
- [ ] Documented smoke (chat or run) with API key  
- [ ] CHANGELOG entry for that SemVer  
