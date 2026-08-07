# 11 — Subagents (full-screen agent)

**Applies to:** `deepseek-build` / `dsb` → product agent (Grok-derived TUI)  
**Not the same as:** thin `dsb-tools` in-process `subagent` helper  
**Upstream detail:** vendored guide under `third_party/grok-build/`  
**Evidence:** Path A hermetic R0A **VC011** · re-prove on L3 cut **VC013** (**5.4.0** on-branch)

## What it is

Subagents are **child agent sessions** with their own context. The parent can
delegate research / implementation work and get a structured result back without
stuffing the main transcript with every intermediate tool call.

Enabled by default in the agent. Disable for a session:

```bash
dsb agent -- --no-subagents
# or headless:
deepseek-build-agent -p "…" --no-subagents
# product entry also accepts agent flags after agent subcommand:
deepseek-build agent -- -p "…" --no-subagents
```

```toml
# ~/.deepseek-build/config.toml (product home = GROK_HOME for the agent)
[subagents]
enabled = false
```

```bash
export GROK_SUBAGENTS=0
```

## Built-in types (typical Path A)

| Type | Role |
|------|------|
| `general-purpose` | Full tools (implement-class) |
| `explore` | Read/search oriented; no edits |
| `plan` | Planning; no edits |

The model uses a **spawn / subagent** tool (commonly `spawn_subagent`; aliases
may appear as `Agent` / `Task` depending on vendor surface).

Optional isolation: `isolation=worktree` may be passed on spawn — **optional**,
not forced for every implement worker (Spec 60 non-goal). See
[13-worktrees.md](./13-worktrees.md).

## Path A dogfood (what is proven)

| Requirement | Status |
|-------------|--------|
| Explore subagent on public Path A | **Proven** hermetic R0A (VC011 / VC013) |
| Implement-class subagent mutation | **Proven** hermetic R0A (disk mutation) |
| Worker reuses parent stable-prefix epoch | **Proven** stamp `worker_epochs_match=true` |
| Worker mutation **expires parent snippet table** on Path A | **Residual V3-60-3** — thin-path unit support only; **not** Path A sole green |

Live API spawn dogfood is available when a DeepSeek key is present:

```bash
./scripts/test-l3-smoke.sh --extended   # L3.5 spawn_subagent explore (env-gated)
```

Hermetic public-entry matrix (no live key required for scripted wire):

```bash
./scripts/test-path-a-vc011-r0a.sh
```

Evidence:

- [`VC011_SUBAGENT_WORKER_CACHE_PATH_A_2026-08-08.md`](../product/evidence/VC011_SUBAGENT_WORKER_CACHE_PATH_A_2026-08-08.md)
- [`VC013_L3_5_4_0_CUT_2026-08-08.md`](../product/evidence/VC013_L3_5_4_0_CUT_2026-08-08.md)

## Honesty

- Subagents are **product Path A machinery** under hearts — not a “future 4.0.0 plan.”
- On-branch vision packaging for the L3 train is **5.4.0** (unmerged stack); live
  `main` / npm / GitHub Latest may still be **5.2.2** until the stack merges and
  the release lane publishes.
- Do not claim **V3-60-3** closed without fresh Path A proof.

## Related

- [12-background-tasks.md](./12-background-tasks.md)  
- [13-worktrees.md](./13-worktrees.md)  
- [10-tools.md](./10-tools.md) (thin-path tools; different surface)  
- [KNOWN_LIMITS.md](../product/KNOWN_LIMITS.md)
