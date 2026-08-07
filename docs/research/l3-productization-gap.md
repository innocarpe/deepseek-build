# L3 productization gap inventory (prep for 4.0.0)

**Status:** Living inventory — **executed prep** (docs + smoke), not a ship claim  
**Normative ops:** [PARALLEL_3X_4X_PLAN.md](../product/PARALLEL_3X_4X_PLAN.md)  
**WAVE unit:** 4x-P0-3 / 4x-P0-4  
**Last updated:** 2026-08-07  
**Smoke:** `./scripts/test-l3-smoke.sh` · evidence `docs/product/evidence/L3_SMOKE_*.md`

Purpose: Grok **L3** already in the vendored machine vs DeepSeek Build **product** surface.

Do **not** change product defaults in this prep track (Lane B). Default flips wait for **4.0.0** after hearts.

---

## Legend

| Column | Meaning |
|--------|---------|
| **In vendor** | Present in `third_party/grok-build` / agent bin |
| **Product default** | On without exotic flags for DSB users |
| **Documented (DSB)** | `docs/user-guide/*` product docs |
| **Dogfooded** | Live under DeepSeek (`base_url` on model) |
| **Code pointers** | Where to look (vendor) |

---

## Matrix

| Capability | In vendor | Product default | Documented (DSB) | Dogfooded | Code / CLI pointers | 4.0 action |
|------------|-----------|-----------------|------------------|-----------|---------------------|------------|
| Headless `-p` | yes | opt-in CLI | [14](../user-guide/14-l3-throughput.md) | **yes** (L3.1 / T4) | `xai-grok-pager` headless; `deepseek-build-agent -p` | polish |
| Background shell | yes | model-driven | [12](../user-guide/12-background-tasks.md) | **L3.2 smoke** | `xai-grok-tools` `run_terminal_cmd`; task output tools | defaults + evidence |
| Subagents | yes (default on) | on, not “fleet UX” | [11](../user-guide/11-subagents.md) | **L3.5** `--extended` | `spawn_subagent`; `--no-subagents`; `[subagents]` | product dogfood + docs |
| Worktree sessions | yes | opt-in `--worktree` | [13](../user-guide/13-worktrees.md) | **L3.4** help; interactive create TBD | `--worktree`; `worktree` subcmd; config `*_worktree_mode` | dogfood create path |
| Parallel tool runs | yes | TBD | partial | partial | tool runtime / agent loop | matrix + defaults |
| MCP | yes | TBD | thin MCP docs | no | `xai-grok-mcp` | later if not P0 |
| Skills | yes | TBD | surface docs | no | skill discovery under tools | 3.x / 4.x |
| Leader / multi-session | yes | no | no | no | leader socket paths | not 4.0 P0 unless promoted |
| Permissions product | Grok modes | residual | 08-permissions (thin) | T5.8 | **3.0 owns agent path** | after hearts |
| Snippet-safe edit | Grok edit + thin strong | agent residual | honesty KNOWN_LIMITS | thin / 3.0 | **3.0 owns** `search_replace` path | after hearts |

---

## Vendor path cheat sheet (pin may move with SOURCE_REV)

| Area | Path (under `third_party/grok-build/`) |
|------|----------------------------------------|
| Agent composition root | `crates/codegen/xai-grok-pager-bin` |
| Headless | `crates/codegen/xai-grok-pager/src/headless*.rs` |
| Worktree CLI | `xai-grok-pager-bin/src/main.rs` → `Command::Worktree` → `xai_grok_pager::worktree_cmd` |
| Tools (bash/edit/…) | `crates/codegen/xai-grok-tools/src/implementations/grok_build/` |
| Subagent / task spawn | `…/grok_build/task/mod.rs` (`run_in_background`, background spawn) |
| Tool name registry | `xai-grok-telemetry/.../schema.rs` includes `"spawn_subagent"` |
| Subagent resolution | `crates/codegen/xai-grok-subagent-resolution/` |
| Shell / session | `crates/codegen/xai-grok-shell/` |
| Upstream user guides | `…/docs/user-guide/16-subagents.md`, `20-background-tasks.md` |

Product install name: `~/.deepseek-build/bin/deepseek-build-agent` (GROK_HOME = product home).

---

## Smoke commands

```bash
export PATH="$HOME/.deepseek-build/bin:$PATH"
./scripts/test-l3-smoke.sh
./scripts/test-l3-smoke.sh --extended
```

---

## Open questions (4.0 finalize)

1. Default profile after hearts — single-session vs throughput-first?  
2. Worktree: flag-only vs `/new` prompt product default?  
3. MCP in 4.0.0 P0 or minor?  
