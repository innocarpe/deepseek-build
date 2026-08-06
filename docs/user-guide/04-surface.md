# 04 — Surface (skills + model UX)

**Product version:** `0.10.0`+ (skills list + opt-out; index since **0.6.0**)

## Skills

| Layer | What |
|-------|------|
| Stable prefix | **Index only** — name + short description from each `skills/*/SKILL.md` |
| On demand | Model calls tool **`skill`** with `{ "name": "…" }` to load full body |
| CLI | `deepseek-build skills list` (or `dsb skills list`) |

Roots scanned (later overrides same name):

1. `{workspace}/skills/`
2. `{workspace}/.deepseek-build/skills/`
3. `~/.deepseek-build/skills/` (if present)

Opt out of the index with frontmatter:

```yaml
---
description: Internal only
disable-model-invocation: true
---
```

Loading a skill mid-session does **not** rebuild the stable prefix / cache epoch.

## Thinking & effort

| Flag | Effect |
|------|--------|
| `--effort low\|high\|max` | Override reasoning effort for the process |
| `--thinking` | Force thinking on |
| `--no-thinking` | Disable thinking for the process |
| (default) | Thinking on; effort from preset/model (high / max on sticky Pro) |

Visibility line each turn:

```text
model=deepseek-v4-flash thinking=on effort=high
```

REPL: `/pro`, `/flash`, `/preset …`, `/model` (status via next-turn visibility).
