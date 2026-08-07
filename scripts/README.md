# scripts/

| Script | Role |
|--------|------|
| `install.sh` | Install **`deepseek-build`** + **`dsb`** onto PATH (`~/.deepseek-build/bin` or Cargo bin) |
| `build-grok-pager.sh` | Build/check vendored Grok composition root (`deepseek-build-agent`) |
| `check-semver.sh` | Fail-close: workspace version must be full SemVer `MAJOR.MINOR.PATCH` |
| `check-pr-title.sh` | **Optional** local Conventional Commits title check (not CI) |
| `sync-labels.sh` | Push `.github/labels.json` to GitHub labels |
| `smoke-dogfood.sh` | Quick offline smoke (+ optional thin live if key set) |
| **`test-pre3x-baseline.sh`** | **Pre-3.0.0 orchestrator** (T0–T4) — see matrix doc |
| `test-product-offline.sh` | T0 + T2 offline product crates / dual bins / config seed |
| `test-grok-vendor-offline.sh` | T1 curated Grok vendor `cargo test` (not full workspace) |
| `test-deepseek-live.sh` | T3 thin + **T4 agent** live DeepSeek API feature probes |
| `lib/common.sh` | Shared helpers (key load, hermetic GROK_HOME, redaction) |

## Pre-3.0.0 baseline (required before heart fusion)

Normative matrix: [`docs/product/PRE_3X_TEST_MATRIX.md`](../docs/product/PRE_3X_TEST_MATRIX.md).

```bash
# Everyday (recommended): product offline + DeepSeek live agent smoke
./scripts/test-pre3x-baseline.sh --live

# Offline product only
./scripts/test-pre3x-baseline.sh

# Vendor offline (tiered — default light; avoid --vendor-full unless needed)
./scripts/test-pre3x-baseline.sh --vendor
./scripts/test-pre3x-baseline.sh --vendor-medium
./scripts/test-pre3x-baseline.sh --vendor-full   # HEAVY disk under third_party/grok-build/target

# After a heavy vendor run, free disk:
rm -rf third_party/grok-build/target
```

Results TSV: `docs/product/evidence/_last_pre3x_results.tsv`  
Durable report: `docs/product/evidence/PRE3X_BASELINE_YYYY-MM-DD.md`

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
