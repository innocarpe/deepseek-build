# 11 — Subagents (full-screen agent)

**Applies to:** `deepseek-build` / `dsb` → product agent (Grok-derived TUI)
**Not the same as:** thin `dsb-tools` in-process `subagent` helper
**Upstream detail:** vendored guide under `third_party/grok-build/`
**Evidence:** Path A hermetic R0A **VC011** · re-prove on L3 cut **VC013** · parent snippet after worker **VC015** (**V3-60-3**)

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
| Worker mutation **invalidates parent snippets** on Path A | **Proven** VC015 R0A: parent pre-mutation `snippet_id` → `snippet_stale` after implement-class mutates same path (Spec 45 version gate). Explicit parent `expire_all` after spawn remains optional product default, not required for this proof. |

Live API spawn dogfood is available when a DeepSeek key is present:

```bash
./scripts/test-l3-smoke.sh --extended   # L3.5 spawn_subagent explore (env-gated)
```

Hermetic public-entry matrix (no live key required for scripted wire):

```bash
./scripts/test-path-a-vc011-r0a.sh
./scripts/test-path-a-vc015-r0a.sh   # V3-60-3 parent snippet after worker
```

Evidence:

- [`VC011_SUBAGENT_WORKER_CACHE_PATH_A_2026-08-08.md`](../product/evidence/VC011_SUBAGENT_WORKER_CACHE_PATH_A_2026-08-08.md)
- [`VC013_L3_5_4_0_CUT_2026-08-08.md`](../product/evidence/VC013_L3_5_4_0_CUT_2026-08-08.md)
- [`VC015_VISION_FREEZE_5_5_0_2026-08-08.md`](../product/evidence/VC015_VISION_FREEZE_5_5_0_2026-08-08.md)

## Honesty

- Subagents are **product Path A machinery** under hearts — not a “future 4.0.0 plan.”
- **VC013** cut the L3 Path A R0A train as merged product **`5.4.0`** (history).
  **VC015** freeze packaging is merged on `main` as **`5.5.0`** (vision-complete
  freeze unit). npm / GitHub Latest may still be **`5.2.2`** until the release
  lane publishes.
- **V3-60-3** is closed by **VC015** Path A R0A (`snippet_stale` after implement-class
  mutates the same path). Do not re-open it without a regression. Explicit parent
  `expire_all` after spawn remains an optional Spec 60 honesty residual, not the
  freeze close bar.

## Related

- [12-background-tasks.md](./12-background-tasks.md)
- [13-worktrees.md](./13-worktrees.md)
- [10-tools.md](./10-tools.md) (thin-path tools; different surface)
- [KNOWN_LIMITS.md](../product/KNOWN_LIMITS.md)
