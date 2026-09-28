# Implementation gates ledger

**Purpose:** Make G0–G6 **auditable facts**, not self-attestation.  
**Normative definitions:** [architecture/HARNESS_PHILOSOPHY.md](architecture/HARNESS_PHILOSOPHY.md) §11.  
**SSOT priority:** [product/SSOT.md](product/SSOT.md).

| Gate | Requirement | Status | Evidence (PR / path) | Flipped by |
|------|-------------|--------|----------------------|------------|
| **G0** | HARNESS_PHILOSOPHY + layered SOURCES merged | **green** | PR #4 | innocarpe |
| **G1** | Toolchain/config ADR | **green** | `docs/adr/0004-toolchain.md` | innocarpe |
| **G1b** | DeepSeek provider contract ADR (pinned ids) | **green** | `docs/adr/0005-deepseek-provider-contract.md` | innocarpe |
| **G2** | Specs **10, 15, 20, 30** ready-for-impl | **green** | specs 10/15/20/30 | innocarpe |
| **G3** | Specs **45** + **90 minimum** ready | **green** | specs 45/90 | innocarpe |
| **G4** | Spec **50** ready | **green** | `docs/specs/50-parallelism-background.md` + agent parallel readonly (**0.12.0**) | innocarpe |
| **G5** | Spec **60** ready | **green** | `docs/specs/60-subagents.md` + in-process workers (**0.14.0**) | innocarpe |
| **G6a** | Spec **100** sessions ready | **green** | `docs/specs/100-sessions.md` + runtime **0.5.0** (#24) | innocarpe |
| **G6b** | Spec **70** skills ready | **green** | `docs/specs/70-skills.md` + runtime **0.6.0** (#25) | innocarpe |
| **G6c** | Spec **80** MCP ready | **green** | `docs/specs/80-mcp.md` + `dsb-tools` mcp catalog/fingerprint (**0.11.0**) | innocarpe |
| **G6d** | Spec **110** plan light ready | **green** | `docs/specs/110-plan-mode.md` + `plan` tool (**0.11.0**) | innocarpe |

**Legacy label G6:** means “all of G6a–G6d green.” Partial progress is tracked per subgate.

## Rules

1. **No runtime feature PR** may claim a gate is green without updating this table in the same PR (or a prior merged PR).  
2. **ready-for-impl** for specs **10, 15, 45, 50, 90, 100, 70** (and others marked automated) requires **automated** golden/negative tests in the test plan.  
3. Adding package code is allowed after G1 green.  
4. **Process-police CI** stays forbidden. Product CI (build/test/smoke) is allowed and encouraged.  
5. Who may flip a gate: maintainer on merge of the evidence PR; record login + PR number.  
6. **Sessions runtime requires G6a**; **skills runtime requires G6b**; **MCP requires G6c**; **plan product requires G6d**. Spec-only PRs may land while red; runtime must flip the subgate.

## Current product implication

- **Wave A dogfood** through **`0.7.0` npm package** shipped on `main` (install + tools + sessions + surface min + npm wrappers).  
- **Registry `npm publish`** is **CI-published** over OIDC trusted publishing ([ADR 0012](adr/0012-npm-trusted-publishing.md), amends [ADR 0007](adr/0007-npm-packaging.md)). The trusted publisher is **enrolled** (2026-09-25, `innocarpe/deepseek-build` + `publish-npm.yml`), but **no OIDC publish has run yet** — the next release is its first exercise.
- Spec **40** is **ready-for-impl** (`docs/specs/40-core-tools-surface.md`); it is **not** a G-number gate (G3 remains 45+90).  
- **2.x shell** shipped. **3.x / 4.x tags exist** as heart/L3 *attempts* — **owner-bar NOT MET** (Path A fusion incomplete).  
- Spec-ready (this table) ≠ Path A enforced. Historical heart evidence is archive only: [HEART_3X_SPEC_BINDING.md](architecture/HEART_3X_SPEC_BINDING.md) · [WAVE_3x_PR_DAG.md](product/WAVE_3x_PR_DAG.md).  
- **Owner-bar complete product** is **`5.0.0`** only when [OWNER_BAR_P0_LEDGER.md](product/OWNER_BAR_P0_LEDGER.md) all PASS on Path A ([OWNER_BAR_ACCEPTANCE.md](product/OWNER_BAR_ACCEPTANCE.md)).

**Ultragoal (product):** **`owner-bar-5x`** → tag **`v5.0.0`** ([ULTRAGOAL_CHAIN.md](product/ULTRAGOAL_CHAIN.md) · [OWNER_BAR_5X_GOALS.md](product/OWNER_BAR_5X_GOALS.md)).  
Do **not** resume `heart-3x` / `fleet-4x` as product SSOT. Gate: `./scripts/test-owner-bar.sh` (RED until fusion).

## Verification — jump-to-bottom chip (2026-09-28)

The chip now owns its complete cell state and one padding column on each side;
a wide scrollback glyph crossing its left boundary is erased before buffer
diffing. The label ladder, last scrollback row, one-row height, hover colour,
and hidden-state rules are unchanged. Full-view tests exercise 55x41 scrolling
and padding clicks at the full, short, and arrow label widths. Primitive tests
exercise inherited colours/modifiers/skip and wide-glyph diff output.

`./scripts/test-owner-bar.sh` printed
`PASS=60 FAIL=0 NOT_RUN=0 linkage_exit=0 forbidden_exit=0` and
`ALL PASS — owner bar green`; `./scripts/check-path-a-linkage.sh` printed
`PASS`. [PR #333 CI](https://github.com/innocarpe/deepseek-build/actions/runs/36401653117)
on source commit `2ff1207` passed `required`, `grok clippy`, and `grok fmt`.
The owner-bar run's 60 SHA-only changes in `OWNER_BAR_STATUS.tsv` were excluded;
the gate result above remains the actual execution result.

The existing [vendored workspace test run](https://github.com/innocarpe/deepseek-build/actions/runs/36404759770/job/108870717710)
compiled and executed the actual pager on `2ff1207`: `10295 passed; 0 failed;
5 ignored`. All 17 `follow_indicator_tests` / `jump_to_bottom_tests` passed,
including both new primitive tests and both new full-frame scrolling/padding
click tests. The **overall workspace run failed** (exit 101): its only failed
target was the unchanged shell library (`7113 passed; 1 failed; 5 ignored`).
`set_consent_answer_is_monotonic_per_account` reported that its config
destination changed from `config.toml` to `dotfiles/config.toml` during
`replay after the ack`. Shell/config sources are absent from this PR's diff;
that separate failure is retained as a validation limitation, not described
as a green workspace run. The successful pager results are reused without
rerunning the workspace or local pager filters.

An isolated Ratatui 0.29.0 harness copied the exact original/fixed helper,
label ladder, Theme definitions, and two new primitive tests without swapping
repository sources. The original produced `0 passed; 2 failed` (exit 101):
inherited modifiers and a wide glyph suppressing the first chip cell. The
fixed helper produced `2 passed; 0 failed` (exit 0). Its HQ wrapper printed
`acquired` / `released rc=0`; cargo used `CARGO_BUILD_JOBS=1`,
`vendor-cargo --jobs 1 --no-seed`, this worktree's `target/chip-source-repro`,
and `--test-threads=1`. The negative run's temporary controller initially
returned 1 because `--nocapture` separated test names from `FAILED`; the stored
failure list and panic messages establish both intended failures, so it was
not rerun.

Local full pager compilation was **not completed**: seeded registry artifacts
failed with `E0463`/`E0460`, and the coherent rebuild was interrupted (exit 130)
to adopt the machine-wide HQ slot. Only this worktree's Rust metadata/libraries
were invalidated; native build outputs remained cached. Remote pager execution
above supersedes that unfinished local gate. No build-wrapper source changed,
no physical iPhone smoke was performed, and this verification does not flip a
spec-readiness gate.

## Verification — prompt quota removal (2026-09-28)

The prompt quota removal rechecked the existing owner bar on the current Path A:
`./scripts/test-owner-bar.sh` printed `PASS=60 FAIL=0 NOT_RUN=0`,
`linkage_exit=0 forbidden_exit=0`, and `ALL PASS — owner bar green`.
`./scripts/check-path-a-linkage.sh` also printed `PASS`. This is a regression
verification; it does not flip a spec-readiness gate in the table above.

All five quota regression tests passed in the full pager library run. That run
printed `10285 passed; 2 failed; 5 ignored`; the two failures were the existing
dashboard multiline footer assertions expecting `Shift+Enter` or `Alt+Enter`
while macOS under `TERM_PROGRAM=Orca` renders `Opt+Enter`. Those dashboard files
are outside this correction and remain unchanged.
Repeating the same suite with the process-local terminal fixture
`TERM_PROGRAM=ghostty ./scripts/vendor-cargo.sh test -p xai-grok-pager --lib`
printed `10287 passed; 0 failed; 5 ignored`.

## Verification — host-observed test change evidence

Spec 130 defines bounded Path A reporting from successful in-process
`FileWritten` receipts. The first receipt supplies each path's baseline; a
receipt-chain mismatch or final-content mismatch omits that path. The report
counts modified, deleted, added, and renamed criteria by relative file scope
without exposing test names, source, literals, hashes, or absolute paths.
Supported conventions are Rust `#[test]` / `#[tokio::test]` (with preceding
attributes fingerprinted), Go `Test*` with `*testing.T`, and default pytest
module functions and `Test*` class methods (including decorated definitions;
decorator tokens are fingerprinted).
Formatting/comment-only writes, a same-name/same-signature file move, net-zero
write/revert, failed writes without a receipt, non-Git projects, unsupported
conventions, parse failures, and oversized inputs produce no attribution.
The host note and turn delta do not establish test execution, passing tests, or
semantic strengthening/weakening.

Coverage is in `session/test_criteria.rs` (baseline, dirty pre-existing file,
successful modified/deleted/added writes, source/literal-free output, net-zero
revert, test-file move, failed write, chain/final-content mismatch, non-Git,
language conventions, pytest decorator and Rust attribute changes, parser/size
bounds, and report truncation) plus
`notification/handle_tests.rs` (synchronous observer and tee propagation) and
`session/signals_tests.rs` (turn-delta serialization),
`session/acp_session_tests/turn_completion_emit_tests.rs` (actual Path A
completion/cancellation settlement handlers, result snapshot and turn-delta
evidence), and `agent/mvp_agent/turn_end.rs` (`TurnResultArgs` to durable
`turn_result.json` JSON mapping). The settlement fixture keeps the actor event
receiver open and undrained: it can accept `FlushReplay`, but no test sink can
acknowledge it. It exercises the shipped common completion/cancel handlers with
host-backed receipts, without a sampler or the run-loop select branch.

Local validation on 2026-09-29, through the worktree-pinned vendor wrapper:

- `./scripts/vendor-cargo.sh fmt --all` — passed.
- `./scripts/vendor-cargo.sh --jobs 4 check -p xai-grok-shell --all-targets` — passed.
- `./scripts/vendor-cargo.sh clippy -p xai-grok-shell --all-targets -- -D warnings` and `./scripts/vendor-cargo.sh clippy -p xai-grok-tools --all-targets -- -D warnings` — passed.
- `./scripts/vendor-cargo.sh test -p xai-grok-shell --lib test_criteria -- --nocapture` — 13 passed.
- `./scripts/vendor-cargo.sh test -p xai-grok-shell --lib path_a_completed_and_cancelled_turns_carry_test_change_evidence -- --nocapture` — 1 passed; completion and cancellation each finished within the 4-second bound, and each host note appeared once in gateway and persistence before its durable terminal.
- `./scripts/vendor-cargo.sh test -p xai-grok-shell --lib successful_test_write_receipt_is_serialized_as_source_free_turn_evidence -- --nocapture` — 1 passed.
- `./scripts/vendor-cargo.sh test -p xai-grok-shell --lib turn_result_metadata_keeps_host_test_evidence_from_the_terminal_snapshot -- --nocapture` — 1 passed.
- `./scripts/vendor-cargo.sh test -p xai-grok-tools --lib file_written_observer_runs_only_for_successful_write_notifications -- --nocapture` and `... tee_preserves_file_written_observers ...` — 1 passed each.
- With `PATH` resolving both `cargo` and `rustc`, the default-stack `./scripts/vendor-cargo.sh test -p xai-grok-shell --lib` run aborted with stack overflow at `agent::mvp_agent::tests::a_cancelled_installer_withdraws_its_unstamped_actor`; the same test also failed in isolation at the default stack. The repository CI job in `.github/workflows/ci-grok-test.yml` sets `RUST_MIN_STACK=16777216` because SessionActor turn futures can exceed Rust's 2 MiB default. Re-running `RUST_MIN_STACK=16777216 ./scripts/vendor-cargo.sh test -p xai-grok-shell --lib a_cancelled_installer_withdraws_its_unstamped_actor -- --nocapture` passed.
- The full `RUST_MIN_STACK=16777216 ./scripts/vendor-cargo.sh test -p xai-grok-shell --lib` run reported 7,110 passed, 8 failed, and 5 ignored. The failures were `agent::mvp_agent::tests::exhausted_fetch_decides_on_the_local_layers`, `claude_import::tests::gate_load_claude_env_returns_empty_when_marker_set`, `session::workflow::registry::tests::save_through_symlinked_session_root_stays_in_canonical_project`, `session::worktree::tests::create_worktree_for_resume_honors_git_ref`, `session::worktree::tests::create_worktree_for_resume_produces_independent_worktree`, `util::config::consent::tests::set_consent_answer_is_monotonic_per_account`, `util::config::mcp::tests::delete_mcp_server_config_at_follows_user_symlink`, and `util::config::persist::tests::no_home_cwd_config_resolves_slot_not_follow`. This local run is not claimed as a full-suite pass.
- Fresh-process reruns through `scripts/vendor-cargo.sh` with `RUST_MIN_STACK=16777216` passed for `agent::mvp_agent::tests::exhausted_fetch_decides_on_the_local_layers`, both `session::worktree::tests::create_worktree_for_resume_*` tests, `util::config::consent::tests::set_consent_answer_is_monotonic_per_account`, and `util::config::mcp::tests::delete_mcp_server_config_at_follows_user_symlink`. These five did not reproduce individually. That is an observation consistent with an interaction in the combined run, not proof of a parallel-only cause.
- `claude_import::tests::gate_load_claude_env_returns_empty_when_marker_set` failed in its fresh process under the ordinary local environment. The shell test seeds its own marker cache, while the workspace reader has a separate marker gate; the test-only `_GROK_CLAUDE_MARKER_OVERRIDE=1` makes that same test pass. The relevant shell/workspace source and test blobs are identical to base `dd0c8b938211e1c5d587c0dbaafff8baa20fe62d`; no test or production code was changed here.
- `session::workflow::registry::tests::save_through_symlinked_session_root_stays_in_canonical_project` and `util::config::persist::tests::no_home_cwd_config_resolves_slot_not_follow` failed with `/var/...` versus `/private/var/...` path spellings. With only `TMPDIR` changed to the canonical realpath of the existing temp directory, both passed in the same already-built test binary. Their test/source blobs also match base `dd0c8b938211e1c5d587c0dbaafff8baa20fe62d`; the observed failure is the temp-path alias affecting those assertions.
- `.github/workflows/ci.yml` runs the vendored PR checks `grok fmt` and strict `grok clippy`; it does not run the vendored shell test suite. `.github/workflows/ci-grok-test.yml` runs the full vendored suite on a `main` push after merge (or by manual dispatch), with `RUST_MIN_STACK=16777216`; it is not a PR check. Record the PR checks and the post-merge full-suite result separately.
