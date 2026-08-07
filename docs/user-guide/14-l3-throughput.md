# 14 — Throughput (L3 overview)

How DeepSeek Build gets **wall-clock progress** out of the Grok-derived agent
**without** claiming 4.0.0 is done.

## Stack

```text
dsb (TTY)
  → deepseek-build-agent
       → DeepSeek models (chat_completions, api.deepseek.com)
       → L3 machine: parallel tools, bg shell, subagents, worktrees
       → L1/L2 hearts: 3.0.0 train (not fully fused in 2.x)
```

## Guides

| Topic | Doc |
|-------|-----|
| Subagents | [11-subagents.md](./11-subagents.md) |
| Background tasks | [12-background-tasks.md](./12-background-tasks.md) |
| Worktrees | [13-worktrees.md](./13-worktrees.md) |
| Thin-path tools | [10-tools.md](./10-tools.md) |

## Verify on your machine

```bash
export PATH="$HOME/.deepseek-build/bin:$PATH"
./scripts/test-l3-smoke.sh              # core
./scripts/test-l3-smoke.sh --extended   # + subagent (slower, more tokens)
```

Requires DeepSeek API key / credentials. Does **not** build vendor `target/`
test trees.

## Roadmap

| Version | Throughput story |
|---------|------------------|
| **2.x** | Machine present; product still “single session” feeling |
| **3.0.0** | Hearts on agent path (safe edit / perms / cache) |
| **4.0.0** | L3 as **product defaults** + evidence ([PRD-v4](../product/PRD-v4.md)) |
