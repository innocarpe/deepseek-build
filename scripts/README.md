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
| **`test-owner-bar.sh`** | **Owner-bar-5x aggregator** — RED until all P0 PASS (`--selftest` for gate substrate) |
| `check-path-a-linkage.sh` | Fail-close dead wiring / missing mint / orphan `path_a_*` |
| `check-forbidden-evidence.sh` | Fail if active 5.x evidence uses sole Path B cargo claims |
| **`test-path-a-public-entry-e2e.sh`** | **Path A R0A** public CLI → agent_launch → scripted DeepSeek + wire |
| `lib/scripted_deepseek_server.py` | Hermetic Chat Completions fixture (SSE + wire JSONL) |
| `lib/owner-bar-common.sh` | Shared helpers for owner-bar gates |

## Owner-bar-5x (active product train → `5.0.0`)

Board: [`docs/product/OWNER_BAR_5X_GOALS.md`](../docs/product/OWNER_BAR_5X_GOALS.md).

```bash
./scripts/test-owner-bar.sh            # expect non-zero until late train
./scripts/test-owner-bar.sh --selftest # gate substrate
./scripts/test-path-a-public-entry-e2e.sh
```

Wire/meta: `docs/product/evidence/PATH_A_R0_*_last.*`

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

## L3 smoke (4.0 prep, parallel-safe)

Does **not** change product defaults. Uses installed `deepseek-build-agent` +
DeepSeek API. Safe while **heart-3x** develops (separate worktree recommended).

```bash
./scripts/test-l3-smoke.sh              # CLI + headless + bg shell + worktree help
./scripts/test-l3-smoke.sh --extended   # + spawn_subagent (slower)
```

Results: `docs/product/evidence/_last_l3_smoke.tsv`  
Ops plan: [PARALLEL_3X_4X_PLAN.md](../docs/product/PARALLEL_3X_4X_PLAN.md)

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
