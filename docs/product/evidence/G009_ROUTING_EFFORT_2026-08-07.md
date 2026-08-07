# G009 RoutingEffort — evidence (owner-bar-5x)

| Field | Value |
|-------|--------|
| **Story** | G009 RoutingEffort |
| **WAVE** | **5x-H2-3** |
| **Date** | 2026-08-07 |
| **Depends** | G002 rig (+ prefer G008) |

## Done criteria

| ID | Check | Result |
|----|-------|--------|
| **L2-20-1** | Default session model Flash (`deepseek-v4-flash`) | **PASS** public-entry wire (`flash=2`) + config seed |
| **L2-20-2** | Pro one-turn escalate then return (or sticky) | **PASS** unit H20.2 + production stamp Flash→Pro→Flash; Path A wire with `-m deepseek-v4-pro` |
| **L2-20-3** | Turn model visibility | **PASS** `RouteDecision::visibility_line` + stamp fields |
| **L2-20-4** | Precedence: user > sticky > auto > default Flash | **PASS** `routing::tests` + path_a_turn H20 |
| **L2-20-5** | Both models carry DeepSeek `base_url` | **PASS** product config seed/repair for flash+pro |
| **L1-30** | Effort controllable; default coding effort + wire | **PASS** router default `effort=high` + CLI `--reasoning-effort`; **PARTIAL** on Path A chat_completions body (field often omitted/`null` on Grok wire today) |
| **S4** | Model-level DeepSeek `base_url` | **PASS** seed + hermetic e2e |

## What shipped

1. **`stamp_path_a_routing`** in `agent_launch` — production call site for `path_a_default_router` / `route_path_a_turn` / `apply_routing_command`.
2. Writes `path_a_routing.txt` under product home (Flash default, Pro once, return Flash, visibility lines).
3. Re-export `apply_routing_command` from `dsb-agent`.
4. Public-entry e2e asserts routing stamp + Flash on wire.
5. Unit `stamp_path_a_routing_flash_pro_once`.

## Commands

```bash
cargo test -p dsb-agent path_a       # H15+H20
cargo test -p dsb-agent routing
cargo test -p dsb-cli stamp_path_a
./scripts/check-path-a-linkage.sh    # PASS
./scripts/test-path-a-public-entry-e2e.sh
# wire_models flash≥1; path_a_routing stamp present

# Pro model on Path A (explicit user override / -m)
deepseek-build agent -p '…' --model deepseek-v4-pro --reasoning-effort high …
# wire model=deepseek-v4-pro
```

## Artifacts

| Path | Role |
|------|------|
| [`PATH_A_ROUTING_last.txt`](./PATH_A_ROUTING_last.txt) | Launch stamp Flash/Pro/effort visibility |
| [`PATH_A_R0_G009_ROUTING_last.txt`](./PATH_A_R0_G009_ROUTING_last.txt) | Copy from Pro run home |
| [`PATH_A_R0_G009_PRO_EFFORT_WIRE_last.jsonl`](./PATH_A_R0_G009_PRO_EFFORT_WIRE_last.jsonl) | Slim wire: pro model + effort keys |
| [`PATH_A_R0_META_last.txt`](./PATH_A_R0_META_last.txt) | Public entry meta incl. stamps |

## Explicit non-claims

- Full ledger greening = G012.
- L3 scheduler / subagents = G010.
- Path A wire may still omit `reasoning_effort` JSON field even when CLI flag is set — residual honesty for L1-30 wire-assert until Grok chat_completions path always serializes it.
