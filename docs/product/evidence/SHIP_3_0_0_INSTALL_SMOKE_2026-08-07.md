# Ship 3.0.0 — G002 install + smoke

**Date:** 2026-08-07  
**Plan:** `ship-3.0.0`

## Install

```bash
cargo install --path crates/dsb-cli --locked --force --root ~/.deepseek-build
export PATH="$HOME/.deepseek-build/bin:$PATH"
deepseek-build --version  # → deepseek-build 3.0.0
dsb --version             # → dsb 3.0.0
```

Agent binary: `~/.deepseek-build/bin/deepseek-build-agent` (present; T2.1 PASS).

## Smoke

| Suite | Result |
|-------|--------|
| `./scripts/test-product-offline.sh` | ALL PASSED |
| `./scripts/test-pre3x-baseline.sh` (default) | PASS=9 FAIL=0 SKIP=3 (T1/T3/T4) |
| Live `--live` | SKIP (no API key in env) |

## Path A contracts

| Crate | Result |
|-------|--------|
| dsb-tools path_a | 15 passed |
| dsb-context path_a | 5 passed |
| dsb-agent path_a | 8 passed |
