# 130 — Test change evidence

**Status:** implemented
**Primary path:** Path A full-screen agent
**Purpose:** report host-observed changes to existing tests in the turn result and its durable turn record.

## Contract

1. Observe only successful, structured `FileWritten` receipts from in-process file tools when the session selected host-backed `LocalFs`. Client-backed ACP filesystem writes are outside this local-path trust boundary and are omitted. A receipt supplies the file path and exact content before and after the write. Never use `HEAD` or a worktree diff as attribution evidence.
2. Use the first successful receipt's `previous_content` as the baseline for that path. This includes content that was already dirty when the turn began. A pre-existing dirty test with no successful write receipt is not attributed to the agent. A `None` previous value is accepted only when `is_new_file` is true; inconsistent receipts are omitted instead of treating an unknown baseline as empty.
3. Reconcile a path's receipt chain. If one receipt's prior-content fingerprint does not match the previous receipt's after-content fingerprint, omit that path. At turn completion, read the path again; omit it if its content no longer matches the last successful receipt. A missing file has an empty final content for this comparison.
4. Compare test names and token signatures from the baseline and final file snapshots. Ignore formatting and comments. A signature is a BLAKE3 digest of declaration syntax tokens excluding the declaration name. For Rust tests it includes all directly preceding attributes; for pytest it includes decorators on the test function or method (and on its enclosing `Test*` class). Decorator/attribute token changes therefore count as declaration changes. Source text and literal values are never retained in the report or turn delta.
5. Report only `modified`, `deleted`, `added`, and `renamed` criteria. New tests are separate from existing-test changes. A test with the same name and signature moved to another file is unchanged and produces no delete/add pair. A unique test with the same signature but a different test name may be reported as renamed.
6. Attach a short host-generated note to a completed or cancelled Path A result only when at least one criterion changed. The run loop flushes its replay buffer inline before terminal settlement; settlement persists and emits the note directly before the durable terminal, without waiting for a `FlushReplay` acknowledgement from that same loop. Label the note as observed write evidence; it does not establish test execution, test pass, or semantic weakening. Persist the same bounded evidence in `turn_result.json`'s turn delta, including the terminal snapshot returned for an evidence-bearing cancellation.
7. Shell commands, MCP writes, external edits, client-backed ACP filesystem writes, non-Git workspaces, files outside the discovered Git worktree, unsupported test conventions, inconsistent receipts, parse errors, and oversized files do not produce attribution. Do not infer an agent change for these cases.

## Supported criteria

| Language | Included convention | Excluded by this minimum |
|---|---|---|
| Rust | `.rs` functions directly marked `#[test]` or `#[tokio::test]`; inline modules are included; preceding attributes such as `#[ignore]`, `#[should_panic]`, and `#[tokio::test(...)]` arguments are fingerprinted | Other test attributes/frameworks, tests in separately compiled module files, macro-generated tests |
| Go | `_test.go` top-level `TestXxx` functions with a `*testing.T` parameter | Benchmarks, fuzz tests, `TestMain`, aliases for `testing`, generated tests |
| Python | Default pytest filenames `test_*.py` and `*_test.py`; module-level `test_*` functions and `Test*` class methods, including decorated definitions; decorators are fingerprinted | `unittest` discovery, custom pytest configuration/plugins, generated or dynamically attached tests |

The classifier does not inspect assertions for correctness. A token change is evidence of a changed test declaration, not proof that coverage or test strength improved or weakened.

## Bounds and report shape

- At most 1 MiB per before/after file snapshot, 512 criteria per file, 64 receipt paths per turn, 128 UTF-8 bytes per test name, 256 bytes per relative path, and 40 report entries.
- Any parse failure or size/count overflow omits the affected path. Report-entry overflow sets a `truncated` flag.
- Each entry contains only a relative path, language, change category, count, and optional previous relative path. Test names are used only in memory to match criteria and are not reported. The report excludes source, literals, hashes, absolute paths, commands, and secret values.
- The user-visible note names the supported conventions and limits, states that evidence comes from successful structured writes in a host-backed Git worktree, and distinguishes host evidence from model claims about test execution.

## Verification plan

- Rust unit tests cover `#[test]` and `#[tokio::test]` detection plus preceding attribute changes.
- Go and pytest unit tests cover their supported filename/declaration conventions and exclusions; pytest fixtures cover `@pytest.mark.parametrize` and `@pytest.mark.skip` on functions and class methods.
- Tracker tests cover pre-existing dirty baselines, mismatched new-file metadata, changed/deleted/added criteria, comment/format-only edits, same-name/signature file moves, edit-then-revert net-zero, failed writes with no receipt, receipt-chain mismatch, final-content mismatch, unsupported/non-Git paths, parse/size failures, output bounds, and absence of source/literal values in serialized reports.
- Path A settlement tests use a real host-backed receipt and the shipped completion/cancel settlement handlers. The fixture keeps `create_test_actor_ex`'s event receiver open but undrained, so a self-queued `FlushReplay` can neither fail fast nor receive a test ack. A short bound proves these common settlement handlers finish without waiting for that ack; this does not exercise the sampler or the run loop's select branch. The test checks one persisted and gateway host note before the durable terminal, completed/cancelled snapshots, and the `turn_result.json` delta while test-pass claims remain separate.

This feature is observational. It does not block valid test edits, add CI policing, or require a review harness.
