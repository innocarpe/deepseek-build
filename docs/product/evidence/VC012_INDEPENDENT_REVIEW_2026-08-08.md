# VC012 — independent Grok review (worktree dogfood Path A)

| Field | Value |
|-------|--------|
| **Story** | **VC012** — public Path A worktree dogfood + bare `dsb` / `deepseek-build` honesty (vision **V3-WT**) |
| **Branch / worktree** | `vc012-worktree-dogfood` @ `/Users/WooseongKim/Projects/deepseek-build/vc012-worktree-dogfood` |
| **Reviewed code head** | Implementation tip prior to this review package (CLI forward + R0A harness + docs; READY docs co-land) |
| **Base** | `vc011-subagent-worker-cache` @ `fd9b215` (open PR **#143**) |
| **Reviewer** | Independent Grok review lane (static + re-ran R0A/gates on this worktree) |
| **Date** | 2026-08-08 |
| **Verdict** | **READY** |
| **Merge / PR open** | **Do not merge** this story PR in-review. PR open is implementer checklist. |

### Head honesty (fail-close)

| Commit kind | Covered by this review |
|-------------|------------------------|
| Plan doc | Scope / non-claims / matrix |
| CLI `--worktree` forward + unit tests | Product surface under review |
| R0A harness + META/WIRE artifacts | Behavior under review |
| User-guide 13 + KNOWN_LIMITS | Docs honesty under review |
| READY evidence + this review | Gate claims |

Docs-only tips after this file that only point at the review do **not** require a second code review.

---

## 1. Scope checked (story must-hold)

| Must-hold | Result |
|-----------|--------|
| Path A public-entry R0A: worktree CLI surface, opt-in stamp, headless no-create | **OK** — three scenarios green |
| Public-path only claims (`deepseek-build`/`dsb`; no `DEEPSEEK_BUILD_AGENT_BIN` sole proof) | **OK** — harness unsets override; META records unset |
| Bare session honesty: opt-in, not mandatory implement worktree | **OK** — stamp + docs + Spec 60 non-goal restated |
| Unversioned — no SemVer bump (stay **5.3.0**) | **OK** |
| Stacked on `vc011-subagent-worker-cache` / Depends on **#143** | **OK** — plan + branch base |
| Gates: owner-bar, path-linkage, heart | **OK** — re-ran this lane |

---

## 2. What was verified (this review)

### 2.1 Verification method

1. **Re-execution** of `./scripts/test-path-a-vc012-r0a.sh --skip-build` → all three scenarios **PASS**
2. **Re-execution** of `./scripts/check-path-a-linkage.sh` → **PASS**
3. **Re-execution** of `./scripts/test-heart-regression.sh` → **PASS**
4. **Re-execution** of `./scripts/test-owner-bar.sh` → **PASS** 60/60
5. **Re-execution** of `cargo test -p dsb-cli tui_forward_flags_worktree reject_worktree_flags_on_line_mode stamp_path_a_l3` → **PASS**
6. Static inspection of plan, harness, CLI forward, docs, META/WIRE

```text
# SemVer
Cargo.toml [workspace.package] version = "5.3.0"
package.json version = "5.3.0"

# Stamp sample (PATH_A_R0_VC012_L3_last.txt)
worktree_product=opt_in
bare_dsb_session=single
worker_epochs_match=true
```

### 2.2 File / evidence inspection

| Artifact | Inspection result |
|----------|-------------------|
| Plan/evidence `VC012_WORKTREE_DOGFOOD_PATH_A_2026-08-08.md` | Call-path map, acceptance matrix, public-path table, non-claims present |
| `crates/dsb-cli/src/main.rs` | `--worktree`/`-w`/`--worktree-ref` on Cli; `tui_forward_flags`; reject on line-mode; unit tests |
| `scripts/test-path-a-vc012-r0a.sh` | Public CLI; dual dsb path; three scenarios; headless git worktree count assert |
| User-guide 13 | Bare single-session; public entry examples; headless honesty; R0A pointer |
| KNOWN_LIMITS | Residual row updated for VC012 / Spec 60 non-goal |
| META/WIRE | CLI surface + stamp + headless no-create consistent |

### 2.3 Stack base correctness

- Branch `vc012-worktree-dogfood` forked at VC011 tip `fd9b215`.
- Open PR base must be **`vc011-subagent-worker-cache`** with **Depends on #143**.
- Do **not** target `origin/main` until #143 merges.

---

## 3. Residuals (honest)

| Residual | Notes |
|----------|-------|
| Interactive TTY worktree **create** sole green | Not claimed; flag forward + docs + headless honesty are the R0A bar |
| `spawn_subagent` `isolation=worktree` live create | Optional; Spec 60 forbids mandatory implement worktree |
| **5.4.0** cut | **VC013** |
| V3-60-3 parent snippet expire Path A | Remains VC011 residual |

---

## 4. Verdict

**READY** to open the stacked unversioned PR. **Do not merge** in this story. **Do not bump SemVer.**
