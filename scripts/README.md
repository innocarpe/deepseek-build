# scripts/

| Script | Role |
|--------|------|
| `install.sh` | Install **`deepseek-build`** + **`dsb`** onto PATH (`~/.deepseek-build/bin` or Cargo bin) |
| `build-grok-pager.sh` | Build/check vendored Grok composition root (`deepseek-build-agent`), pinned to this checkout's own `third_party/grok-build/target` |
| `vendor-cargo.sh` | **The way to run vendored cargo.** Pins `CARGO_TARGET_DIR` to this worktree's `third_party/grok-build/target` (an inherited value is ignored and named), seeds a cold target once from the sibling worktree whose recorded sources differ least, caps jobs (`--jobs`, default 4), and refuses to start while another worktree's vendored build is in flight (`--allow-concurrent` goes through the memory gate as a second build at 2 jobs). Subcommands that compile nothing (`fmt`, `metadata`, …) skip seed and queue check; everything after the wrapper flags goes to cargo unchanged (`--target <triple>`, `-j`) |
| `test-vendor-cargo.sh` | Hermetic test for `vendor-cargo.sh` on a throwaway two-worktree repo with real cargo: the shared-target hazard itself (B's no-op build runs A's binary), the pin under an inherited `CARGO_TARGET_DIR`, cargo's own flags passing through, one seed then B's own code, the queue refusal, and the gate (offline, macOS) |
| `vendor-build.sh` | Read every worktree's vendored-Grok build queue — `status` reports file-lock holders/waiters, elapsed, command, worktree, host free/swap/load5m and the memory-gate verdict (exit 1 while busy); `--target DIR` reads one target. `run -- <cmd>` passes a free queue through and otherwise starts only through the memory gate with `CARGO_BUILD_JOBS=2`; it never picks a target directory. `prune` clears old `~/.cache/dsb-vendor-targets` copies from the retired shared-target escape hatch (base target untouched) |
| `test-vendor-build.sh` | Hermetic test for `vendor-build.sh` (fixture target trees + python3 file-lock holders, fake rustc children, a fixture repo with three worktrees; no cargo, no network, no `~/.cache` writes) |
| `lib/vendor-target.sh` | `vendor_target_pin <root>` — the pin the scripts that run cargo under `third_party/grok-build` call, so a `CARGO_TARGET_DIR` from the caller's environment cannot put two worktrees on one target (pinned by `test-vendor-target.sh`) |
| `test-vendor-target.sh` | Hermetic test for that helper (fixture checkout; exports, warning, silence when the inherited value already matches; no repo writes) |
| `dsbdev.sh` | Build the vendored pager for one worktree and open `dsb` on it (`DEEPSEEK_BUILD_AGENT_BIN`). Builds into that worktree's own `third_party/grok-build/target` — a shared target lets worktrees overwrite each other and hand one worktree another's code from a no-op build. The first debug build is seeded by APFS clone from the sibling worktree target whose recorded sources differ least, so only crates whose sources differ compile. Prints units compiled and seconds per build (`--no-run`, `--release`, `--jobs N`; logic in `lib/dsbdev.py`) |
| `test-dsbdev.sh` | Test for `dsbdev.sh` on a throwaway four-crate repo with real cargo and git worktrees: cold, no-op, seeded, an old-mtime edit, and a donor rebuilt after its manifest — each case checks the compiled unit set and what the binary prints (offline, macOS) |
| `install-dsb-exempt.sh` | macOS: install the DsbExempt runner (`/Applications/DsbExempt.app` + `~/.local/bin/dsb-exempt`, source `lib/dsb-exempt.c`). macOS assesses every freshly built executable and dylib on its first run (300–590 ms each, 10 s for the debug pager); Developer Tools exempts processes whose *responsible* app is listed there, and Orca tabs are responsible for themselves, so listing Orca does nothing. The runner is what gets listed, once, by hand; `vendor-cargo.sh` and `dsbdev.sh` then run cargo and `dsb` under it (`DSB_EXEMPT=0` turns it off). An installed bundle built from the same source is left alone, since a rebuild voids its Developer Tools entry |
| `test-dsb-exempt.sh` | Test for `install-dsb-exempt.sh` and the runner, installed into a temp dir: the child is attributed to the runner, a second install leaves the bundle alone, exit status / signal / stdin pass through (macOS only; the exemption itself needs the manual Developer Tools step) |
| `check-semver.sh` | Fail-close: workspace version must be full SemVer `MAJOR.MINOR.PATCH` |
| `bump-version.sh` | Bump the version across `Cargo.toml`, `package.json`, `Cargo.lock`, `CHANGELOG.md` and the versions log; **moves the `Unreleased` items into the new section** (`--dry-run` previews the move). The user-facing READMEs are version-free by policy and are not touched |
| `reorder-changelog.sh` | Reorder CHANGELOG to the invariant (Unreleased top, newest-first); `--check` for CI. Reorders only — it does not move items between sections |
| `test-changelog-release.sh` | Hermetic regression test for the `Unreleased` → version-section move (fixture CHANGELOGs; no network, no repo writes) |
| `lib/changelog_release.py` | The mover behind `bump-version.sh` (`plan` / `apply`), shared so the dry-run and the real bump report the same outcome |
| `next-version.sh` | The bump-level judgment (`docs/contributing/versioning.md` §1c): reads the first-parent merges on `origin/main` since the newest `v*` tag — the `<type>/` prefix of each merge branch and the paths it changed — and prints one line per merge plus the proposed version (`--level` / `--version`; `--check <ver>` is the gate `release.sh` runs). `gh` is never involved |
| `release.sh` | Release orchestrator (bump level → bump → PR → wait for the PR's checks → merge → tag → assets → CI publish → verify); refuses a version below the computed bump level unless `--level-override "<reason>"` records why |
| `lib/pr_checks.py` | The wait behind that merge: polls `gh pr view --json state,mergeable,statusCheckRollup` until every check the PR reports is complete with none failed and GitHub says `MERGEABLE`; a failed check, a conflict, a closed PR or the deadline (`--checks-timeout`, default 3600 s) stops before the merge call |
| `test-release-args.sh` | Hermetic regression for `release.sh` argument expansion: a release with no `--desc` must reach the bump under macOS `/bin/bash` 3.2 (empty-array `set -u` guard, the 6.1.7 stop), and a multi-word `--desc` stays one argv element |
| `test-release-pr-wait.sh` | Hermetic regression for that wait (a temp repo with a bare origin and a fake `gh` replaying rollup fixtures): empty → pending → green waits, merges and reaches the tag; a failing check, a conflict and a rejected `gh pr merge` stop before/short of the merge with the resume command printed; `--checks-timeout` bounds the wait |
| `test-next-version.sh` | Hermetic regression for the bump-level rule and the `release.sh` gate (throwaway repos with GitHub-shaped merge subjects, a fake `gh`): feat on the surface → MINOR, fix/perf → PATCH, harness only → none, a direct surface commit floors at PATCH; `--check` refuses below the judgment naming the deciding merge; `release.sh` stops before the bump and records the override reason in the release PR body; no tag → exit 2 and the release warns and carries on. Local-only by contract |
| `npm-emergency-publish.sh` | **Emergency** local npm publish; drives the interactive login + emailed code through `aside` (ADR 0012) |
| `verify-npm-version.sh` | Wait (bounded retry) until the registry serves a published version — the post-publish read shared by `release.sh`, `npm-emergency-publish.sh` and `publish-npm.yml` |
| `test-npm-verify-retry.sh` | Hermetic regression test for that retry (local mock registry; no network, no credentials) |
| `lib/mock_npm_registry.py` | Mock npm packument server used by the test above |
| `check-pr-title.sh` | **Optional** local Conventional Commits title check (not CI) |
| `adversarial-review.sh` | Run and track pinned, read-only `gpt-6-astra` and `claude-opus-5-5` xhigh reviews, including per-attempt status and evidence |
| `sync-labels.sh` | Push `.github/labels.json` to GitHub labels |
| `smoke-dogfood.sh` | Quick offline smoke (+ optional thin live if key set) |
| **`test-pre3x-baseline.sh`** | **Pre-3.0.0 orchestrator** (T0–T4) — see matrix doc |
| `test-product-offline.sh` | T0 + T2 offline product crates / dual bins / config seed |
| `test-grok-vendor-offline.sh` | T1 curated Grok vendor `cargo test` (not full workspace) |
| `test-deepseek-live.sh` | T3 thin + **T4 agent** live DeepSeek API feature probes |
| `lib/common.sh` | Shared helpers (key load, hermetic GROK_HOME, redaction) |
| **`test-owner-bar.sh`** | **Owner-bar-5x aggregator** — RED until all P0 PASS (`--selftest` for gate substrate) |
| `check-path-a-linkage.sh` | Fail-close dead wiring / missing mint / orphan `path_a_*` |
| `check-forbidden-evidence.sh` | Fail if active 5.x evidence uses sole Path B cargo claims |
| `check-worktree-ownership.sh` | Read-only live check: every worktree of this repo with dirty work must hold an agent tab (`orca terminal list` title/preview, evidence printed) — exit 1 on the empty-card defect, `--json`; pinned by `test-check-worktree-ownership.sh` |
| **`test-path-a-public-entry-e2e.sh`** | **Path A R0A** public CLI → agent_launch → scripted DeepSeek + wire |
| `lib/scripted_deepseek_server.py` | Hermetic Chat Completions fixture (SSE + wire JSONL) |
| `lib/owner-bar-common.sh` | Shared helpers for owner-bar gates |
| `theme-preview/metrics.py` | Colorimetry for theme candidates — WCAG 2.1 contrast, CIE L\*, APCA Lc, LCh↔sRGB (stdlib only) |
| `theme-preview/remap.py` | Recolor a screenshot's terminal region with a candidate palette (`--src`/`--old`/`--new`/`--out`/`--crop`); the method and its limits are in [`THEME_CLASSIC_READABILITY_2026-09-27.md`](../docs/product/THEME_CLASSIC_READABILITY_2026-09-27.md) |
| `theme-preview/remap2.py` | Blend-aware pass of the same: backgrounds that are a mix of two ramp colors, host chrome preserved |

## Owner-bar-5x (active product train → `5.0.0`)

Board: [`docs/product/OWNER_BAR_5X_GOALS.md`](../docs/product/OWNER_BAR_5X_GOALS.md).

```bash
./scripts/test-owner-bar.sh            # expect non-zero until late train
./scripts/test-owner-bar.sh --selftest # gate substrate
./scripts/test-path-a-public-entry-e2e.sh
```

Wire/meta: `docs/product/evidence/PATH_A_R0_*_last.*`

## Pre-3.0.0 baseline (required before heart fusion)

Normative matrix: [`docs/product/PRE_3X_TEST_MATRIX.md`](../docs/product/PRE_3X_TEST_MATRIX.md).

```bash
# Everyday (recommended): product offline + DeepSeek live agent smoke
./scripts/test-pre3x-baseline.sh --live

# Offline product only
./scripts/test-pre3x-baseline.sh

# Vendor offline (tiered — default light; avoid --vendor-full unless needed)
./scripts/test-pre3x-baseline.sh --vendor
./scripts/test-pre3x-baseline.sh --vendor-medium
./scripts/test-pre3x-baseline.sh --vendor-full   # HEAVY disk under third_party/grok-build/target

# After a heavy vendor run, free disk:
rm -rf third_party/grok-build/target
```

Results TSV: `docs/product/evidence/_last_pre3x_results.tsv`  
Durable report: `docs/product/evidence/PRE3X_BASELINE_YYYY-MM-DD.md`

## L3 smoke (4.0 prep, parallel-safe)

Does **not** change product defaults. Uses installed `deepseek-build-agent` +
DeepSeek API. Safe while **heart-3x** develops (separate worktree recommended).

```bash
./scripts/test-l3-smoke.sh              # CLI + headless + bg shell + worktree help
./scripts/test-l3-smoke.sh --extended   # + spawn_subagent (slower)
```

Results: `docs/product/evidence/_last_l3_smoke.tsv`  
Ops plan: [PARALLEL_3X_4X_PLAN.md](../docs/product/PARALLEL_3X_4X_PLAN.md)

## Install (product)

```bash
# From repo root
./scripts/install.sh              # → ~/.deepseek-build/bin
./scripts/install.sh --cargo      # → ~/.cargo/bin
./scripts/check-semver.sh
deepseek-build --version          # after PATH includes the bin dir
dsb --version
```

See root [README.md](../README.md) § Install.
