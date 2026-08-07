# Release cycle (fix → PR → merge → npm → verify)

**Status:** Normative runbook for the standard change cycle on `4.x`+ releases.

Agents: load the [`release` skill](../../skills/release/SKILL.md) before cutting
a release — it is the agent-facing checklist for everything below.

The default cycle for any product-affecting change is:

```
fix on a branch → PR (pr-authoring skill) → merge (merge commit)
→ ./scripts/release.sh <version> → npm i -g @innocarpe/deepseek-build@<version> → verify
```

Only the npm publish step can need a human, and only when npm actually demands
a one-time code (EOTP) — a publish-capable token (granular/automation) publishes
with no OTP at all. Everything else in the release half is scripted so a single
change costs minutes, not a build marathon.

## Where the time actually goes

| Step | Time | Why |
|------|------|-----|
| `npm publish` | ~10 s | The npm package is a thin JS wrapper (no compiled code, ADR 0009) |
| User `npm i -g` | ~10 s | `postinstall` downloads `deepseek-build-{ver}-{platform}.tar.gz` from GitHub Releases |
| Prebuilt build (cold, full vendored change) | 30–60+ min | Whole Grok TUI workspace (1300+ packages) compiled per platform |
| Prebuilt build (wrapper-only change) | ~1–3 min | `third_party/` unchanged → agent binary reused from previous release tarball (fast path) |
| Prebuilt build (pure version bump) | ~1–3 min | Same fast path; only `dsb-cli` rebuilt so `--version` matches |

The 60–70 min local builds seen before were cold builds on a loaded machine
(parallel worktrees). sccache (local and CI) plus the change-scope fast path
turn repeated builds into incremental ones.

## Scripts

| Script | Role |
|--------|------|
| [`bump-version.sh`](../../scripts/bump-version.sh) | Single-command bump: `Cargo.toml`, `package.json`, `Cargo.lock`, `CHANGELOG.md`, README.md version literals, `docs/product/versions/README.md`. Requires a clean tree; `--dry-run` previews. |
| [`reorder-changelog.sh`](../../scripts/reorder-changelog.sh) | Reorder CHANGELOG.md to the invariant (Unreleased top, versions newest-first) without touching non-version sections; `--check` exits non-zero if out of order. |
| [`release.sh`](../../scripts/release.sh) | Orchestrator: bump → MAJOR/README gate → verify → PR (`chore(release)`) → merge → tag `v{ver}` → wait for prebuilt assets → `npm publish` (OTP only if npm demands it). |

### `release.sh` flags

| Flag | Meaning |
|------|---------|
| `--desc "…"` | One-line note seeded into CHANGELOG + versions README |
| `--no-publish` | Stop after assets are ready (hand off to human) |
| `--skip-bump` / `--skip-pr` / `--skip-tag` | Resume from a later stage |
| `--publish-only` | Skip everything, wait for assets + publish |
| `--platform ID` | Platform to wait for (default: detect from `npm/lib/platform.js`) |
| `--wait-all` | Retained for future matrix expansion; currently waits for the single `darwin-arm64` target |
| `--timeout SEC` | Asset wait timeout (default 5400) |

## CHANGELOG convention (fail-close)

`CHANGELOG.md` must stay in this shape, **always**:

```
# Changelog
## Unreleased        ← pinned at the very top (even when empty)
## <newest version>  ← newest-first, SemVer descending
## …
## 3.0.0
## <older sections, non-version sections, notes — untouched>
```

- `bump-version.sh` inserts the new section directly below `Unreleased` and
  moves a drifted `Unreleased` back to the top; it **exits non-zero** if the
  file is not newest-first afterward.
- `reorder-changelog.sh` fixes a drifted file in place (one-time cleanup) and
  `--check` fails CI/humans that let the invariant rot.
- Prereleases sort below their release (`4.0.4` > `4.0.4-beta.1` > `4.0.4-alpha.1`).

