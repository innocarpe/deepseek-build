# Post-stack 5.5.0 stabilization - 2026-08-09

## Objective

Stabilize the merged 5.x vision stack documentation on `main` before the
release lane publishes `5.5.0`.

## Current floor

- Base branch: `origin/main`
- Base commit: `b2c1122beae8f7fabf64a1856049ef491f654a24`
- On-main version: `5.5.0`
- npm latest: `5.2.2`
- GitHub Latest release: `v5.2.2`

`5.5.0` is merged on `main`, but publication is still pending. Until the release
lane completes, installer-visible npm and GitHub Latest evidence remains `5.2.2`.

PR #149 already resolved the final-main formatting failure, including the exact
rustfmt hunks from run `31264907520`, fmt job `93121419547`, and merged with
green remote CI. This branch starts after that fix.

## Scope

- Remove only mechanical whitespace reported by the post-stack diff check in the
  specified evidence and version-index files.
- Reconcile current control and user-facing documentation after PRs
  `#125`, `#130`, `#135` through `#147`, and `#149` merged.
- Mark vision-train prompts as archived so they are not reused as live execution
  instructions.

## Validation boundary

Local validation for this branch is static only:

- `git diff --check origin/main..HEAD`
- conflict-marker scan
- stale current-state phrase scan
- textual version consistency checks
- diff and changed-file inspection

No local build, test, Cargo, rustc, rustfmt, clippy, install, setup, generation,
or LSP validation is allowed for this stabilization branch. Remote CI is the
validator after supervisor review and any authorized PR push.

## Non-goals

- No tag creation.
- No GitHub release publication.
- No npm publication.
- No behavior changes.
- No historical evidence rewriting beyond mechanical whitespace cleanup.
