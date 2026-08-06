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

## npm (ADR 0007)

- Package name: **`@innocarpe/deepseek-build`** (scoped)  
- CLI bins: **`deepseek-build`** + **`dsb`**  
- `package.json` version = Cargo workspace SemVer  
- Agent complete: pack + local install smoke; **publish = human**  
- See [ADR 0007](../adr/0007-npm-packaging.md) · [user-guide/05-npm.md](../user-guide/05-npm.md)

## `0.x.y` trains

- Wave A: [RELEASE_TRAIN_0x.md](../product/RELEASE_TRAIN_0x.md)  
- All waves: [MASTER_PLAN.md](../product/MASTER_PLAN.md)  
- Do not plan **`1.0.0`** until Wave D checklist.

### Every `0.y.z` release checklist

- [ ] Files in [SSOT.md](../product/SSOT.md) version list updated  
- [ ] `./scripts/check-semver.sh`  
- [ ] `npm run version-check` if package.json exists  
- [ ] Dual bins same SemVer  
- [ ] `./scripts/smoke-dogfood.sh` (or document skip reason)  
- [ ] Progress log MASTER_PLAN / RELEASE_TRAIN as applicable  

### Dogfood-usable

- [ ] `./scripts/smoke-dogfood.sh` exit 0  
- [ ] Optional live key smoke  

### Later: `1.0.0`

Wave D PRD only.
