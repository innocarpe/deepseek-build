# G010 L3UnderHearts — evidence (owner-bar-5x)

| Field | Value |
|-------|--------|
| **Story** | G010 L3UnderHearts |
| **WAVE** | **5x-H3-1**, **5x-H3-2** |
| **Date** | 2026-08-07 |
| **Depends** | Hearts G004–G009 |

## Done criteria

| ID | Check | Result |
|----|-------|--------|
| **L3-50-1** | RO parallel / mutate serial | **PASS** unit `partition_indices` + production L3 stamp |
| **L3-50-2** | Fail-closed unknown/bash/MCP | **PASS** unit + stamp `bash/mcp/unknown_mutating=true` |
| **L3-50-3** | Background shell + collect-by-id | **PASS** product surface (help/docs/tools); live probe env-gated residual |
| **L3-50-4** | Bg/wait without secret flags | **PASS** documented product flags; no secret flags required |
| **L3-60-1** | Subagents default-on | **PASS** config seed `[subagents] enabled = true` + stamp |
| **L3-60-2** | Explore + implement kinds | **PASS** unit + stamp kinds |
| **L3-60-3** | Worker cache law | **PASS** `worker_stable_prefix` dual-build same epoch (unit + stamp) |
| **L3-60-4** | Worker mutation invalidates parent snippets | **PASS** unit `implement_write_mutates` + `parent_after_worker` |
| **L3-WT-1** | Worktree dogfoodable | **PASS** offline L3.0/L3.4 help + user-guide 13 |
| **L3-WT-2** | Honesty if worktree opt-in | **PASS** KNOWN_LIMITS + stamp `worktree_product=opt_in` |
| **L1-90-5** | Parallel/subagent cannot skip perms | **PASS** explore deny-write unit |
| **Heart regression** | Full L1/L2 re-run under L3 | **PASS** `./scripts/test-heart-regression.sh --with-e2e` |

## What shipped

1. **`stamp_path_a_l3`** in `agent_launch` — production call site for Spec 50 classifier + Spec 60 worker prefix.
2. Writes `path_a_l3.txt` under product home on every agent launch.
3. **`scripts/test-heart-regression.sh`** — linkage + context/agent/cli hearts + L3 offline + optional Path A e2e.
4. **`find_agent_bin`** prefers a *usable* agent (`--help` has `worktree`) so hollow `deepseek-build-agent` installs do not false-fail L3 smoke.
5. Public-entry e2e asserts L3 stamp content (epochs match, bash mutating, subagents enabled).

## Commands

```bash
cargo test -p dsb-cli stamp_path_a
cargo test -p dsb-agent parallel subagent path_a routing
./scripts/check-path-a-linkage.sh
./scripts/test-l3-smoke.sh --offline-only
./scripts/test-heart-regression.sh --with-e2e
./scripts/test-path-a-public-entry-e2e.sh
# path_a_l3 stamp present; worker_epochs_match=true
```

## Artifacts

| Path | Role |
|------|------|
| [`PATH_A_L3_last.txt`](./PATH_A_L3_last.txt) | Live L3 stamp from public entry |
| [`PATH_A_R0_G010_L3_last.txt`](./PATH_A_R0_G010_L3_last.txt) | G010 copy |
| [`PATH_A_R0_G010_HEART_REGRESSION_last.tsv`](./PATH_A_R0_G010_HEART_REGRESSION_last.tsv) | Heart regression matrix |
| [`PATH_A_R0_G010_META_last.txt`](./PATH_A_R0_G010_META_last.txt) | Meta |
| [`_last_l3_smoke.tsv`](./_last_l3_smoke.tsv) | Offline L3 smoke |

## Explicit non-claims

- Live multi-tool concurrent dogfood / live spawn_subagent still need API key (`--extended`).
- Worktree remains **opt-in** (`--worktree`); bare `dsb` is single-session (product choice).
- Full ledger greening = G012; install dual CLI = G011.
