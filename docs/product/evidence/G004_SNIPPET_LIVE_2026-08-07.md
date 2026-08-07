# G004 SnippetLive — evidence (owner-bar-5x)

| Field | Value |
|-------|--------|
| **Story** | G004 SnippetLive (`G004-g004-snippetlive`) |
| **WAVE unit** | **5x-H1-1** |
| **Date** | 2026-08-07 |
| **Depends** | G003 mint (PR #96) |

## Done criteria

| Check | Result |
|-------|--------|
| Dead wiring fixed: Standard applies `tool_configs` | **PASS** (`check-path-a-linkage` PASS) |
| `snippet_safe` + empty-old guard on Standard default | **PASS** (configs already defined; now applied) |
| Negatives unit (missing / stale version) | **PASS** (3 tests) |
| **L1-45-0 liveness** ≥3 edits / ≥2 files / exit 0 | **PASS** (`LIVENESS_PASS` Path A R0A) |

## What shipped

1. **agent_ops.rs** — always `override_file_tools(tool_configs(...))` including Standard.
2. **subagent_coordinator.rs** — same for worker file tool overrides.
3. Scripted server `liveness-3edits` scenario for Path A R0A.
4. Evidence wire/meta for liveness.

## Commands

```bash
./scripts/check-path-a-linkage.sh
# PASS (no DEAD_WIRING, no NO_MINT)

cd third_party/grok-build
cargo test -p xai-grok-tools snippet_safe
# 3 passed

# Path A public entry + rebuilt agent + scripted liveness-3edits
# a.txt: hello → hello1 → hello2
# b.txt: world → world1
# LIVENESS_PASS agent_exit=0
```

## Artifacts

- `PATH_A_R0_LIVENESS_WIRE_last.jsonl`
- `PATH_A_R0_LIVENESS_META_last.txt`

## Explicit non-claims

- Write/bash invalidate = G005.
- Full ledger still RED until later stories.
