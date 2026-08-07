# Release cycle (fix → PR → merge → npm → verify)

**Status:** Normative runbook for the standard change cycle on `4.x`+ releases.

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
| [`bump-version.sh`](../../scripts/bump-version.sh) | Single-command bump: `Cargo.toml`, `package.json`, `Cargo.lock`, `CHANGELOG.md`, `docs/product/versions/README.md`. Requires a clean tree; `--dry-run` previews. |
| [`release.sh`](../../scripts/release.sh) | Orchestrator: bump → verify → PR (`chore(release)`) → merge → tag `v{ver}` → wait for prebuilt assets → `npm publish` (OTP only if npm demands it). |

### `release.sh` flags

| Flag | Meaning |
|------|---------|
| `--desc "…"` | One-line note seeded into CHANGELOG + versions README |
| `--no-publish` | Stop after assets are ready (hand off to human) |
| `--skip-bump` / `--skip-pr` / `--skip-tag` | Resume from a later stage |
| `--publish-only` | Skip everything, wait for assets + publish |
| `--platform ID` | Platform to wait for (default: detect from `npm/lib/platform.js`) |
| `--wait-all` | Wait for all matrix platforms, not just the local one |
| `--timeout SEC` | Asset wait timeout (default 5400) |

## Human gates

1. **npm OTP (only if demanded)** — `release.sh` publishes without `--otp` first;
   if npm returns EOTP it pauses for the one-time code (or `NPM_OTP` env).
2. **PR body review** — read the generated `chore(release)` PR before it is merged;
   fill CHANGELOG release notes before running the script if a placeholder remains.

## CI notes (`release-prebuilt.yml`)

- **Change-scope fast path:** if `third_party/` is unchanged since the previous
  release tag, the vendored agent binary is extracted from that release's
  tarball and only `dsb-cli` is rebuilt (seconds). If the previous tarball is
  missing for a platform, the job falls back to a full build.
- **sccache:** `RUSTC_WRAPPER=sccache` + `SCCACHE_GHA_CACHE=true` make full
  builds incremental across runs (cache scoping follows the GitHub Actions
  cache service; the fast path is the guaranteed win).
- **Honest limits:** GitHub runner queue time is outside our control; a first
  full build after a vendored change is cold; publishing before all platforms
  attach means other-platform users get a 404 until those assets land (only the
  publishing machine's platform is waited on by default — use `--wait-all`
  when other platforms matter).

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
