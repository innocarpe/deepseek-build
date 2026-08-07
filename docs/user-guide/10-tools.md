# 10 — Tools

**Commands:** `deepseek-build` (primary) · `dsb` (alias)  
**Primary path:** full-screen agent (**Path A**) — public CLI → product agent  
**Secondary path:** thin line-mode tools (`dsb run` / overlay `dsb-tools`) — different names and proofs

## Path A vs thin path (fail-close)

| Path | Entry | Edit safety | Proof that counts for vision |
|------|-------|-------------|------------------------------|
| **Path A** | bare `dsb` / `deepseek-build`, or `… agent` | Session **`snippet_id`** mint / require / invalidation (Spec 45) | Public-entry hermetic R0A + heart/owner-bar |
| **Thin** | `dsb run` / library tools | Overlay catalog; may use different tool ids | Support only — **not** sole Path A proof |

Do not assume the same tool **names** on both paths.

## Spec 45 snippet safety (Path A)

On Path A, safe edits are **snippet-scoped**, not free-form whole-file primary:

| Behavior | Reality |
|----------|---------|
| Read mints | Session-local **`snippet_id`** (with version / content binding) |
| Edit requires | Valid `snippet_id` for the target span (stale id fail-closed) |
| Write | Create-new only; overwrite uses the same safety class as edit |
| Bash / external mutation | Invalidates snippets for touched paths (content / version laws) |
| Free-form whole-file primary | Fail-closed when it would skip the snippet contract |

Compatibility: older **`file_version` (sha256)** wording remains a version alias in
hearts docs; Path A vision stack closed **snippet_id** multi-edit R0A on the
Deep Code cut (on-branch packaging **5.3.0** under live floor **5.2.2** — see
[KNOWN_LIMITS](../product/KNOWN_LIMITS.md)).

Evidence: [`docs/product/evidence/VC006_PATH_A_HEART_R0A_2026-08-08.md`](../product/evidence/VC006_PATH_A_HEART_R0A_2026-08-08.md)  
(plus VC003 mint · VC004 require · VC005 write/bash invalidation)

## Built-in surfaces (illustrative)

### Full-screen agent (Path A / Grok-derived)

Typical agent tools include read / search_replace (snippet-safe) / write /
terminal / spawn_subagent / background collect, MCP tools from catalog, and
skill load. Exact **names** follow the agent binary tool list for your install.

### Thin overlay (`dsb-tools` / line-mode)

| Tool | Role |
|------|------|
| `read` | Read file; may mint snippet / version for edit |
| `edit` | Snippet-scoped replace (Spec 45 spirit) |
| `write` | Create-new only |
| `grep` | Workspace search |
| `skill` | On-demand skill body |
| `bash` | Shell; optional `background: true` → collect tool |
| `plan` | Light non-blocking checklist |
| `subagent` | In-process explore/implement helper (**not** Path A `spawn_subagent`) |
| `mcp__server__tool` | Dynamic MCP tools from catalog |

## Parallelism (L3 / Spec 50 spirit)

On Path A, multiple **read-only** tools in one model turn may run concurrently
(product cap documented in L3 matrix / smoke). **Mutating** tools run **serially**.

Hermetic public-entry dogfood: multi-read parallel + mixed mutate serial
(vision **VC010**, re-proven on **5.4.0** cut **VC013**).

```bash
./scripts/test-path-a-vc010-r0a.sh
```

## CLI helpers

```bash
deepseek-build skills list
dsb skills list
```

## Specs

40 (surface), 45 (snippet), 50 (parallel/bg), 60 (subagents), 70 (skills),
80 (MCP), 90 (permissions), 110 (plan).

## Related

- [11-subagents.md](./11-subagents.md)  
- [12-background-tasks.md](./12-background-tasks.md)  
- [14-l3-throughput.md](./14-l3-throughput.md)  
- [KNOWN_LIMITS.md](../product/KNOWN_LIMITS.md)
