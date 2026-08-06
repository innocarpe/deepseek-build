# scripts/

| Script | Role |
|--------|------|
| `install.sh` | Install **`deepseek-build`** + **`dsb`** onto PATH (`~/.deepseek-build/bin` or Cargo bin) |
| `check-semver.sh` | Fail-close: workspace version must be full SemVer `MAJOR.MINOR.PATCH` |
| `check-pr-title.sh` | **Optional** local Conventional Commits title check (not CI) |
| `sync-labels.sh` | Push `.github/labels.json` to GitHub labels |

## Install (product)

```bash
# From repo root
./scripts/install.sh              # → ~/.deepseek-build/bin
./scripts/install.sh --cargo      # → ~/.cargo/bin
./scripts/check-semver.sh
deepseek-build --version          # after PATH includes the bin dir
dsb --version
```

See root [README.md](../README.md) § Install.
