# G008 PrefixSkillsResume — evidence (owner-bar-5x)

| Field | Value |
|-------|--------|
| **Story** | G008 PrefixSkillsResume |
| **WAVE** | **5x-H2-2** |
| **Date** | 2026-08-07 |
| **Depends** | G004 schema stable (+ prefer G007) |

## Done criteria

| ID | Check | Result |
|----|-------|--------|
| **L2-10-1** | stable_prefix + volatile_tail on Path A | **PASS** library `assemble_path_a_context` + multi-turn wire (system fixed; tail grows) |
| **L2-10-2** | Prefix order Spec 10 | **PASS** unit (`PrefixBuilder`: system → tools → skills index → env → project) |
| **L2-10-3** | Unchanged inputs → byte-stable prefix across turns | **PASS** unit H10.1 + wire analyze (`system_stable` across 5 deepseek turns) |
| **L2-10-4** | No wall-clock in prefix | **PASS** unit `prefix_no_timestamp` + wire system has no wall-clock markers |
| **L2-10-5** | Compaction/resume preserve contract | **PASS** product `--resume` forward + `SessionStore` load/repair; residual: full TUI compaction still Grok-owned |
| **L2-10-6** | Heart impl linked from Path A | **PASS** `agent_launch::stamp_path_a_prefix_epoch` calls `assemble_path_a_context`; linkage check PASS |
| **L1-70** | Skills index in stable prefix; body on demand | **PASS** skills unit (index only + load body) + wire skills reminder stable |
| **L1-100** | Session resume on product agent path | **PASS** CLI `--resume` / `--session` surface + session store R0; Path A TUI resume forwarded to agent |

## What shipped

1. **Production Path A stamp** in `crates/dsb-cli/src/agent_launch.rs`:
   - `stamp_path_a_prefix_epoch` builds Spec 10 inputs (tools + skills index + env)
   - calls `dsb_context::assemble_path_a_context`
   - writes `path_a_prefix_epoch.txt` under product home before agent exec
2. **Unit:** `stamp_path_a_prefix_epoch_writes_stable_file` (dual stamp identical).
3. **Wire analyzer:** `scripts/lib/analyze_path_a_prefix_wire.py` (multi-turn Spec 10 checks).
4. **Public-entry e2e** asserts epoch stamp file after launch.

## Commands

```bash
./scripts/check-path-a-linkage.sh
# PASS (assemble_path_a_context no longer orphan)

cargo test -p dsb-context path_a     # 5 passed
cargo test -p dsb-context skills     # 7 passed
cargo test -p dsb-cli stamp_path_a   # 1 passed
cargo test -p dsb-cli resume         # 6 passed
cargo test -p dsb-agent session      # 4 passed (incl. load_repairs_unpaired_tool_calls)

python3 scripts/lib/analyze_path_a_prefix_wire.py \
  docs/product/evidence/PATH_A_R0_G008_PREFIX_WIRE_last.jsonl
# PASS (5 deepseek turns; system + skills stable; volatile grows)

./scripts/test-path-a-public-entry-e2e.sh
# PASS + path_a_prefix_epoch stamp present
```

## Artifacts

| Path | Role |
|------|------|
| [`PATH_A_R0_G008_PREFIX_WIRE_last.jsonl`](./PATH_A_R0_G008_PREFIX_WIRE_last.jsonl) | Multi-turn Path A wire (post-G004 liveness capture) |
| [`PATH_A_R0_G008_PREFIX_ANALYSIS.json`](./PATH_A_R0_G008_PREFIX_ANALYSIS.json) | Analyzer report |
| [`PATH_A_R0_G008_PREFIX_META_last.txt`](./PATH_A_R0_G008_PREFIX_META_last.txt) | Commands + stamp meta |
| [`PATH_A_R0_G008_EPOCH_last.txt`](./PATH_A_R0_G008_EPOCH_last.txt) | Live stamp from public entry |
| [`PATH_A_PREFIX_EPOCH_last.txt`](./PATH_A_PREFIX_EPOCH_last.txt) | Same stamp (e2e copy) |

## Explicit non-claims

- Flash/Pro routing wire = **G009**.
- L3 parallel/subagent/worktree = **G010**.
- Full ledger greening / cut = **G012** (`test-owner-bar` remains RED until all P0 PASS).
- Grok may still put calendar date in **user_info** (volatile head); Spec 10 system/prefix via `assemble_path_a_context` remains clock-free.
- Complete agent-binary embed of dsb-context (F1) remains cut-time; this story proves **production call site** + wire stability, not Cargo graph fusion.
