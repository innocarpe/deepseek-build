# 14 — Throughput (L3 overview)

How DeepSeek Build gets **wall-clock progress** out of the Grok-derived agent on
the public Path A entry — with honest limits.

**Commands:** `deepseek-build` (primary) · `dsb` (alias)

## Stack

```text
dsb / deepseek-build (TTY or agent subcommand)
  → deepseek-build-agent
       → DeepSeek models (chat_completions, api.deepseek.com)
       → L3: parallel tools, bg shell, subagents, opt-in worktrees
       → L1/L2 hearts: snippet_id, Spec 10 assembly, effort, cache signal
```

Bare `dsb` / `deepseek-build` is a **single-session** TUI. Worktree isolation is
**opt-in** (`--worktree`); headless `-p --worktree` does **not** create a worktree.

## Guides

| Topic | Doc |
|-------|-----|
| Tools / snippet safety | [10-tools.md](./10-tools.md) |
| Subagents | [11-subagents.md](./11-subagents.md) |
| Background tasks | [12-background-tasks.md](./12-background-tasks.md) |
| Worktrees | [13-worktrees.md](./13-worktrees.md) |
| Skills / effort / cache | [04-surface.md](./04-surface.md) |

## What Path A has proven (vision L3 train)

| ID | Capability | Evidence |
|----|------------|----------|
| **V3-50-1** | Multi-tool RO parallel + mutate serial | VC010 · re-prove VC013 |
| **V3-50-2** | Background shell + collect-by-id | VC010 · re-prove VC013 |
| **V3-60-1** | Explore + implement-class subagents | VC011 · re-prove VC013 |
| **V3-60-2** | Worker reuses parent stable-prefix epoch | VC011 stamp · re-prove VC013 |
| **V3-60-3** | Parent snippet invalidate after worker mutation | **Proven** VC015 Path A R0A (`snippet_stale` after implement-class mutates same path) |
| **V3-WT** | Worktree CLI dogfood + bare-session honesty | VC012 · re-prove VC013 |

**VC013** packaged the L3 Path A R0A train as merged product **`5.4.0`** (L3 cut
history). **VC015** freeze packaging is merged on `main` as **`5.5.0`** (vision
freeze unit; includes V3-60-3 close). `5.5.0` is not on npm/GitHub Latest until
the release lane publishes — see [KNOWN_LIMITS](../product/KNOWN_LIMITS.md).

Cut / freeze evidence:

- L3 train cut: [`VC013_L3_5_4_0_CUT_2026-08-08.md`](../product/evidence/VC013_L3_5_4_0_CUT_2026-08-08.md)
- Vision freeze: [`VC015_VISION_FREEZE_5_5_0_2026-08-08.md`](../product/evidence/VC015_VISION_FREEZE_5_5_0_2026-08-08.md)

## Verify on your machine

```bash
export PATH="$HOME/.deepseek-build/bin:$PATH"

# Offline help / flag surface (no API key)
./scripts/test-l3-smoke.sh --offline-only

# Live core (+ extended subagent) when credentials present
./scripts/test-l3-smoke.sh
./scripts/test-l3-smoke.sh --extended
```

Dev checkout — hermetic public-entry Path A R0A (scripted DeepSeek; rebuilds
agent as needed):

```bash
./scripts/test-path-a-vc010-r0a.sh
./scripts/test-path-a-vc011-r0a.sh
./scripts/test-path-a-vc012-r0a.sh
```

These do **not** require committing vendor `target/` trees. Owner-bar / heart
gates remain the product regression bar:

```bash
./scripts/test-owner-bar.sh
./scripts/test-heart-regression.sh
./scripts/check-path-a-linkage.sh
```

## Version line (honesty)

| Cut | Throughput story |
|-----|------------------|
| **2.x** | Shell + machine present; single-session product feel |
| **3.x tagged** | Heart fusion *attempt* — not owner-bar green |
| **4.x tagged** | L3 machinery *attempt* — not owner-bar green |
| **5.0.0** | Owner-bar complete product (Path A P0) |
| **5.2.x published** | npm/GitHub Latest may still be **5.2.2** until publish |
| **5.3.0 merged** | Spec 45 Path A `snippet_id` Deep Code cut |
| **5.4.0 merged** | L3 Path A R0A train cut — **VC013** history |
| **5.5.0 on main** | Vision freeze packaging — **VC015** (incl. V3-60-3 Path A R0A); release publish pending |

## Related

- [KNOWN_LIMITS.md](../product/KNOWN_LIMITS.md)
- Vision board: [VISION_COMPLETE_5X_GOALS.md](../product/VISION_COMPLETE_5X_GOALS.md)
