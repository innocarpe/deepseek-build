# 12 — Background tasks (full-screen agent)

**Applies to:** `deepseek-build-agent` tool loop  
**Upstream:** vendored `20-background-tasks.md`  
**Product plan:** first-class throughput defaults → **4.0.0**

## What it is

Long shell (and related) work can run **without blocking** the agent turn:

1. Model calls `run_terminal_cmd` / `run_terminal_command` with **background** enabled.  
2. Agent receives a **task id**.  
3. Later: `get_command_or_subagent_output` / `get_task_output` to poll or wait.  
4. Optional: `kill_command_or_subagent` to stop work.

Interactive TUI: **Ctrl+B** often backgrounds the current foreground command
(upstream behavior; product chrome may still say “Grok” in deep strings).

## Thin path vs agent path

| Path | Background |
|------|------------|
| `dsb run` / thin tools (`dsb-tools`) | `bash` + `background: true` → `bash_collect` (1.x overlay) |
| Full-screen agent | Grok managed tools (`run_terminal_cmd`, task output, …) |

Do not assume the same tool **names** on both paths.

## Smoke

```bash
./scripts/test-l3-smoke.sh
# L3.2 expects the model to use background shell and print bg-ok-77
```

## Honesty

Capability is **available** under DeepSeek when the agent is configured with
`base_url = https://api.deepseek.com` on model stanzas. Product **defaults**
and docs identity for “always use bg for long builds” are **4.0.0** work.

## Related

- [11-subagents.md](./11-subagents.md)  
- [13-worktrees.md](./13-worktrees.md)
