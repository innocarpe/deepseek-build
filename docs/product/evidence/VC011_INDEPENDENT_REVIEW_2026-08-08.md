# VC011 — independent Grok review (subagent + worker cache Path A R0A)

| Field | Value |
|-------|--------|
| **Story** | **VC011** — hermetic Path A R0A for explore + implement-class subagents, worker cache stamp, parent snippet invalidation honesty |
| **Branch / worktree** | `vc011-subagent-worker-cache` @ `/Users/WooseongKim/Projects/deepseek-build/vc011-subagent-worker-cache` |
| **Reviewed code head** | Implementation tip prior to this review package (`a549c35` unit support + harness/fixture commits; READY docs co-land) |
| **Base** | `vc010-l3-parallel-bg` @ `9d6f1d0` (open PR **#142**) |
| **Reviewer** | Independent Grok review lane (static + re-ran R0A/gates on this worktree) |
| **Date** | 2026-08-08 |
| **Verdict** | **READY** |
| **Merge / PR open** | **Do not merge** this story PR in-review. PR open is implementer checklist. |

### Head honesty (fail-close)

| Commit | Kind | Covered by this review |
|--------|------|------------------------|
| `7e617b8` | plan doc | Scope / non-claims / matrix |
| `162ccb0` … `a549c35` | fixture + R0A harness + agent units | Product/test behavior under review |
| READY evidence + this review | docs package | Wire/META samples + gate claims |

Docs-only tips after this file that only point at the review do **not** require a second code review.

---

## 1. Scope checked (story must-hold)

| Must-hold | Result |
|-----------|--------|
| Path A public-entry R0A: explore spawn, implement-class mutate, worker cache stamp | **OK** — three scenarios green; wire + META + disk golden consistent |
| Public-path only claims (`deepseek-build`/`dsb` agent; no `DEEPSEEK_BUILD_AGENT_BIN` sole proof) | **OK** — harness unsets override; META records `DEEPSEEK_BUILD_AGENT_BIN_unset=yes` |
| Unversioned — no SemVer bump (stay **5.3.0**) | **OK** — workspace + npm package still `5.3.0` |
| Stacked on `vc010-l3-parallel-bg` / Depends on **#142** | **OK** — plan + branch baseRef `vc010-l3-parallel-bg` |
| V3-60-3 parent snippet expire on Path A | **Residual** — honest; thin unit only (see §3) |
| Gates: owner-bar, path-linkage, heart | **OK** — re-ran this lane (see §2.1) |

---

## 2. What was verified (this review)

### 2.1 Verification method

This independent lane verified by:

1. **Re-execution** of `./scripts/test-path-a-vc011-r0a.sh --skip-build` → all three scenarios **PASS**
2. **Re-execution** of `./scripts/check-path-a-linkage.sh` → **PASS**
3. **Re-execution** of `./scripts/test-heart-regression.sh` → **PASS** (AGENT_SUBAGENT PASS; live L3 SKIP)
4. **Re-execution** of `./scripts/test-owner-bar.sh` → **PASS** 60/60
5. **Re-execution** of `cargo test -p dsb-agent subagent` → **PASS** 5 tests
6. Static inspection of fixture, harness, plan, wire/META artifacts, SemVer files

```text
# Artifact head identity (from META files)
git_sha=a549c358ccaea7a4f7b4cb1545f3d97b76894f68
cli=…/target/release/deepseek-build
agent_resolved=…/third_party/grok-build/target/release/xai-grok-pager
DEEPSEEK_BUILD_AGENT_BIN_unset=yes
scenarios=explore-subagent | implement-subagent-mutate | worker-cache-stamp
agent_exit=0 (all three META)
path_a_l3_stamp=present; worker_epochs_match=true (all)

# SemVer
Cargo.toml [workspace.package] version = "5.3.0"
package.json version = "5.3.0"
```

### 2.2 File / evidence inspection

| Artifact | Inspection result |
|----------|-------------------|
| Plan/evidence `VC011_SUBAGENT_WORKER_CACHE_PATH_A_2026-08-08.md` | Call-path map, acceptance matrix, public-path table, explicit non-claims present; V3-60-3 residual called out |
| `scripts/lib/scripted_deepseek_server.py` | Scenarios `explore-subagent`, `implement-subagent-mutate`, `worker-cache-stamp`; child session detection via tokens; spawn + optional collect |
| `scripts/test-path-a-vc011-r0a.sh` | Public CLI; unsets `DEEPSEEK_BUILD_AGENT_BIN`; keeps subagent tools (only strips web); hard asserts wire + stamp + disk |
| `crates/dsb-agent/src/subagent.rs` | Units: explore deny-write, cache law epochs, implement mutate → parent expire, unknown kind, explore non-mutate |
| `PATH_A_R0_VC011_explore-subagent_{WIRE,META}` | META final contains `explore-subagent-ok` / child saw FINDME-77; wire spawn explore + child read |
| `PATH_A_R0_VC011_implement-subagent-mutate_{WIRE,META}` | META `worker_out='worker-mutated-ok\n'`; wire spawn general-purpose |
| `PATH_A_R0_VC011_worker-cache-stamp_{WIRE,META}` | Final `worker-cache-stamp-ok`; stamp epochs match |
| `PATH_A_R0_VC011_L3_last.txt` | `worker_epochs_match=true`, kinds explore/implement, subagents enabled |

### 2.3 Stack base correctness

- Branch forked from VC010 tip `9d6f1d0` (open PR **#142**).
- Plan §0 / §7 require PR base `vc010-l3-parallel-bg` and body **Depends on #142**.
- Diff surface: docs + hermetic scripts + agent unit tests — no SemVer packaging.

---

## 3. Claims honesty (public Path A vs thin Path B)

| Claim | Path A proof? | Notes |
|-------|---------------|-------|
| V3-60-1 explore + implement-class dogfood | **Yes** | Public spawn + child tool/disk |
| V3-60-2 worker reuses stable prefix (hash) | **Yes** (stamp on public launch) | Dual `worker_epoch_*` match via `stamp_path_a_l3` |
| V3-60-3 parent snippet expire after worker mutation | **Thin unit only** | **Residual** — not Path A sole green; correctly residualized in plan §6.4 |
| Live spawn without key | **No** | SKIP residual |

No greenwashing observed: V3-60-3 residual is explicit.

---

## 4. P0 / P1 findings

| Severity | Finding | Disposition |
|----------|---------|-------------|
| — | None blocking | — |
| **P2** | V3-60-3 Path A parent snippet table expire still residual | Accept as residual; follow-up optional after Path A wiring if product wants sole Path A proof |
| **P2** | Wire shows two spawn_emits (parent path retries / dual tool surface) | Acceptable; explore_typed and disk goldens still hold |

---

## 5. Verdict

**READY** to open the stacked unversioned PR on `vc010-l3-parallel-bg` with **Depends on #142**, labels, English body. **Do not merge** in this story. **Do not bump SemVer**.

---

## 6. Residuals for stack owner

- VC012 worktree dogfood
- VC013 **5.4.0** cut
- Live L3.5 when API key present
- Optional: Path A wiring for parent snippet expire after implement-class child mutation (closes V3-60-3 residual)
