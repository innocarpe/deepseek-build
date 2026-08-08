# 12 — Background tasks (full-screen agent)

**Applies to:** product agent tool loop (`deepseek-build` / `dsb` → agent)
**Upstream:** vendored background-task guide in `third_party/grok-build/`
**Evidence:** Path A hermetic R0A **VC010** · re-prove on L3 cut **VC013** (**5.4.0** on-branch)

## What it is

Long shell (and related) work can run **without blocking** the agent turn:

1. Model starts a terminal command with **background** enabled.
2. Agent receives a **task / job id**.
3. Later: collect / poll / wait by id.
4. Optional: kill the background task.

Interactive TUI: **Ctrl+B** often backgrounds the current foreground command
(upstream behavior; some deep strings may still say “Grok”).

## Thin path vs agent path

| Path | Background surface |
|------|--------------------|
| `dsb run` / thin tools (`dsb-tools`) | `bash` + `background: true` → collect tool (overlay names) |
| Full-screen agent (Path A) | Grok managed tools (`run_terminal_command` / task output helpers — **names may vary**) |

Do not assume the same tool **names** on both paths. Path A vision proof is
**collect-by-id** dogfood on the public agent entry, not thin-path alone.

## Path A dogfood (what is proven)

| Requirement | Status |
|-------------|--------|
| Background shell + collect-by-id on public Path A | **Proven** hermetic R0A (VC010 / VC013) |
| Multi-tool read-only parallel in the same train | **Proven** with mutate-serial (VC010 / VC013) |

```bash
# Hermetic public-entry (scripted DeepSeek wire)
./scripts/test-path-a-vc010-r0a.sh

# Live/offline L3 smoke (installed agent)
./scripts/test-l3-smoke.sh
# L3.2 expects the model to use background shell when key present
```

Evidence:

- [`VC010_L3_MULTI_TOOL_BG_PATH_A_2026-08-08.md`](../product/evidence/VC010_L3_MULTI_TOOL_BG_PATH_A_2026-08-08.md)
- [`VC013_L3_5_4_0_CUT_2026-08-08.md`](../product/evidence/VC013_L3_5_4_0_CUT_2026-08-08.md)

## Honesty

- Background capability is **available** under DeepSeek when models use
  `base_url = https://api.deepseek.com`.
- This is **shipped Path A machinery** under the vision L3 train — not a residual
  “wait for 4.0.0” story.
- On-branch cut **5.4.0** packages the L3 R0A train; live registry/GitHub may lag
  until merge + human-gated publish.

## Related

- [11-subagents.md](./11-subagents.md)
- [13-worktrees.md](./13-worktrees.md)
- [14-l3-throughput.md](./14-l3-throughput.md)
- [KNOWN_LIMITS.md](../product/KNOWN_LIMITS.md)
