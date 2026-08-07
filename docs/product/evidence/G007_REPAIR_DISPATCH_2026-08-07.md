# G007 RepairDispatch — evidence (owner-bar-5x)

| Field | Value |
|-------|--------|
| **Story** | G007 RepairDispatch |
| **WAVE** | **5x-H2-1** |
| **Date** | 2026-08-07 |

## Done criteria

| ID | Check | Result |
|----|-------|--------|
| **L2-15-1** | One repair pass then execute or structured error | **PASS** unit + Path A R0A |
| **L2-15-2** | Never invent required args / rename tool | **PASS** unit (`h15_2`, `h15_3`) |
| **L2-15-3** | Session pairing (resume) | **PARTIAL** thin path; Path A re-prove G008 |
| **L2-15-4** | Repair on **Grok dispatch** not only dsb-agent | **PASS** `tool_calls.rs` uses `repair_tool_arguments_one_pass` |

## What shipped

1. **`repair_tool_arguments_one_pass`** in Grok `tool_input_parsing` (Spec 15: trailing commas, single quotes, control escapes, one pass).
2. **Production call site** in `acp_session_impl/tool_calls.rs` before JSON parse/execute.
3. Unit tests `spec15_*` (4) + existing `dsb-agent path_a` H15 (8 tests).
4. Path A R0A: scripted tool args with trailing comma → edit applied → `hello-repaired`.

## Commands

```bash
cargo test -p dsb-agent path_a          # 8 passed (H15+H20 units)
cargo test -p xai-grok-shell spec15_    # 4 passed

# Path A R0A
# scenario repair-trailing-comma → REPAIR_DISPATCH_PASS
# a.txt: hello → hello-repaired
# tool: The file a.txt has been updated successfully.
```

## Artifacts

- `PATH_A_R0_G007_REPAIR_WIRE_last.jsonl`
- `PATH_A_R0_G007_REPAIR_META_last.txt`

## Explicit non-claims

- Prefix goldens / skills resume = G008.
- Flash/Pro routing wire = G009.
