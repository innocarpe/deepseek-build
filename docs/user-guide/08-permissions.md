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

## Opt-in: `--dangerously-allow-all` (disposable sandbox)

For throwaway sandboxes only (benchmark / CI containers):

```bash
dsb run --dangerously-allow-all --max-tool-rounds 200 "…"
```

The flag turns the scope gate off: every scope is allowed — network,
unclassified commands, write/delete outside the workspace — and bash executes.
Use it only where the sandbox boundary is the safety boundary; on a real
machine it protects nothing.

It is per-invocation: never a config key, and without it the fail-closed
defaults above are unchanged. `--dogfood`, the grants file, and the
full-screen TUI keep their own profiles.

## Tool-round cap

The native `run` / `chat` loop stops a turn after 8 tool rounds with
`max tool rounds reached`. `--max-tool-rounds N` raises that cap for long
headless runs; `0` is rejected and the default stays 8.

## Spec

Normative: `docs/specs/90-permissions.md`.
