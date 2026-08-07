# H90 Path A permissions + dogfood — G005 evidence

**Date:** 2026-08-07  
**Band:** `3.0.0-alpha.2`  
**Story:** G005 L1-Permissions · WAVE `3x-H1-2`, `3x-H1-3`  
**Binding:** [HEART_3X_SPEC_BINDING.md](../../architecture/HEART_3X_SPEC_BINDING.md)  
**Cases:** [HEART_3X_P0_TEST_PLAN.md](../HEART_3X_P0_TEST_PLAN.md) H90.*

## What shipped

| Layer | Change |
|-------|--------|
| Product contract | `crates/dsb-tools/src/path_a_permissions.rs` — TTY/headless × capability × Spec 90 scopes |
| Config seed/repair | Explicit `yolo = false` when missing; never product-default YOLO |
| Hermetic live home | `scripts/lib/common.sh` documents Spec 90 yolo default |

## Matrix (H90)

| Mode | Write in cwd | Write out cwd | Notes |
|------|--------------|---------------|-------|
| TTY, yolo=false | **Ask** | **Deny** | Product interactive default |
| Headless, yolo=false | **Deny** | **Deny** | Fail-closed (Ask→Deny) |
| Headless, yolo=true (opt-in) | **Allow** | **Deny** | Explicit CLI/config only |
| Capability ReadOnly | **Deny** edit | **Deny** | Capability wins before scope ask |

## Commands

```bash
cargo test -p dsb-tools path_a
cargo test -p dsb-cli product_config_seed repair_injects_yolo -- --nocapture
./scripts/check-semver.sh
# Live (when agent + API key): PRE_3X T5.8 permission deny without --yolo
# ./scripts/test-deepseek-live.sh --agent
```

## Dogfood note (3x-H1-3)

| Check | Result |
|-------|--------|
| Automated H90.* | PASS (`path_a_permissions` tests) |
| Seed not YOLO | PASS (`yolo = false` in seed + repair) |
| Live agent headless write without `--yolo` | PRE_3X **T5.8** designed for this; run when `deepseek-build-agent` + key available |
| Live with `--yolo` | Opt-in only; still out-of-cwd deny under Spec 90 product policy |

**H1 exit claim:** L1 snippet (G004) + L1 permissions contract (this PR) are on the **default product path** (config + capability×policy matrix). Full live T5.8 remains environment-gated, not claimed green without agent binary/key.

## SemVer

**`3.0.0-alpha.2`**
