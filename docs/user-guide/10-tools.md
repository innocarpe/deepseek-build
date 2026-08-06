# 10 — Tools

**Product version:** `0.16.0`+ (catalog evolved since 0.3.0)

## Built-in tools

| Tool | Role |
|------|------|
| `read` | Read file; issues `snippet_id` for edit |
| `edit` | Snippet-scoped replace (spec 45) |
| `write` | Create-new only |
| `grep` | Literal workspace search |
| `skill` | On-demand skill body |
| `bash` | Shell; optional `background: true` → `job_id` |
| `bash_collect` | Collect background job output |
| `plan` | Light non-blocking checklist |
| `subagent` | In-process explore/implement worker |
| `mcp__server__tool` | Dynamic MCP tools from catalog |

## CLI helpers

```bash
deepseek-build skills list
```

## Parallelism (0.12.0+)

Multiple **read-only** tools in one model turn may run concurrently (cap 8). Mutating tools run serially.

## Specs

40 (surface), 45 (snippet), 50 (parallel/bg), 60 (subagents), 70 (skills), 80 (MCP), 90 (permissions), 110 (plan).
