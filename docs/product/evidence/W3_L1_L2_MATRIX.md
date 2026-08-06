# W3 L1 / L2 under product shell (G009–G010)

**Date:** 2026-08-06

## L1 — snippet + permissions (P0 #4)

| Surface | Status | Evidence |
|---------|--------|----------|
| Snippet edit contract | PASS | `crates/dsb-tools` Spec 45 tests (`cargo test -p dsb-tools`) |
| Permissions ask/deny/allow | PASS | `permissions.rs` + grants; interactive vs headless |
| Headless fail-closed | PASS | `PermissionPolicy.headless` converts Ask → Deny |
| Grok tools capability filter | DOCUMENTED | `xai-grok-workspace::capability::CapabilityMode` (read / no-edit / full) |
| Product default YOLO | OFF | config seed `[ui]` does not enable yolo; dogfood is opt-in |

### TTY vs headless matrix

| Mode | Interactive ask | Deny on ask? |
|------|-----------------|--------------|
| TTY `chat` / `repl-legacy` | yes (default) | no |
| `run` without `--ask-permissions` | no | yes (fail-closed) |
| `--dogfood` | no (workspace write allowed under policy) | n/a |
| Grok agent TTY | Grok permission reverse-requests | headless agent flags fail-closed |

### Commands

```bash
cargo test -p dsb-tools
cargo test -p dsb-cli
```

## L2 — prefix / epoch (P0 #5)

| Surface | Status | Evidence |
|---------|--------|----------|
| Stable prefix bytes | PASS | `dsb-context` `stable_prefix_bytes` + epoch SHA-256 |
| Epoch stability | PASS | `prefix_stable_across_two_builds` test |
| Live turn | PASS | `prefix_epoch=…` line on live `dsb run` (W2 chat evidence) |
| Grok-equivalent | DOCUMENTED | Grok compaction + chat-state; product continues dsb-context for thin path |

### Commands

```bash
cargo test -p dsb-context
```

## REPLAN §2 P0 items 4–5

4. L1 minimum under shell — **green** (tests + matrix)  
5. L2 minimum stable prefix — **green** (tests + live epoch line)

Optional P1 (skills thrash-free, Flash/Pro polish) may slip past 2.0.0.