## README policy

- **Pure version literals** (the `# → deepseek-build X.Y.Z` / `dsb X.Y.Z` /
  `check-semver: ok (X.Y.Z)` lines under the install header) are updated
  automatically by `bump-version.sh`.
- **MAJOR bump gate (fail-close):** cutting a new major (e.g. `5.0.0`) is
  rejected by `release.sh` unless README's product-status banner already
  references that major (a `**5.0.0** …` row). Update `docs/product/` + README
  *before* running the release — the tag must never ship ahead of the
  documented story.

## Human gates

1. **npm OTP (only if demanded)** — `release.sh` publishes without `--otp` first;
   if npm returns EOTP it pauses for the one-time code (or `NPM_OTP` env).
2. **PR body review** — read the generated `chore(release)` PR before it is merged;
   fill CHANGELOG release notes before running the script if a placeholder remains.
3. **CHANGELOG/README honesty** — run `./scripts/reorder-changelog.sh --check`
   before merging anything that touches the changelog; keep the newest-first
   invariant green.

## CI notes (`release-prebuilt.yml`)

> **Operational reality (do not rely on CI):** the `release-prebuilt.yml` tag
> run routinely stays stuck in the GitHub Actions queue ("queued" forever), so
> release assets have been attached **manually from a local tag worktree** for
> every shipped version. The wait loop in `release.sh` is a fast-path when CI
> works; the manual fallback below is the reliable path. The current release
> matrix is intentionally limited to Apple Silicon macOS (`darwin-arm64`).

- **Change-scope fast path:** if `third_party/` is unchanged since the previous
  release tag, the vendored agent binary is extracted from that release's
  tarball and only `dsb-cli` is rebuilt (seconds). If the previous Apple
  Silicon tarball is missing, the job falls back to a full build.
- **sccache:** `RUSTC_WRAPPER=sccache` + `SCCACHE_GHA_CACHE=true` make full
  builds incremental across runs (cache scoping follows the GitHub Actions
  cache service; the fast path is the guaranteed win).
- **Honest limits:** GitHub runner queue time is outside our control; a first
  full build after a vendored change is cold; non-Apple-Silicon users are
  outside the current product support boundary and receive a clear
  unsupported-platform message.

### Manual asset fallback (when CI never runs)

1. **Build from the tag tree** (never from a worktree HEAD that differs from
   the tag): `git -C <wt> fetch origin && git -C <wt> checkout v<version>`
   then `./scripts/build-grok-pager.sh release` in that worktree.
2. **Stage the agent binary:** copy
   `third_party/grok-build/target/release/xai-grok-pager-bin` to
   `~/.deepseek-build/bin/deepseek-build-agent` (replace the stale copy) and
   keep the `deepseek-build` / `dsb` wrappers in sync.
3. **Attach assets:** `./scripts/package-release-binaries.sh --upload` (creates
   the GitHub release `v<version>` if missing and uploads the local platform
   tarball). Confirm with `gh release view v<version> --json assets`.
4. **Publish:** `./scripts/release.sh <version> --publish-only` (or
   `npm publish --access public` directly in `npm/`; OTP only if npm demands it).

## Verification after publish

```bash
npm i -g @innocarpe/deepseek-build@<version>
dsb --version                      # matches <version>
dsb --resume                       # resumes most-recent TUI session (if any)
# quit a full-screen session → hint should read: dsb --resume <id>
```

## Related

- [versioning.md](./versioning.md) — SemVer fail-close rules
- [pr-body-standard.md](./pr-body-standard.md) — PR narrative bar
- [pull-requests.md](./pull-requests.md) — units, titles, labels, merge
- [ADR 0009](../adr/0009-npm-prebuilt-binaries.md) — prebuilt npm install
- [ADR 0008](../adr/0008-grok-build-base.md) — vendored Grok TUI (SOURCE_REV pin)
