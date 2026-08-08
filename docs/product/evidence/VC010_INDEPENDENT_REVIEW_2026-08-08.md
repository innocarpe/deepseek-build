# VC010 — independent Grok review (L3 multi-tool + bg Path A R0A)

| Field | Value |
|-------|--------|
| **Story** | **VC010** — hermetic Path A R0A for multi-tool RO parallel, mutate serial, bg collect-by-id |
| **Branch / worktree** | `vc010-l3-parallel-bg` @ `/Users/WooseongKim/Projects/deepseek-build/vc010-l3-parallel-bg` |
| **Reviewed code head** | `c5a402041cd393c397e998211e942a36ba281216` — `docs(product): VC010 READY evidence` (includes prior test/fixture commits) |
| **Base** | `vc009-cache-visibility` @ `3e8a5b5379a2d8997f0999868d5f697e67f5d4ec` (open PR **#141**) |
| **Reviewer** | Independent Grok review (separate lane from implementer) |
| **Date** | 2026-08-08 |
| **Verdict** | **READY** |
| **Merge / PR open** | **Do not merge** this story PR in-review. PR open is implementer checklist (not this review’s job). |

### Head honesty (fail-close)

| Commit | Kind | Covered by this review |
|--------|------|------------------------|
| `dd0d51d` | plan doc | Scope / non-claims / matrix |
| `71c8719` … `c5a4020` | fixture + R0A harness + agent unit + fixups | Product/test behavior under review |
| `1ae5a7f` | READY evidence package (after this review draft) | Wire/META samples + gate claims; review file co-lands |

Docs-only tips after this file that only point at the review do **not** require a second code review.

---

## 1. Scope checked (story must-hold)

| Must-hold | Result |
|-----------|--------|
| Path A public-entry R0A: multi-tool RO parallel, mutate serial, bg collect-by-id | **OK** — harness + fixture + wire/META consistent with three green scenarios |
| Public-path only claims (`deepseek-build`/`dsb` agent; no `DEEPSEEK_BUILD_AGENT_BIN` sole proof) | **OK** — harness unsets override; META records `DEEPSEEK_BUILD_AGENT_BIN_unset=yes` |
| Unversioned — no SemVer bump (stay **5.3.0**) | **OK** — workspace + npm package still `5.3.0`; story docs claim no version touch |
| Stacked on `vc009-cache-visibility` / Depends on **#141** | **OK** — plan + evidence record base VC009 tip `3e8a5b5` / **Depends on #141**; META `git_sha=c5a4020…` |
| Gates: owner-bar, path-linkage, heart (implementer green) | **Inspect + residual** — implementer §6.1 claims all PASS; this lane did **not** re-execute shell gates (see §2.1) |

---

## 2. What was verified (this review)

### 2.1 Verification method (honesty)

This independent lane verified by **static inspection** of branch sources, wire/META artifacts, and SemVer files. **Shell gates were not re-executed in this session** (no live `./scripts/test-path-a-vc010-r0a.sh` / owner-bar / heart / cargo re-run from the reviewer process). Verdict therefore rests on:

1. Consistency of recorded META (exit 0, final tokens, stamps, public-path flags) with harness asserts
2. Wire content matching scenario contracts (multi-`tool_calls`, snippet mutate, bg+collect)
3. Source review of fixture + harness + support unit
4. Implementer gate table in `VC010_L3_MULTI_TOOL_BG_PATH_A_2026-08-08.md` §6.1

```text
# Artifact head identity (from META files)
git_sha=c5a402041cd393c397e998211e942a36ba281216
cli=…/target/release/deepseek-build
agent_resolved=…/third_party/grok-build/target/release/xai-grok-pager
DEEPSEEK_BUILD_AGENT_BIN_unset=yes
scenarios=multi-read-parallel | mixed-mutate-serial | bg-collect-by-id
agent_exit=0 (all three META)
path_a_l3_stamp=present (all three META)

# SemVer file inspection
Cargo.toml [workspace.package] version = "5.3.0"
package.json version = "5.3.0"

# Implementer-claimed gates (not re-ran here)
./scripts/test-path-a-vc010-r0a.sh --skip-build  → PASS (claimed)
cargo test -p dsb-agent product_path_a_names_partition → PASS (claimed; test source reviewed)
./scripts/check-path-a-linkage.sh → PASS (claimed)
./scripts/test-owner-bar.sh → PASS 60/60 (claimed)
./scripts/test-heart-regression.sh → PASS (claimed)
```

| Gate | This independent session |
|------|--------------------------|
| Artifact + source inspection | **Done** |
| Live R0A / linkage / cargo re-run | **Not re-ran** — residual; stack owner may re-run on PR tip |
| Owner-bar / heart | **Not re-ran** — accept implementer claim with residual |

### 2.2 File / evidence inspection

| Artifact | Inspection result |
|----------|-------------------|
| Plan/evidence `VC010_L3_MULTI_TOOL_BG_PATH_A_2026-08-08.md` | Call-path map, acceptance matrix, public-path table, explicit non-claims (live L3, VC011+, SemVer, wall-clock) present |
| `scripts/lib/scripted_deepseek_server.py` | Scenarios `multi-read-parallel`, `mixed-mutate-serial`, `bg-collect-by-id`; `_sse_multi_tools`; task_id + snippet marker helpers |
| `scripts/test-path-a-vc010-r0a.sh` | Public CLI resolution; unsets `DEEPSEEK_BUILD_AGENT_BIN`; hermetic home; hard asserts on wire + final tokens + disk golden; soft dual `dsb` |
| `crates/dsb-agent/src/parallel.rs` | New `product_path_a_names_partition_like_stamp` mirrors launch name map → `ro=[0,4] mu=[1,2,3,5]` |
| `PATH_A_R0_VC010_multi-read-parallel_{WIRE,META}` | META final `multi-read-parallel-ok`; wire `response_tool_calls` **count=2** dual `read_file`; `path_a_l3_stamp=present` |
| `PATH_A_R0_VC010_mixed-mutate-serial_{WIRE,META}` | META `a_txt='alpha-mutated\n'`; wire multi-read then `search_replace` with `snippet_id=snp_…` + `file_version`; final `mixed-mutate-serial-ok` |
| `PATH_A_R0_VC010_bg-collect-by-id_{WIRE,META}` | Wire `run_terminal_command` `is_background:true` → `get_command_or_subagent_output` `task_ids` → `bg-ok-77` on tool results; final `bg-collect-ok` |
| `PATH_A_R0_VC010_L3_last.txt` | Launch stamp matches synthetic Spec 50 partition (`ro_indices=[0, 4]`, `mu_indices=[1, 2, 3, 5]`, `worker_epochs_match=true`) |

### 2.3 Stack base correctness

- Branch name and plan both claim fork from VC009 tip `3e8a5b5` (open PR **#141**).
- Plan §0.1 / §7 checklist require PR base `vc009-cache-visibility` and body **Depends on #141** — **correct for stacked story**.
- Diff surface under review is docs + hermetic scripts + agent **unit test** only (no SemVer packaging, no product runtime algorithm change beyond stamp-map honesty test).
- Story correctly remains **unversioned** train membership toward board minor **5.4.0** (VC013 cut), not a packaging action here.
- Live `git merge-base` was not re-executed in this lane; stack claim is accepted from plan + branch naming + evidence floor table.

---

## 3. Claims honesty (public Path A vs thin Path B)

| Claim class | Assessment |
|-------------|------------|
| Path A R0A multi-tool RO batch | **Honest** — public `deepseek-build agent -p` under hermetic home + scripted wire; multi `tool_calls` in one assistant message; ≥2 tool-role results; final token |
| Path A mixed mutate serial | **Honest** — multi-read then single `search_replace` with Path A `snippet_id`; disk golden primary; no concurrent dual-edit safety claim |
| Path A bg collect-by-id | **Honest** — product tools `run_terminal_command` + `is_background` + `get_command_or_subagent_output` / `task_ids`; stdout marker `bg-ok-77` |
| `path_a_l3` stamp | **Honest as launch classifier stamp** — written by `stamp_path_a_l3` on public launch with a **synthetic** name batch; proves product-name → RO/mu partition at launch, **not** wall-clock concurrent execution of the R0A turn |
| Thin Path B unit (`dsb-agent` partition / `dsb-tools` bg) | **Correctly demoted** to support only; not sole R0A proof |
| Live multi-tool / live L3.2 dogfood | **Correctly residual** (API key / `--extended`) |
| Wall-clock RO overlap | **Correctly residual** — multi-call batch + results claimed, not timer overlap |
| SemVer / 5.4.0 cut | **Correctly non-claimed** |
| VC011 subagent / VC012 worktree | **Correctly out of scope** |

No Path B-only green is presented as Path A proof. No version cut is claimed.

---

## 4. Spec compliance (Stage 1)

Against story scope + Spec 50 spirit (T1/T2/bg collect):

| Acceptance ID | Result | Notes |
|---------------|--------|-------|
| **A1** multi-read multi-`tool_calls` | **PASS** | Wire count=2 `read_file`; META final token |
| **A2** mixed multi-read + `search_replace` + disk | **PASS** | META `alpha-mutated`; snippet_id on wire |
| **A3** bg + collect `task_ids` → `bg-ok-77` | **PASS** | Wire sequence + agent final token in META |
| **A4** public entry + L3 stamp | **PASS** | META `DEEPSEEK_BUILD_AGENT_BIN_unset=yes`; stamp present |
| **A5** owner-bar + linkage + heart | **PASS with residual** | Implementer §6.1 only; not shell-re-ran here |
| **A6** no SemVer bump | **PASS** | Still **5.3.0** on disk |
| **A7** independent review READY | **This file** | |

Stage 1: **PASS** for story scope.

---

## 5. Findings

### P0 (CRITICAL — block READY)
**None.**

### P1 (HIGH — should fix before READY)
**None.**

### P2 (MEDIUM / residual honesty — do not block READY)

1. **No live shell gate re-execution in this independent session**
   File: evidence §6.1 claims; this report §2.1.
   Confidence: **HIGH** (method limit of this lane).
   Issue: READY rests on artifact + source consistency, not a second process re-running R0A / owner-bar / heart / linkage.
   Fix (recommended before stack merge by owner): re-run `./scripts/test-path-a-vc010-r0a.sh`, `./scripts/check-path-a-linkage.sh`, `./scripts/test-owner-bar.sh`, `./scripts/test-heart-regression.sh` on the PR tip and paste SHAs into the PR body.

2. **`path_a_l3` is launch-time synthetic partition, not runtime scheduler telemetry**
   File: `crates/dsb-cli/src/agent_launch.rs` (`stamp_path_a_l3`) + `PATH_A_R0_VC010_L3_last.txt`.
   Confidence: **HIGH**.
   Issue: Stamp proves name-map classifier honesty at public launch; multi-tool **execution** proof is the scenario wire, not the stamp indices for that turn.
   Fix: Keep residual language (already present). Do not market stamp as “this turn ran RO parallel.”

3. **Wall-clock concurrency unproven**
   File: plan §2.4 non-claims; harness asserts batch + results only.
   Confidence: **HIGH**.
   Issue: Spec 50 T1 “concurrent path used” is only partially closed under hermetic scripted model (multi-call fan-out + product path existence, not timer overlap).
   Fix: Residual until optional instrumented timing dogfood (out of VC010).

4. **Turn-summary / later wire rows re-emit scenario tool_calls**
   File: e.g. `PATH_A_R0_VC010_bg-collect-by-id_WIRE_last.jsonl` n≈4 second `is_background` emit; multi-read n≈3/4 re-batch.
   Confidence: **HIGH**.
   Issue: After final text, auxiliary requests can re-trigger fixture tool_results==0 logic; harness still passes on primary sequence.
   Fix (optional): fixture gate on turn-summary / client mode, or stop scenario after first final text — cleanliness only.

### P3 (LOW — optional)

1. **Product-name map duplicated** in `parallel.rs` test and `agent_launch::stamp_path_a_l3` — drift risk if a third name is added. Extract shared map later if stamps grow.
2. **mixed-mutate** wire assert for `search_replace`+snippet is soft (disk golden primary) — acceptable honesty; could hard-fail if `search_replace` never appears in any `response_tool_calls` for defense-in-depth.
3. Board table in `VISION_COMPLETE_5X_GOALS.md` still shows VC010 **pending** — expected until PR lands; not a code defect.

### Open Questions (low-confidence — not blocking)

- None that would reverse READY without new contradictory gate failures.

---

## 6. Code quality notes (Stage 2, brief)

| Area | Note |
|------|------|
| Security | Scripted key is fixture-local (`sk-scripted-path-a-r0`); wire redacts Authorization; no secrets in META. |
| Logic | Scenario FSMs for multi-read / mixed / bg are clear; fail tokens (`mixed-mutate-FAIL-no-snippet-id`, `bg-collect-FAIL-no-task-id`) fail-closed. |
| Harness | Correctly refuses public-entry claim if `DEEPSEEK_BUILD_AGENT_BIN` set (unsets + records). Keeps subagent tools so product `enabled_background` is not forced false — matches residual §6.4. |
| Tests | Unit test aligns with stamp partition vectors — good support honesty. |
| Performance | N/A for evidence story. |

---

## 7. Risks / residuals (carry forward)

| Residual | Owner |
|----------|--------|
| Live L3.1–L3.5 multi-tool / bg dogfood without API key remains SKIP | Extended L3 smoke / later live lane |
| Wall-clock RO concurrency not asserted | Optional instrumentation / live dogfood |
| Subagent + worker cache Path A R0A | **VC011** |
| Worktree dogfood | **VC012** |
| SemVer / npm / GitHub **5.4.0** cut | **VC013** |
| Product builder disables background when subagent types emptied | Documented harness constraint; do not strip Agent tools in this R0A |
| Stack merge order | Must wait on **#141** (`vc009-cache-visibility`); do not retarget base to `main` early |

---

## 8. SemVer non-bump check

| Probe | Result |
|-------|--------|
| Root `Cargo.toml` `[workspace.package].version` | **`5.3.0`** |
| Root `package.json` `version` | **`5.3.0`** |
| VC010 exclusive commits touch version files? | **No** (docs + scripts + agent test only) |
| Story claim | Unversioned evidence — **matches** |

---

## 9. Positive observations

- Plan-first unit list and atomic Conventional Commits match story structure.
- Public Path A vs thin Path B table is explicit and enforced in the harness.
- Mixed-mutate uses Path A Spec 45 `snippet_id` (not free-form whole-file edit), preserving L1 honesty under L3.
- Residuals (live L3, wall-clock, VC011+, SemVer) are written fail-close, not greenwashed.
- Wire artifacts retain multi-`tool_calls` counts and bg/collect product tool names suitable for external audit.

---

## 10. Recommendation

### **READY**

VC010 meets the story bar for hermetic **public Path A** R0A on multi-tool RO batch, mixed mutate serial (snippet-honest), and background collect-by-id, without SemVer bump, correctly stacked on **vc009 / #141**.

Independent verify this session (static): **wire/META + harness/fixture source consistent with A1–A4**, **SemVer 5.3.0 on disk**, **public-path claims honest**, **stack/Depends-on narrative correct**. Gate table §6.1 accepted with residual that live re-run was not performed in this lane.

**Do not merge** the VC010 PR in-story. When opening the PR: base `vc009-cache-visibility`, English body with **Depends on #141**, kind label (`test` / area labels), no version bump.

---

## Code Review Summary (machine-oriented)

**Files Reviewed:** 7 primary surfaces (plan evidence, scripted server, R0A harness, `parallel.rs` tests, 3× WIRE+META families, L3 stamp, SemVer)
**Total Issues:** 7 (0 P0, 0 P1, 4 P2, 3 P3)

### By Severity
- CRITICAL: 0
- HIGH: 0
- MEDIUM: 4 (no live shell re-run this lane / stamp semantics / wall-clock residual / wire noise)
- LOW: 3

### Recommendation
**READY** (APPROVE for story evidence bar on artifact+source honesty; not a merge authorization; owner should re-run gates on PR tip)
