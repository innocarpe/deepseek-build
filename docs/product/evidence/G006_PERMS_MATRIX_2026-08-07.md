# G006 PermsMatrix — evidence (owner-bar-5x)

| Field | Value |
|-------|--------|
| **Story** | G006 PermsMatrix |
| **WAVE** | **5x-H1-3** |
| **Date** | 2026-08-07 |

## Done criteria

| ID | Check | Result |
|----|-------|--------|
| **L1-90-1** | allow/deny/ask matrix on mutate | **PASS** unit (`path_a_permissions` 7 tests) |
| **L1-90-2** | Headless Ask→deny | **PASS** Path A R0A: `denied by prompt policy (tool not pre-approved)` without `--yolo` |
| **L1-90-3** | Product default not YOLO | **PASS** seed/repair `yolo = false` + unit `h90_4_product_default_not_yolo` |
| **L1-90-4** | Workspace boundary | **PASS** unit out-of-cwd deny; Path A headless denied mutate entirely when not YOLO |
| **L1-90-5** | Parallel/subagent cannot skip perms | **Deferred recheck G010** (unit capability filter present; full R0A under L3) |
| **S8** | Default non-YOLO | **PASS** (seed + Path A config) |

## Commands

```bash
cargo test -p dsb-tools path_a_perm
# 7 passed

cargo test -p dsb-cli product_config_seed_contains_deepseek_defaults
# ok (yolo = false in seed)

# Path A R0A headless, yolo=false, no --yolo:
# deepseek-build agent -p … → search_replace denied by prompt policy
# file preserved keep-me
# HEADLESS_SAFE_PASS
```

## Artifacts

- `PATH_A_R0_G006_HEADLESS_WIRE_last.jsonl`
- `PATH_A_R0_G006_META_last.txt`

## Wire excerpt

```text
Tool `search_replace` was not executed: denied by prompt policy (tool not pre-approved)
```

Config: `yolo = false`, `permission_mode = default`, headless (`-p`), no CLI `--yolo`.

## Explicit non-claims

- L1-90-5 full subagent bypass proof waits for G010.
- Live interactive TTY Ask UX not re-dogfooded this PR (matrix unit covers TTY Ask decision).
