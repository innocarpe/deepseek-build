# 13 — Git worktrees (full-screen agent)

**Applies to:** public `deepseek-build` / `dsb` (primary) → agent binary
**Alias:** both commands are the same product CLI; agent flags are forwarded via
`deepseek-build agent …` / `dsb agent …` (or bare TTY launch).
**Evidence:** Path A dogfood **VC012** · re-prove on L3 cut **VC013** (**5.4.0** on-branch)

## Honesty (fail-close)

| Claim | Reality |
|-------|---------|
| Bare `dsb` / `deepseek-build` (TTY, no flags) | **Single-session** full-screen TUI — does **not** auto-create a git worktree |
| Worktree create | **Opt-in** via `--worktree` / interactive flows — never mandatory for implement workers |
| Headless `-p` + `--worktree` | **Does not create** a worktree (flag ignored for create; session runs in `--cwd`) |
| Subagent isolation | Path A `spawn_subagent` may pass `isolation=worktree` **optionally**; default is shared workspace |
| Interactive TTY worktree **create** after process `exec` | **Residual** — not asserted as sole Path A green (VC012 / VC013) |

Public launch also stamps product home `path_a_l3.txt` with `worktree_product=opt_in` and
`bare_dsb_session=single` (see Path A L3 stamp / VC012 evidence).

## CLI (public entry)

```bash
# Product top-level opt-in (parsed by deepseek-build/dsb, forwarded to agent)
dsb --worktree feat-foo
deepseek-build --worktree feat-foo
dsb -w feat-foo
dsb --worktree feat-foo --worktree-ref main

# Same flags after the agent subcommand (product still parses globals; also valid:)
dsb agent -- --worktree feat-foo

# Headless: --worktree is accepted but does NOT create a worktree
dsb --worktree feat-foo agent -p "…" --cwd .

# Manage worktrees / agent help (public entry)
deepseek-build agent -- --help
dsb agent worktree --help
dsb agent worktree list --json

# Raw agent bin still works when installed:
deepseek-build-agent --worktree=feat-foo
deepseek-build-agent worktree --help
```

**Actual syntax (honesty):** product CLI owns top-level `--worktree` / `-w` /
`--worktree-ref` on bare TTY and `agent` paths only (rejected on `run`/`chat`).
Agent trailing flags remain valid. Headless `-p` never creates a worktree from
the flag.

Useful flags:

| Flag | Owner | Role |
|------|-------|------|
| `-w, --worktree [NAME]` | product (forwarded) + agent | Start in a new git worktree (**interactive**; ignored for create under `-p`) |
| `--worktree-ref REF` | product (forwarded) + agent | Base branch/tag/commit (with `--worktree`) |
| `--restore-code` | agent | With resume of remote session, apply snapshot codebase |

Config (product home `~/.deepseek-build/config.toml`):

```toml
# Examples from upstream; keys may evolve with vendor pin
# new_session_worktree_mode = "never"   # ask | always | never
# fork_worktree_mode = "ask"
```

## Smoke / Path A R0A

```bash
./scripts/test-l3-smoke.sh --offline-only
# L3.0/L3.4: agent help lists worktree; worktree --help available

# Public-entry Path A dogfood (VC012, re-proven VC013):
# CLI surface + product flag-forward (stub argv) + opt-in stamp + headless no-create
./scripts/test-path-a-vc012-r0a.sh
# Residual: interactive TTY worktree create after process exec is not asserted
```

Evidence:

- [`VC012_WORKTREE_DOGFOOD_PATH_A_2026-08-08.md`](../product/evidence/VC012_WORKTREE_DOGFOOD_PATH_A_2026-08-08.md)
- [`VC013_L3_5_4_0_CUT_2026-08-08.md`](../product/evidence/VC013_L3_5_4_0_CUT_2026-08-08.md)

## Related

- [11-subagents.md](./11-subagents.md)
- [12-background-tasks.md](./12-background-tasks.md)
- [14-l3-throughput.md](./14-l3-throughput.md)
- [KNOWN_LIMITS.md](../product/KNOWN_LIMITS.md) — residual list
