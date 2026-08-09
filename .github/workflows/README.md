# GitHub Actions

Build/test CI only — **no process-police** (PR title/label regex bots).

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
| **`required`** | **always** | aggregate; branch protection requires this |

```text
PR / push
   └─ changes
         ├─ fmt ──────┐
         ├─ clippy ───┤  (parallel if rust)
         ├─ test ─────┤
         ├─ semver ───┤  (if version files)
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
| Base restore | PR clippy/test jobs first restore the stable `main` cache family with `save-if: false` |
| PR/main save layer | A second cache step saves `*-pr-${{ github.event.pull_request.number }}` on pull requests and the stable `*-v2` key on `main` |
| `cache-workspace-crates` | `true`, so workspace artifacts are retained instead of caching only dependencies |
| `cache-on-failure` | `true`, so a failing PR clippy/test run can still save artifacts for reruns |
| `cache-provider` | Explicitly `github` |

The first run for a PR may restore a stable base cache from `main`, compile the
delta, and save a PR-number-scoped layer even when the job fails. Later runs of
the same PR can restore that PR layer directly.

GitHub Actions caches are immutable: for a given PR/cache key, the first
successfully saved entry is the one later runs restore until the key changes or
GitHub evicts it. Bumping the cache family to `v2` lets `main` and PRs create
fresh entries instead of full-hitting older `v1` caches and reporting
`Cache up-to-date`.

Sibling PRs do not share saved PR layers because pull request runs are scoped to
their `refs/pull/.../merge` refs. They can still restore the stable `main` base
cache before saving their own PR layer. This improves rerun latency but uses more
GitHub cache storage; old entries remain subject to GitHub cache eviction.

`fmt` and `grok fmt` do not use rust-cache because they only run rustfmt and do
not compile artifacts.

## Path filters (skip expensive work)

| Filter | Paths |
|--------|--------|
| **rust** | `crates/**`, `Cargo.toml`, `Cargo.lock`, toolchain, rustfmt, clippy, this workflow |
| **semver** | `Cargo.toml`, `package.json`, check-semver scripts |

Docs-only → `changes` + `required` only (~seconds).

## Not in CI

| Skip | Why |
|------|-----|
| `./scripts/smoke-dogfood.sh` | Largely duplicates `test` + `semver` (re-runs workspace tests + version checks). Keep as **local / release** checklist |
| Live DeepSeek API | Secrets; optional in the smoke script when `DEEPSEEK_API_KEY` is set |
| npm publish | Owner-gated ADR 0007 |
| Process-police | Docs + review harness |

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
./scripts/check-semver.sh && node npm/scripts/check-version-match.js
# release / dogfood checklist (not a CI job):
./scripts/smoke-dogfood.sh
```
