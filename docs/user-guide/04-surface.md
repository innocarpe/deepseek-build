# 04 — Surface (skills + model UX + cache)

**Commands:** `deepseek-build` (primary) · `dsb` (alias)
**Path:** default full-screen agent (**Path A**) unless noted

## Skills

| Layer | What |
|-------|------|
| Stable prefix | **Index only** — name + short description from each `skills/*/SKILL.md` |
| On demand | Model loads full skill body via the skill tool (name may differ thin vs agent) |
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

Loading a skill mid-session does **not** rebuild the stable prefix / cache epoch
when Spec 10 assembly is active on Path A turns.

## Thinking & effort

| Flag / control | Effect |
|----------------|--------|
| `--effort low\|high\|max` | Per-process override. Unknown values are rejected before launch; the flag never rewrites the saved default |
| `--model ID` / `-m ID` | Per-process model override. The TUI resolves the ID against its catalog; native `run`/`chat` accept `deepseek-v4-flash` or `deepseek-v4-pro` |
| `[models] default_reasoning_effort` (config.toml) | Standing default for the Flash slot; both paths (`dsb run` and the bare TUI) read it. `/effort` and `/model <name> <effort>` save a successful pick; CLI flags do not |
| `--thinking` | Force thinking on |
| `--no-thinking` | Disable thinking for the process |
| (default) | Thinking on; effort from product model seed / sticky Pro preset (shipped default `high`) |

Visibility line each turn (illustrative):

```text
model=deepseek-flash thinking=on effort=high
```

REPL: `/pro`, `/flash`, `/preset …`, `/model` (status via next-turn visibility).

### Another process with a model and effort

`--model` and `--effort` apply to the process that runs them. They do not
change `[models] default` or `default_reasoning_effort`; interactive `/effort`
and `/model` picks save the default for the next launch.

`deepseek-build` is the primary command; `dsb` is its short alias.

```bash
# Full-screen TUI with a catalog model and effort for this process
deepseek-build --model deepseek-v4-pro --effort high

# Native one-shot: IDs map to the Flash and Max routes
deepseek-build --model deepseek-v4-pro --effort high run "explain this repo"

# Headless through the vendored agent flags
dsb agent -- -m deepseek-v4-pro --reasoning-effort high -p "explain this repo"

# Workspace root is used by the native loop and forwarded to the TUI
deepseek-build --cwd /path/to/workspace --model deepseek-v4-flash --effort low

# Scripted run: model/effort plus a JSON result and a turn cap
dsb --cwd /path/to/workspace --model deepseek-v4-pro --effort high agent -- \
  -p "explain this repo" --output-format json --max-turns 12
```

`--pro` and `--model` choose the same route on `run`; pass one. The headless
`agent --` path keeps the vendored model menu, including its remapped effort
option IDs.

### Effort honesty (L2 / Path A)

| Claim | Reality |
|-------|---------|
| DeepSeek chat body can carry `reasoning_effort` | **Yes** on product Path A when product config seed/repair injects it (vision stack **VC008**) |
| Product model stanzas seed effort | Flash/Pro defaults include `supports_reasoning_effort`, an effort string, and the `reasoning_efforts` menu (`max`/`high`/`low`) the vendored `/effort` picker reads |
| CLI `--effort` override | Forwarded as agent `--reasoning-effort` on product TUI/agent path; process-scoped and never written back as the saved default. Hermetic wire proof is primarily the **default seed/repair** path |
| Full Spec 30 “thinking body field” on every Grok wire shape | **Not claimed** — Path A Grok chat-completions path may not carry a separate thinking object; do not over-read effort as full Spec 30 |

Evidence: [`docs/product/evidence/VC008_REASONING_EFFORT_WIRE_2026-08-08.md`](../product/evidence/VC008_REASONING_EFFORT_WIRE_2026-08-08.md)

## Stable prefix assembly (L2 / Path A)

On Grok Path A turns, the product applies a Spec 10–ordered **stable prefix**
layout (tools / skills index / environment / project instructions) to the
conversation system content before the model call (vision stack **VC007**).

| Claim | Reality |
|-------|---------|
| Assembly mutates the wire system content | **Yes** on Path A turn path (not stamp-only) |
| Skills index thrash-free under multi-turn | Index lives in stable prefix; body load on demand |
| Every historical installed binary has turn stamps | Soft e2e may warn if an old agent binary lacks the turn stamp — rebuild/agent pin matters |

Evidence: [`docs/product/evidence/VC007_SPEC10_ASSEMBLY_PATH_A_2026-08-08.md`](../product/evidence/VC007_SPEC10_ASSEMBLY_PATH_A_2026-08-08.md)

## Cache-hit signal (L2 / Path A)

When DeepSeek usage reports cache fields, Path A can surface:

| Surface | What |
|---------|------|
| User-visible | Bottom status chip style `cache N%` (pager format path) |
| Loggable | Product-home stamp `path_a_cache_signal.txt` under `DEEPSEEK_BUILD_HOME` (vision stack **VC009**) |

### Cache honesty

| Claim | Reality |
|-------|---------|
| Hermetic Path A can prove usage → stamp/chip path | **Yes** (fixture emits `prompt_cache_hit_tokens`) |
| Live DeepSeek always hits cache | **No** — provider policy; fixture ≠ live hit rate |
| Cache signal alone = cheap long sessions forever | **No** — signal is visibility, not a cost SLA |

Evidence: [`docs/product/evidence/VC009_CACHE_VISIBILITY_2026-08-08.md`](../product/evidence/VC009_CACHE_VISIBILITY_2026-08-08.md)

## Related

- [10-tools.md](./10-tools.md) — snippet-safe tools
- [14-l3-throughput.md](./14-l3-throughput.md) — L3 overview
- [KNOWN_LIMITS.md](../product/KNOWN_LIMITS.md)
