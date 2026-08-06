# 02 — Dogfood profile

**Product version:** `0.3.0`+  
**Commands:** `deepseek-build` · `dsb`

## When to use

Local, trusted workspace where you want the agent to **edit files** and **run shell** without re-typing obscure flags every turn — still **not** YOLO outside the repo.

## One flag

```bash
deepseek-build --dogfood chat
# or
dsb --dogfood run "add a unit test for X"
```

### What it enables

1. **Workspace write** — `write` / `edit` / in-cwd delete scopes allowed without interactive ask (headless).  
2. **Bash execute** — `bash` tool runs commands (subject to classifier + deny list).  
3. **Still denied** — write/delete **outside** the workspace root.

### Without `--dogfood`

| Flag | Effect |
|------|--------|
| (default) | Read/grep OK; write needs ask→deny in headless; bash is dry-run |
| `--allow-workspace-write` | Mutating file tools in-cwd allowed |
| `--bash-execute` | Bash actually runs when policy allows |

## Tools available

| Tool | Role |
|------|------|
| `read` | File read + `snippet_id` for edit |
| `edit` | Snippet-scoped edit (spec 45) |
| `write` | Create new file only |
| `grep` | Literal text search in workspace |
| `bash` | Shell; declare `side_effects`; classifier authoritative |

## Limits (honest)

- No interactive “always allow” UX yet (headless ask → deny unless pre-allowed).  
- `grep` is literal substring (not full PCRE).  
- Network / mutate-git still default to ask→deny unless policy expanded later.
