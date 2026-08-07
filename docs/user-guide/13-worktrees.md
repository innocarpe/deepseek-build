# 13 — Git worktrees (full-screen agent)

**Applies to:** public `deepseek-build` / `dsb` (primary) → agent binary  
**Alias:** both commands are the same product CLI; agent flags are forwarded via
`deepseek-build agent …` / `dsb agent …` (or bare TTY launch).  
**Product plan:** worktree as L3 surface → **4.0.0+** (vision dogfood **VC012** / **V3-WT**)

## Honesty (fail-close)

| Claim | Reality |
|-------|---------|
| Bare `dsb` / `deepseek-build` (TTY, no flags) | **Single-session** full-screen TUI — does **not** auto-create a git worktree |
| Worktree create | **Opt-in** via `--worktree` / interactive flows — never mandatory for implement workers |
| Headless `-p` + `--worktree` | **Does not create** a worktree (flag ignored for create; session runs in `--cwd`) |
| Subagent isolation | Path A `spawn_subagent` may pass `isolation=worktree` **optionally**; default is shared workspace |

Public launch also stamps product home `path_a_l3.txt` with `worktree_product=opt_in` and
`bare_dsb_session=single` (see Path A L3 stamp / VC012 evidence).

## CLI (public entry)

```bash
# Interactive session in a new worktree (name optional) — bare TTY product path
dsb --worktree feat-foo
deepseek-build --worktree feat-foo
# short form:
dsb -w feat-foo

# Or forward through the agent subcommand (same public CLI family)
dsb agent -- --worktree feat-foo
deepseek-build agent -- --worktree feat-foo

# Forward agent help / worktree subcommand through the public CLI
deepseek-build agent -- --help          # agent flags (includes --worktree)
dsb agent worktree --help               # manage subcommand
dsb agent worktree list --json          # list tracked worktrees

# Raw agent bin still works when installed:
deepseek-build-agent --worktree=feat-foo
deepseek-build-agent worktree --help
```

Useful agent flags (see `deepseek-build agent -- --help`):

| Flag | Role |
|------|------|
| `-w, --worktree [NAME]` | Start in a new git worktree (**interactive**; ignored for create under `-p`) |
| `--worktree-ref REF` | Base branch/tag/commit (with `--worktree`) |
| `--restore-code` | With resume of remote session, apply snapshot codebase |

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

# Public-entry Path A dogfood (VC012, conservative bounded):
# CLI surface + product flag-forward (stub argv) + opt-in stamp + headless no-create
./scripts/test-path-a-vc012-r0a.sh
# Residual: interactive TTY worktree create after process exec is not asserted
```

Evidence: [`docs/product/evidence/VC012_WORKTREE_DOGFOOD_PATH_A_2026-08-08.md`](../product/evidence/VC012_WORKTREE_DOGFOOD_PATH_A_2026-08-08.md)

## Related

- [11-subagents.md](./11-subagents.md)  
- [12-background-tasks.md](./12-background-tasks.md)  
- [KNOWN_LIMITS.md](../product/KNOWN_LIMITS.md) — bare session residual  
- Parallel ops: [PARALLEL_3X_4X_PLAN.md](../product/PARALLEL_3X_4X_PLAN.md)
