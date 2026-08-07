# L3 productization gap inventory (prep for 4.0.0)

**Status:** Living inventory — fill during heart-3x; **not** a ship claim  
**Normative ops:** [PARALLEL_3X_4X_PLAN.md](../product/PARALLEL_3X_4X_PLAN.md)  
**WAVE unit:** 4x-P0-3  
**Last updated:** 2026-08-07

Purpose: list Grok **L3-class** capabilities already in the vendored machine vs what DeepSeek Build treats as **product defaults / docs / dogfood**.

Do **not** change product defaults here. Evidence-only.

---

## Legend

| Column | Meaning |
|--------|---------|
| **In vendor** | Present in `third_party/grok-build` (or product agent) |
| **Product default** | On by default for DeepSeek Build users without exotic flags |
| **Documented** | User-facing DeepSeek Build docs (not only upstream Grok guide) |
| **Dogfooded on DeepSeek** | Live evidence under `api.deepseek.com` path |
| **4.0 action** | docs / default / evidence / later |

---

## Matrix (initial)

| Capability | In vendor | Product default | Documented (DSB) | Dogfooded (DeepSeek) | 4.0 action |
|------------|-----------|-----------------|------------------|----------------------|------------|
| Parallel / multi tool calls | yes | TBD | partial | partial (T4 tools serial cases) | matrix + defaults |
| Background shell / task output | yes | TBD | no / weak | no | evidence + defaults |
| Subagent spawn | yes | TBD | weak | no (T5.2 skip) | dogfood + docs |
| Worktree isolation | yes | TBD | weak | no (T5.7 skip) | dogfood + docs |
| Headless `-p` scripting | yes | yes (when used) | partial | **yes** (T4/T5) | guide polish |
| MCP | yes | TBD | partial | no | later minor if not P0 |
| Skills | yes | TBD | partial | no | 3.x minor / 4.x as needed |
| Leader / multi-session | yes | no | no | no | out of 4.0.0 P0 unless promoted |
| Permissions product matrix | Grok modes | partial | partial | T5.8 smoke | **owned by 3.0** first |
| Snippet-safe edit | thin path strong | agent path residual | honesty in KNOWN_LIMITS | thin yes | **owned by 3.0** |

Update rows when heart-3x PRs reveal real injection points (paths under `xai-grok-tools`, shell, pager).

---

## Suggested evidence commands (no default mutation)

```bash
export PATH="$HOME/.deepseek-build/bin:$PATH"
# Headless only; hermetic GROK_HOME with DeepSeek base_url (see scripts/lib/common.sh)
./scripts/test-deepseek-live.sh --extended   # existing T5 stubs
# Manual follow-ups (record in docs/product/evidence/):
# deepseek-build-agent -p "…" --tools … --yolo --max-turns N
```

---

## Open questions for 4.0 finalize

1. Which agent **profile** is the product default after hearts?  
2. Is worktree **opt-in flag** enough for 4.0.0 P0, or must bare `dsb` teach fleet?  
3. How much MCP is P0 vs 4.x minor?  

Resolve in WAVE_4x ready-for-impl PR after `v3.0.0`.
