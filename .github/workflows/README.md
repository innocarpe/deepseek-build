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
| `smoke` | product/smoke paths | `./scripts/smoke-dogfood.sh` |
| `semver` | version files | Cargo/npm SemVer match (no compile) |
| **`required`** | **always** | aggregate; branch protection requires this |

```text
PR / push
   └─ changes
         ├─ fmt ──────┐
         ├─ clippy ───┤  (parallel if rust)
         ├─ test ─────┤
         ├─ smoke ────┤  (parallel if product paths)
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
| `shared-key` | `workspace-v1` |
| `save-if` | `main` only |
| `cache-on-failure` | `true` |

## Path filters (skip expensive work)

| Filter | Paths |
|--------|--------|
| **rust** | `crates/**`, `Cargo.toml`, `Cargo.lock`, toolchain, rustfmt, clippy, this workflow |
| **smoke** | rust + `package.json`, `npm/**`, smoke/install scripts |
| **semver** | `Cargo.toml`, `package.json`, check-semver scripts |

Docs-only → `changes` + `required` only (~seconds).

## Not in CI

| Skip | Why |
|------|-----|
| Live DeepSeek API | Secrets; local optional |
| npm publish | Owner-gated ADR 0007 |
| Process-police | Docs + review harness |

## Branch protection / ruleset

Require **exactly**:

```text
required
```

(Do **not** require individual `fmt` / `clippy` / `test` job names — path-skipped jobs would break merges.)

**Migration note:** older rulesets may still require `gate` (from the former
`product-ci` workflow). Update the required check name to **`required`** when
this lands on `main`.

## Local mirrors

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -W clippy::all
cargo test --workspace
./scripts/smoke-dogfood.sh
./scripts/check-semver.sh && node npm/scripts/check-version-match.js
```
