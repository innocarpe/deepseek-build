# L3 capability matrix (4.0.0 product)

**Status:** ready-for-impl / ship evidence for **4.0.0**  
**PRD:** [PRD-v4.md](./PRD-v4.md) · WAVE [WAVE_4x_PR_DAG.md](./WAVE_4x_PR_DAG.md)  
**Smoke:** `./scripts/test-l3-smoke.sh`  
**Hearts:** must remain green ([PRD-v3.md](./PRD-v3.md) Path A)

---

## Product surface (DeepSeek Build)

| Capability | Product entry | Default (4.0.0) | Docs | Smoke |
|------------|---------------|-----------------|------|-------|
| Full-screen agent | `dsb` / `deepseek-build` | on (TTY no-args) | user-guide | pre3x T2 |
| Headless scripting | `deepseek-build-agent -p` / `dsb agent -- -p` | opt-in | [14](../user-guide/14-l3-throughput.md) | L3.1 |
| Subagents | spawn tools; `--no-subagents` to disable | **enabled** (`[subagents] enabled = true`) | [11](../user-guide/11-subagents.md) | L3.3 / L3.5 |
| Background shell / tasks | `run_terminal_cmd` + task output tools | model-driven (tools present) | [12](../user-guide/12-background-tasks.md) | L3.2 |
| Worktrees | `--worktree`, `worktree` subcommand | opt-in CLI (not forced on bare `dsb`) | [13](../user-guide/13-worktrees.md) | L3.4 |
| Parallel tool calls | agent loop | vendor default | [14](../user-guide/14-l3-throughput.md) | heart + pre3x |
| L1 snippet / L2 prefix | Path A under Grok tools | **fail-closed hearts** (3.0.0) | KNOWN_LIMITS | path_a tests |

---

## Vendor code pointers (`third_party/grok-build/`)

| Area | Path |
|------|------|
| Composition root | `crates/codegen/xai-grok-pager-bin` |
| Headless | `crates/codegen/xai-grok-pager/src/headless*.rs` |
| Worktree CLI | `xai-grok-pager-bin` → `worktree_cmd` |
| Task / subagent spawn | `xai-grok-tools/.../grok_build/task/mod.rs` |
| Tool name `spawn_subagent` | telemetry schema / tools registry |
| Subagent resolution | `xai-grok-subagent-resolution` |

Product home: `~/.deepseek-build/` (agent `GROK_HOME`).

---

## Heart regression (must stay green)

```bash
./scripts/check-semver.sh
cargo test -p dsb-tools path_a
cargo test -p dsb-context path_a
cargo test -p dsb-agent path_a
./scripts/test-product-offline.sh
./scripts/test-l3-smoke.sh --offline-only
# optional live:
# ./scripts/test-pre3x-baseline.sh --live
# ./scripts/test-l3-smoke.sh --extended
```

---

## Non-goals (still)

- YOLO / always-approve as product default  
- Multi-vendor identity  
- Replacing Grok base  
