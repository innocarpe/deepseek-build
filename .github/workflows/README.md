# GitHub Actions

Build/test CI only — **no process-police** (PR title/label regex bots).

## Workflows

| Workflow | File | Trigger | Role |
|----------|------|---------|------|
| **CI** | [`ci.yml`](./ci.yml) | PR / push to `main` | Build + test; one required check |
| **CI grok test** | [`ci-grok-test.yml`](./ci-grok-test.yml) | push to `main`, grok paths | Vendored workspace tests. Not a required check |
| **release-prebuilt** | [`release-prebuilt.yml`](./release-prebuilt.yml) | `v*.*.*` tag | Build + attach the `darwin-arm64` release tarball |
| **seed-release-cache** | [`seed-release-cache.yml`](./seed-release-cache.yml) | `release-prebuilt` completes | Re-save that build's Rust artifacts on `main` |
| **publish-npm** | [`publish-npm.yml`](./publish-npm.yml) | `v*.*.*` tag | Publish to npm over OIDC trusted publishing ([ADR 0012](../../docs/adr/0012-npm-trusted-publishing.md)) |

## Primary workflow

| Workflow | File | Required check name |
|----------|------|---------------------|
| **CI** | [`ci.yml`](./ci.yml) | **`required`** (check run name) |

GitHub UI shows checks as `CI / <job>` (e.g. `CI / fmt`, `CI / test`, `CI / required`).

### Jobs (parallel when paths match)

| Job | When | Work |
|-----|------|------|
| `changes` | always | `dorny/paths-filter` |
| `fmt` | rust paths | `cargo fmt --check` |
| `clippy` | rust paths | clippy |
| `test` | rust paths | `cargo test --workspace` |
| `semver` | version files | Cargo/npm SemVer match (no compile) |
| `npm` | `npm/**`, `package.json` | `node --test npm/test/*.js` (hermetic, seconds) |
| `release_verify` | release/publish paths | publish → verify retry guard, hermetic (no network) |
| `session close` | harness-close paths | finish line stays in the loaded description window; brief fixtures (hermetic) |
| `grok fmt` | grok paths | `cargo fmt --all -- --check` in `third_party/grok-build` (no rust-cache) |
| `grok clippy` | grok paths | `cargo clippy --workspace -- -D warnings` there (libs and bins, not tests) |
| **`required`** | **always** | aggregate; branch protection requires this |

`CI grok test` is not a job of this workflow. It runs `cargo test --workspace`
in `third_party/grok-build` on `main` when grok paths change, in its own
concurrency group, so a docs push cannot cancel it. Grove identity tests stay
behind the off-by-default `grove-identities` feature. Do not require that
check: docs PRs never start the workflow, and GitHub fails a required check
that did not run.

```text
PR / push
   └─ changes
         ├─ fmt ─────────┐
         ├─ clippy ──────┤  (parallel if rust)
         ├─ test ────────┤
         ├─ semver ──────┤  (if version files)
         ├─ npm ─────────┤  (if npm paths)
         ├─ release_verify ┤  (if release/publish paths)
         ├─ session close ┤  (if harness-close paths)
         ├─ grok fmt ────┤  (parallel if grok paths)
         ├─ grok clippy ─┤
         └─ required (always) ← require this check only
```

### Why not only separate path-filtered workflows?

GitHub treats **never-run required checks as failing**. Docs-only PRs would never
report `test` and could not merge. So:

- **Work** is still split and parallel (jobs, not one serial mega-script).
- **One always-on `required`** is the only status check you should require.

## Caching

| Setting | Value |
|---------|--------|
| Action | `Swatinem/rust-cache@v2` |
| Compile cache families | `workspace-clippy-v2`, `workspace-test-v2`, `grok-build-clippy-v2`, `grok-build-test-v2` |
| Who saves | `main` only (`save-if` is false on pull requests) |
| Who restores | Pull requests restore the default-branch entry for that key |
| `cache-workspace-crates` | `true` on CI jobs. A full hit still rebuilds changed workspace crates; the archive is about the size of the dependency cache. `cache-all-crates` stays `false` there. |
| `cache-on-failure` | `true` on `main`, so a failing main run can still save |
| `cache-provider` | Explicitly `github` |

