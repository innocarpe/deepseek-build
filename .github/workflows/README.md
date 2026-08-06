# GitHub Actions

Product CI only — **no process-police**.

## Primary workflow

| Workflow | File | Required check name |
|----------|------|---------------------|
| **product-ci** | [`product-ci.yml`](./product-ci.yml) | **`gate`** (check run name) |

### Jobs (parallel when paths match)

| Job | When | Work |
|-----|------|------|
| `detect-paths` | always | `dorny/paths-filter` |
| `cargo-fmt` | rust paths | `cargo fmt --check` |
| `cargo-clippy` | rust paths | clippy |
| `cargo-test-workspace` | rust paths | `cargo test --workspace` |
| `offline-smoke` | product/smoke paths | `./scripts/smoke-dogfood.sh` |
| `cargo-npm-version` | version files | SemVer match (no compile) |
| **`gate`** | **always** | aggregate; branch protection requires this |

```text
PR / push
   └─ detect-paths
         ├─ fmt ──────┐
         ├─ clippy ───┤  (parallel if rust)
         ├─ test ─────┤
         ├─ smoke ────┤  (parallel if product paths)
         ├─ semver ───┤  (if version files)
         └─ gate (always) ← require this check only
```

### Why not only separate path-filtered workflows?

GitHub treats **never-run required checks as failing**. Docs-only PRs would never
report `rust-test` and could not merge. So:

- **Work** is still split and parallel (jobs, not one serial mega-script).
- **One always-on `gate`** is the only status check you should require.

Legacy split YAMLs (`rust-fmt.yml`, etc.) were removed in favor of this pattern.

## Caching

| Setting | Value |
|---------|--------|
| Action | `Swatinem/rust-cache@v2` |
| `shared-key` | `workspace-v1` |
| `save-if` | `main` only |
| `cache-on-failure` | `true` |

## Path filters (skip expensive work)

| Filter | Paths |
|--------|--------|
| **rust** | `crates/**`, `Cargo.toml`, `Cargo.lock`, toolchain, rustfmt, clippy, this workflow |
| **smoke** | rust + `package.json`, `npm/**`, smoke/install scripts |
| **semver** | `Cargo.toml`, `package.json`, check-semver scripts |

Docs-only → detect + gate only (~seconds).

## Not in CI

| Skip | Why |
|------|-----|
| Live DeepSeek API | Secrets; local optional |
| npm publish | Owner-gated ADR 0007 |
| Process-police | Docs + review harness |

## Branch protection / ruleset

Require **exactly**:

```text
gate
```

(Do **not** require individual fmt/clippy/test job names — path-skipped jobs would break merges.)

## Local mirrors

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -W clippy::all
cargo test --workspace
./scripts/smoke-dogfood.sh
./scripts/check-semver.sh && node npm/scripts/check-version-match.js
```
