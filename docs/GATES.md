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
- **Registry `npm publish`** is **CI-published** over OIDC trusted publishing ([ADR 0012](adr/0012-npm-trusted-publishing.md), amends [ADR 0007](adr/0007-npm-packaging.md)). The trusted publisher was enrolled on 2026-09-25 for `innocarpe/deepseek-build` and `publish-npm.yml`. Its first successful exercise was [publish-npm run 36422527144](https://github.com/innocarpe/deepseek-build/actions/runs/36422527144) on 2026-09-28 for release ref `v6.8.3` at head `4432ba562f98365af89cbbda137381e595f92395`; the workflow and `publish` job succeeded, including the OIDC publish, package verification, and global install smoke.
- Spec **40** is **ready-for-impl** (`docs/specs/40-core-tools-surface.md`); it is **not** a G-number gate (G3 remains 45+90).  
- **2.x shell** shipped. **3.x / 4.x tags exist** as heart/L3 *attempts* — **owner-bar NOT MET** (Path A fusion incomplete).  
- Spec-ready (this table) ≠ Path A enforced. Historical heart evidence is archive only: [HEART_3X_SPEC_BINDING.md](architecture/HEART_3X_SPEC_BINDING.md) · [WAVE_3x_PR_DAG.md](product/WAVE_3x_PR_DAG.md).  
- **Owner-bar complete product** is **`5.0.0`** only when [OWNER_BAR_P0_LEDGER.md](product/OWNER_BAR_P0_LEDGER.md) all PASS on Path A ([OWNER_BAR_ACCEPTANCE.md](product/OWNER_BAR_ACCEPTANCE.md)).

**Ultragoal (product):** **`owner-bar-5x`** → tag **`v5.0.0`** ([ULTRAGOAL_CHAIN.md](product/ULTRAGOAL_CHAIN.md) · [OWNER_BAR_5X_GOALS.md](product/OWNER_BAR_5X_GOALS.md)).  
Do **not** resume `heart-3x` / `fleet-4x` as product SSOT. Gate: `./scripts/test-owner-bar.sh` (RED until fusion).

## Verification — welcome phone top-bar margins (2026-09-29)

The welcome location row now uses the conversation's
`views::agent::effective_narrow(cols, rows)` predicate: no blank row above it
and a one-column gutter at each side on a phone. The screenshot's location
row is a separate header above the logo and centered menu; this change keeps
the menu's side spacing and the composer/footer margin rules independent.

Buffer evidence with regular mode, a pinned theme, and the actual welcome
renderer (coordinates are zero-based):

| Frame | Location start before | Location start after | Complete desktop text buffer |
|---|---|---|---|
| 55x41 | (2, 1) | (1, 0) | Phone |
| 73x53 | (2, 1) | (1, 0) | Phone at smaller text |
| 110x82 | (2, 1) | (1, 0) | Phone at smaller text |
| 60x30 | (2, 1) | (1, 0) | Width boundary |
| 61x30 | (2, 1) | (2, 1) | Identical before/after |
| 80x24 | (2, 1) | (2, 1) | Identical before/after |
| 120x40 | (2, 1) | (2, 1) | Identical before/after |
| 160x134 | (2, 1) | (2, 1) | Identical before/after |
| 121x82 | (2, 1) | (2, 1) | Identical before/after |

The regression also covers compact mode and nonzero frame origins. Compact
phones retain their one-column gutter and lose the blank top row; desktops
retain their existing one-row top pad and one-/two-column gutters. It checks
the width-truncated location text and the right gutter. Its pre-fix run
reported `0 passed; 1 failed`, with the location coordinates showing the
old margin; the fixed run reported `1 passed; 0 failed`.

Local commands through the worktree-pinned vendored wrapper:

- `./scripts/vendor-cargo.sh fmt --all -- --check` — passed (`rc=0`).
- `./scripts/vendor-cargo.sh --jobs 2 check -p xai-grok-pager --all-targets` — passed (`rc=0`).
- `./scripts/vendor-cargo.sh --jobs 2 clippy -p xai-grok-pager -- -D warnings` — passed (`rc=0`).
- `./scripts/vendor-cargo.sh --jobs 2 test -p xai-grok-pager --lib welcome_top_bar_margins_follow_phone_grid_and_preserve_desktop_frames -- --nocapture --test-threads=1` with `DSB_WELCOME_FRAME_DUMP=1` — `1 passed; 0 failed`; the before/after captures yielded five identical desktop text buffers.
- `./scripts/vendor-cargo.sh --jobs 2 test -p xai-grok-pager --lib views::welcome:: -- --test-threads=1` — `107 passed; 0 failed`. The logo-tier fixture now uses 28 frame rows to retain its original 27-row content budget after removing the phone's top pad; its 13-row draft cap and compact-logo assertion remain.
- `./scripts/vendor-cargo.sh --jobs 2 test -p xai-grok-pager --lib effective_narrow_keeps_the_phone_grid_at_a_smaller_text_size -- --test-threads=1` — `1 passed; 0 failed`.
- `./scripts/vendor-cargo.sh --jobs 2 test -p xai-grok-pager --lib layout_uses_full_width_phone_bars_and_ends_a_phone_on_its_footer -- --test-threads=1` — `1 passed; 0 failed`.

Test runs used `RUST_MIN_STACK=16777216`. The full local pager library suite
was not run because of the previously recorded CoreAudio probe limitation;
the broad vendored workspace test remains the separate post-merge CI job.
No G0–G6 gate status changes here.

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

## Verification — Path A resume cache usage (2026-09-29)

Path A now restores the cumulative usage ledger from `usage.json` before the
session actor accepts requests. The first resumed write subtracts that restored
baseline, preserving saved billing totals while adding only new usage. The
incoming-turn fold cursor remains process-local: late interjections fold into
the same-process row, and a turn-number collision after restart is renumbered.
Legacy cache/main-loop fields and a missing usage file on an existing session
remain explicitly unknown/incomplete; explicit new-schema zeros remain known.
The main-loop response subtotal observed after an upgrade is still persisted
while its known flag remains false, and billing `modelCalls` is never used to
infer the missing legacy split. A malformed present summary is rejected rather
than restored as an unknown ledger.

Local production-crate verification passed using the per-worktree vendored
target and serialized build wrapper:

- `./scripts/vendor-cargo.sh check --all-targets` — passed.
- `./scripts/vendor-cargo.sh fmt --all -- --check` — passed.
- `./scripts/vendor-cargo.sh test -p xai-chat-state cache_session_sums_reported_halves_and_skips_unreported` — 1 passed.
- `./scripts/vendor-cargo.sh test -p xai-grok-shell --lib session::usage_file::tests` — 16 passed, including disk round-trip, legacy/explicit-zero distinction, malformed present-summary rejection, and process-local cursor collision.
- `./scripts/vendor-cargo.sh test -p xai-grok-shell --lib resume_restores_live_baseline` — 1 passed; persisted billing/cache values were not double-counted, and a same-process late interjection folded into the current row.
- `./scripts/vendor-cargo.sh test -p xai-grok-shell --lib restored_ledger_reaches_session_usage_and_deepseek_status_requests` — 1 passed through both production status extensions.
- `./scripts/vendor-cargo.sh test -p xai-grok-shell --lib response_preserves_observed_turn_subtotal_with_unknown_history` — 1 passed.
- `./scripts/vendor-cargo.sh test -p xai-grok-pager --lib session_usage_block_shows_restored_cache_totals_and_unknown_legacy_history` — 1 passed.
- `./scripts/vendor-cargo.sh test -p xai-grok-shell --lib legacy_main_loop_subtotal_survives_repeated_resume_roundtrips` — 1 passed; two post-upgrade calls survive the first disk resume, a third survives another resume, the old billing count remains separate, and historical completeness stays false.

These are targeted local results, not a gate-table status change. Hosted PR CI
for #339 completed successfully at source head
`9a9c58f8af1afdc8412f83629d38430fb25d149f` ([run 36466155902](https://github.com/innocarpe/deepseek-build/actions/runs/36466155902)):
`CI / required`, `changes`, `grok fmt`, `grok clippy`, and `changelog move` passed.
Path-filtered jobs `fmt`, `clippy`, `test`, `semver`, `release verify retry`,
`session close`, `vendor build queue`, `worktree ownership`, and `npm` were
skipped. The separate `CI grok test` workflow runs on `main` pushes, so it was
not part of this pre-merge PR run.

## Verification — post-merge vendored run and legacy usage snapshots (2026-09-29)

The post-merge `CI grok test` run
[36468767878](https://github.com/innocarpe/deepseek-build/actions/runs/36468767878)
tested merge head `aa6ff15c79d0a50825ef07da5317df8fcb5491d7` and exited 101.
The complete job log, rather than the truncated `gh run view --log-failed`
excerpt, showed that the only failed target was `xai-grok-pager --lib`:
`session_usage_block_formats_tokens_and_cost` and
`session_usage_block_absent_cost_is_unknown_not_free`. Both snapshots omitted
the existing renderer's `Note: main-loop response count before tracking is
unknown.` line. The complete vendor run also reported chat-state 393 passed,
shell 7,140 passed / 0 failed / 5 ignored, tools 3,386 passed / 0 failed / 3
ignored, and pager 10,294 passed / 2 failed / 5 ignored.

The correction adds only that rendered line to the two affected snapshots;
their token, cost, and alignment expectations remain unchanged. No renderer,
knowledge flag, assertion, or test behavior was changed. The run was created at
`2026-09-28T18:56:46Z`, its job started at `19:20:26Z`, and `Test vendored
workspace` started at `19:23:44Z` and failed at `19:49:23Z` (job completed at
`19:49:26Z`). These timestamps keep queue time separate from the test step.

The targeted local snapshot rerun passed: `./scripts/vendor-cargo.sh test -p xai-grok-pager --lib session_usage_block_` reported 6 passed, 0 failed, 0 ignored. The corrective [full vendored workspace run #36479367295](https://github.com/innocarpe/deepseek-build/actions/runs/36479367295) completed successfully with no failed tests on source head `2775a2ace2eda23af284b03af552882d40efb8c9`; its `third_party/grok-build` tree was `c2b792a0e7e9cccb86e9233acd61b8a9ca9774e5`. Job `109120947540` completed at `2026-09-28T21:04:42Z`; `Test vendored workspace` ran from `20:30:09Z` through `21:04:40Z`. The full job log reports chat-state 393 passed, pager library 10,296 passed / 0 failed / 5 ignored, pager binary 39 passed, shell 7,140 passed / 0 failed / 5 ignored, and tools 3,386 passed / 0 failed / 3 ignored.

The run tested the exact snapshot and source tree; the subsequent GATES-only evidence commit does not change `third_party/grok-build`. Verify that the final PR head and merge commit retain the same vendor tree hash; no full-suite rerun is needed for that documentation-only commit.

## Verification — new-file Git EOL policy (2026-09-29)

Path A and the thin `dsb-tools` create paths now resolve `text` / `eol` from
Git for each true new file when the filesystem backend confirms the resolved
path is on the host. The lookup uses literal, NUL-delimited argv paths from the
nearest existing parent and a bounded process/read deadline. Backends without
that guarantee, including the current ACP adapter, use the platform fallback;
host `.gitattributes` are not assumed to describe their target. The create result,
stored bytes, `FileWritten.content`, and `EditsApplied.new_string` share the
same normalized content. Existing CRLF edits remain CRLF under a conflicting
new-file attribute; snippet-safe overwrites and unreadable targets fail closed.
Regression coverage includes real non-Git platform-fallback writes and a mock
filesystem whose path overlaps a host Git repository with the opposite EOL
attribute; it uses fallback and writes only to the mock backend.

`cargo check -p dsb-tools --all-targets` passed and
`cargo test -p dsb-tools --lib` printed `79 passed; 0 failed`. With the
worktree cargo bin on `PATH`,
`./scripts/vendor-cargo.sh check -p xai-grok-tools --all-targets` passed and
`./scripts/vendor-cargo.sh check -p xai-grok-workspace --all-targets` passed,
compiling the ACP adapter against the conservative capability default. The
`PATH="$HOME/.cargo/bin:$PATH" ./scripts/vendor-cargo.sh test -p xai-grok-tools --lib search_replace:: -- --test-threads=1`
printed `125 passed; 0 failed; 3262 filtered out`.
`./scripts/check-path-a-linkage.sh` printed `PASS`; `git diff --check` passed.
This is feature verification and does not flip a spec-readiness gate.

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
- `claude_import::tests::gate_load_claude_env_returns_empty_when_marker_set` failed in its fresh process under the ordinary local environment. The shell test seeds its own marker cache, while the workspace reader has a separate marker gate; `RUST_MIN_STACK=16777216 _GROK_CLAUDE_MARKER_OVERRIDE=1 ./scripts/vendor-cargo.sh test -p xai-grok-shell --lib claude_import::tests::gate_load_claude_env_returns_empty_when_marker_set -- --nocapture` passed. The relevant shell/workspace source and test blobs are identical to base `dd0c8b938211e1c5d587c0dbaafff8baa20fe62d`; no test or production code was changed here.
- `session::workflow::registry::tests::save_through_symlinked_session_root_stays_in_canonical_project` and `util::config::persist::tests::no_home_cwd_config_resolves_slot_not_follow` failed with `/var/...` versus `/private/var/...` path spellings under `RUST_MIN_STACK=16777216`. With only `TMPDIR` changed to the canonical realpath of the existing temp directory, both passed in the same already-built test binary. Their relevant shell source/test blobs also match base `dd0c8b938211e1c5d587c0dbaafff8baa20fe62d`; the observed failure is the temp-path alias affecting those assertions.
- `.github/workflows/ci.yml` runs the vendored PR checks `grok fmt` and strict `grok clippy`; it does not run the vendored shell test suite. `.github/workflows/ci-grok-test.yml` runs the full vendored suite on a `main` push after merge (or by manual dispatch), with `RUST_MIN_STACK=16777216`; it is not a PR check. Record the PR checks and the post-merge full-suite result separately.

## Verification — full vendored workspace follow-up (2026-09-28)

The earlier [run #36404759770](https://github.com/innocarpe/deepseek-build/actions/runs/36404759770)
remains a historical failure: its unchanged shell library had one
`set_consent_answer_is_monotonic_per_account` failure. It is not the current
workspace result. The later [main run #36409062503](https://github.com/innocarpe/deepseek-build/actions/runs/36409062503)
completed with `conclusion=success` on source commit
`035fe245e5b80b7ec28545d0df575578c4197d14`; its `grok test` job and
`Test vendored workspace` step both succeeded. This entry records only the
observation that run #36409062503 completed successfully.

## Verification — current cache-resume source full vendored workspace run (2026-09-29)

The [main full-vendor run #36468767878](https://github.com/innocarpe/deepseek-build/actions/runs/36468767878)
completed with `conclusion=failure` on source SHA
`aa6ff15c79d0a50825ef07da5317df8fcb5491d7`, the merge commit for cache PR #339.
GitHub reports `createdAt=2026-09-28T18:56:46Z`; the `grok test` job started at
`2026-09-28T19:20:26Z`; its `Test vendored workspace` step ran
`cargo test --workspace --no-fail-fast`, started at `2026-09-28T19:23:44Z`,
and completed with `conclusion=failure` at `2026-09-28T19:49:23Z`. The job
completed at `2026-09-28T19:49:26Z`.

The full job log for job `109094750211` reports one failed target,
`xai-grok-pager --lib`: `10294 passed; 2 failed; 5 ignored`. The failures were
`app::status_blocks::tests::session_usage_block_formats_tokens_and_cost`
(snapshot `session_usage_block_full`) and
`app::status_blocks::tests::session_usage_block_absent_cost_is_unknown_not_free`
(snapshot `session_usage_block_absent_cost`). Both snapshot diffs show the
rendered line `Note: main-loop response count before tracking is unknown.`
missing from the expected snapshot. Cargo reported one failed target and the
job annotation recorded exit code 101.

Source comparison from `aa6ff15^1` to `aa6ff15` confirms that cache PR #339
added this line in
`third_party/grok-build/crates/codegen/xai-grok-pager/src/app/status_blocks.rs`
when `usage.num_turns_known` is false; the merge did not update the two expected
snapshot files. The observed full-vendor failure is therefore the pager
snapshots missing the new status line from the merged source. This docs PR does
not modify source or snapshots.

This hosted result is separate from the historical local full-run failures
above and from main run #36409062503, which completed successfully on
`035fe245e5b80b7ec28545d0df575578c4197d14`. It does not establish that an earlier
local failure was intermittent, fixed, or caused by a particular factor.

## Verification — jump-to-bottom chip padding and right edge (2026-09-29)

The chip's two padding columns now paint as half blocks (`▐` left, `▌` right —
the chip's colour as the block's ink, the canvas as its background), so the
padding reads at half its former width while both columns stay whole cells
inside the click target. A wide glyph starting on the chip's last column can no
longer leave a terminal-default cell just right of the chip: before painting,
the chip repaints that exposed trailing cell in the glyph's own style (`bg`
falls back to `theme.bg_base` when the glyph carries none), so the cell
`Buffer::diff` forces out after the chip replaces the glyph's leading half
carries a real background instead of the content's hidden `Cell::EMPTY`. Themes
whose chip or canvas colour is `Color::Reset` (terminal-native) keep the plain
padded cell, because a half block painted with default colours would ink the
default foreground.

Local evidence on this host, serially through the vendored wrapper:

- `./scripts/vendor-cargo.sh fmt --all -- --check` — passed (`rc=0`).
- `./scripts/vendor-cargo.sh --allow-concurrent check -p xai-grok-pager
  --all-targets` — passed (`check_rc=0`).
- `./scripts/vendor-cargo.sh test -p xai-grok-pager --lib follow_indicator_tests
  -- --nocapture` — `8 passed; 0 failed` (`follow_rc=0`).
- `./scripts/vendor-cargo.sh test -p xai-grok-pager --lib jump_to_bottom_tests
  -- --nocapture` — `11 passed; 0 failed` (`jump_rc=0`), including the 55x41
  frame dump `line 37      ▐Jump to bottom (click) ↓▌      8:28 AM`, the
  emitted-cell checks on the chip's row, and the padding hover/click tests at
  every label width.

The first run of the two filters failed 3 of 11 `jump_to_bottom_tests`
(`jump_rc=101`); the failure output prints `left: Reset` / `right: Reset` for
the chip colours, i.e. those tests read a `Reset` palette in that process. They
now take `theme::cache::pin_theme()` like the other colour-sensitive frame
tests, which pins the process to `GrokNight` at truecolor; the 11/11 rerun above
is the result.

Observation vs inference: the `Reset` palette in the failing run and the 11/11
rerun are measured. Why the palette was `Reset` is **inferred**, not measured —
this host's dsb tool shell exports `NO_COLOR=1` and `TERM=dumb` (`env`),
`color_support` maps `NO_COLOR` to `ColorLevel::None`
(`theme/color_support.rs:98-100`), and `quantize_color` maps every colour to
`Color::Reset` at that level (`theme/color_support.rs:232`), so
`Theme::current()` in an agent shell resolves to the all-`Reset` palette. No
cross-test interference is claimed: the filtered run executes only the filtered
tests. The pager's full `--lib` suite is not run locally on purpose (this host
blocks in the CoreAudio voice probe, measured 2026-09-27); the broad run is CI's
workspace `grok test`.

[PR #347 CI](https://github.com/innocarpe/deepseek-build/actions/runs/36509381823)
on source commit `d42e9cd` passed `required`, `grok clippy`, `grok fmt`,
`changelog move`, and `changes`. The branch was rebased onto the day's `main`
(the PR was `CONFLICTING`, and a conflicted head runs no CI at all); the two
conflicts were this unit's own `CHANGELOG.md` bullet and its
`docs/architecture/GROK_VENDOR.md` overlay row, both resolved keeping the
`Unreleased` items main had added.
