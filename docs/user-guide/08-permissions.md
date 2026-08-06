# 08 — Permissions

**Product version:** `0.9.0`+ (interactive), documented **0.16.0**

## Defaults (fail-closed)

| Scope class | Default |
|-------------|---------|
| Read in workspace | allow |
| Write/delete in workspace | **ask** (TTY) / **deny** (headless) |
| Write/delete outside workspace | **deny** (cannot grant always) |
| Network / mutate-git / unknown | ask → deny headless |

## Interactive ask (TTY `chat`)

When a tool needs confirmation:

```text
[permission] scopes need approval: write-in-cwd
  [a] allow once   [A] allow always   [d] deny
```

| Choice | Effect |
|--------|--------|
| `a` | Allow this call; remember for session |
| `A` | Persist under `~/.deepseek-build/permission-grants.json` (0600) |
| `d` | Deny |

`--dogfood` and `--allow-workspace-write` skip ask for in-cwd write/delete (still deny out-of-cwd).

## Spec

Normative: `docs/specs/90-permissions.md`.
