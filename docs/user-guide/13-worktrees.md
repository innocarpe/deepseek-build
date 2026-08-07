# 13 — Git worktrees (full-screen agent)

**Applies to:** `deepseek-build-agent` / `dsb` forwarded flags  
**Product plan:** worktree as product identity → **4.0.0**  
**Honesty:** Headless `-p` **does not** create a worktree from `--worktree` in
upstream behavior; interactive / CLI `worktree` subcommand does.

## CLI

```bash
# Interactive session in a new worktree (name optional)
dsb --worktree=feat-foo "implement the feature"
# or
deepseek-build-agent --worktree=feat-foo

# Manage worktrees
deepseek-build-agent worktree --help
```

Useful flags (see `deepseek-build-agent --help`):

| Flag | Role |
|------|------|
| `-w, --worktree [NAME]` | Start in a new git worktree |
| `--worktree-ref REF` | Base branch/tag/commit |
| `--restore-code` | With resume of remote session, apply snapshot codebase |

Config (product home `~/.deepseek-build/config.toml`):

```toml
# Examples from upstream; keys may evolve with vendor pin
# new_session_worktree_mode = "never"   # ask | always | never
# fork_worktree_mode = "ask"
```

## Smoke

```bash
./scripts/test-l3-smoke.sh
# L3.4 checks worktree --help is available
```

## Related

- [11-subagents.md](./11-subagents.md)  
- [12-background-tasks.md](./12-background-tasks.md)  
- Parallel ops: [PARALLEL_3X_4X_PLAN.md](../product/PARALLEL_3X_4X_PLAN.md)
