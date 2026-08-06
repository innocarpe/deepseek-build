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

## `0.x.y` train (default)

Normative schedule and dogfood definition:  
[**RELEASE_TRAIN_0x.md**](../product/RELEASE_TRAIN_0x.md).

Stay on **`0.y.z`** until dogfood-usable. Do not plan **`1.0.0`** in ultragoal stories for this phase.

### Every `0.y.z` release checklist

- [ ] Version fields consistent (`Cargo.toml` / npm if any / tag `vMAJOR.MINOR.PATCH`)  
- [ ] `./scripts/check-semver.sh` passes  
- [ ] `deepseek-build --version` and `dsb --version` show the **same** full SemVer  
- [ ] README smoke commands updated for that minor’s theme  
- [ ] CHANGELOG (when file exists) or PR body lists user-visible delta  
- [ ] Progress log row in `RELEASE_TRAIN_0x.md`  

### Dogfood-usable (see train §3) — not `1.0.0`

- [ ] PATH install works  
- [ ] Auth works  
- [ ] Chat + tools (read/edit/write/search/bash under policy) for real work  
- [ ] Documented workspace write profile  
- [ ] Owner dogfood note recorded  

### Later: `1.0.0` (out of current train)

Only after sustained dogfood + boring npm/install; separate plan.
