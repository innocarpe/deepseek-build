# 11 — Subagents (full-screen agent)

**Applies to:** `dsb` / `deepseek-build` → `deepseek-build-agent` (Grok-derived TUI)  
**Not the same as:** 1.x thin `dsb-tools` in-process `subagent` helper  
**Upstream detail:** vendored guide `third_party/grok-build/.../16-subagents.md`  
**Product plan:** L3 product defaults → **4.0.0** ([PRD-v4](../product/PRD-v4.md)); hearts → **3.0.0**

## What it is

Subagents are **child agent sessions** with their own context. The parent can
delegate research / planning work and get a summary back without stuffing the
main transcript with every intermediate tool call.

Enabled by default in the agent. Disable for a session:

```bash
dsb agent -- --no-subagents
# or headless:
deepseek-build-agent -p "…" --no-subagents
```

```toml
# ~/.deepseek-build/config.toml (product home = GROK_HOME for the agent)
[subagents]
enabled = false
```

```bash
export GROK_SUBAGENTS=0
```

## Built-in types (typical)

| Type | Role |
|------|------|
| `general-purpose` | Full tools |
| `explore` | Read/search oriented; no edits |
| `plan` | Planning; no edits |

The model uses a **spawn / subagent** tool (name may appear as `spawn_subagent`
or similar in the tool list).

## Product honesty (2.x / pre-4.0)

- Capability is **in the machine** (vendored Grok agent).  
- DeepSeek Build has **not** yet made “fleet-first subagent UX” the product
  identity — that is **4.0.0** ([PARALLEL_3X_4X_PLAN](../product/PARALLEL_3X_4X_PLAN.md)).  
- Verify live with: `./scripts/test-l3-smoke.sh --extended`

## Related

- [12-background-tasks.md](./12-background-tasks.md)  
- [13-worktrees.md](./13-worktrees.md)  
- [10-tools.md](./10-tools.md) (thin-path tools; different surface)