Pull requests do not save a `*-pr-N` copy. That copy was immutable for the
life of the key, and the vendored clippy archive was about 2 GB per PR. On
2026-09-26 twelve entries totaled 11.37 GB against the 10 GB included limit,
and the oldest entries were the ones a release needs. A PR rerun restores
`main` and recompiles its delta.

GitHub Actions caches are immutable: the first successful save of a key is
the one later runs restore until the key changes or GitHub evicts it.

`fmt` and `grok fmt` do not use rust-cache because they only run rustfmt and do
not compile artifacts.

`RUST_*` and `CARGO_*` are part of the cache key. `CI grok test` sets
`RUST_MIN_STACK`, so it does not share `grok clippy`'s archive. That is
deliberate: the test job's stack size must not retarget the clippy key.

## Path filters (skip expensive work)

| Filter | Paths |
|--------|--------|
| **rust** | `crates/**`, `Cargo.toml`, `Cargo.lock`, toolchain, rustfmt, clippy, this workflow |
| **grok** | `third_party/grok-build/**`, grok patches, grok build/test scripts, `docs/architecture/GROK_VENDOR.md`, this workflow |
| **semver** | `Cargo.toml`, `package.json`, check-semver scripts |
| **npm** | `npm/**`, `package.json`, this workflow |
| **release_verify** | `scripts/verify-npm-version.sh`, its test + mock, `release.sh`, `npm-emergency-publish.sh`, `publish-npm.yml` |

Docs-only → `changes` + `required` only (~seconds).

## Not in CI

| Skip | Why |
|------|-----|
| `./scripts/smoke-dogfood.sh` | Largely duplicates `test` + `semver` (re-runs workspace tests + version checks). Keep as **local / release** checklist |
| Live DeepSeek API | Secrets; optional in the smoke script when `DEEPSEEK_API_KEY` is set |
| Process-police | Docs + review harness |

## Release workflows (not PR checks)

Neither release workflow reports a check on pull requests, so neither belongs in
branch protection.

| Workflow | Publishes? | Notes |
|----------|-----------|-------|
| `release-prebuilt.yml` | No | Builds and attaches the release tarball; `contents: write`. Restores the default-branch Rust cache and uploads an archive; it does not save a tag-scoped cache |
| `seed-release-cache.yml` | No | After a successful prebuilt run, saves that archive on `main` so the next tag can restore it |
| `publish-npm.yml` | **Yes** | OIDC on the macOS job. The wait runs on Linux and stops early if `release-prebuilt` has already failed |

`publish-npm.yml` refuses to publish unless the release asset exists and the
packaged agent executes and reports the release version (ADR 0009 ordering), so
it is safe to re-run: the ordering gate and an "already published" check both
guard the registry write.

**npm publish is no longer owner-gated** (ADR 0012 amended ADR 0007). The
trusted publisher must be enrolled on npmjs.com once — see
[release-cycle.md](../../docs/contributing/release-cycle.md) §Trusted Publisher
enrollment.

## Branch protection / ruleset

Require **exactly**:

```text
required
```

(Do **not** require individual `fmt` / `clippy` / `test` job names — path-skipped jobs would break merges.)

## Local mirrors

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -W clippy::all
cargo test --workspace
# vendored grok, from third_party/grok-build (libs and bins only):
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
./scripts/check-semver.sh && node npm/scripts/check-version-match.js
node --test npm/test/*.js
# workflow lint (matches what CI relies on):
actionlint .github/workflows/*.yml
# release / dogfood checklist (not a CI job):
./scripts/smoke-dogfood.sh
```
