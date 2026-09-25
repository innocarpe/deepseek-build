# Release cycle (fix → PR → merge → npm → verify)

**Status:** Normative runbook for the standard change cycle on `4.x`+ releases.

Agents: load the [`release` skill](../../skills/release/SKILL.md) before cutting
a release — it is the agent-facing checklist for everything below.

The default cycle for any product-affecting change is:

```
fix on a branch → PR (pr-authoring skill) → merge (merge commit)
→ ./scripts/release.sh <version> → npm i -g @innocarpe/deepseek-build@<version> → verify
```

**No step needs a human.** The tag push triggers
[`publish-npm.yml`](../../.github/workflows/publish-npm.yml), which publishes
over OIDC **trusted publishing** — there is no npm token to hold and no
one-time code to type ([ADR 0012](../adr/0012-npm-trusted-publishing.md)).
`release.sh` waits for that run and verifies the registry afterwards. The one
remaining human step is a **one-time** enrollment on the npm website
(§Trusted Publisher enrollment), not a per-release gate.

> **Status of the automatic path.** It is implemented and the trusted publisher
> is **enrolled** (2026-09-25), but an OIDC publish has **not yet run**: no
> release has been cut since. The next release is the first real exercise. If
> the tag run fails at the publish step, the emergency path is the working
> route, and the troubleshooting order is in ADR 0012.
>
> Enrolling it needed 2FA enabled on the npm account first — npm requires
> interactive 2FA to modify package settings, and the account had it disabled.
> That is why the enrollment is a browser step and not part of this cycle.

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
| [`release.sh`](../../scripts/release.sh) | Orchestrator: bump → MAJOR/README gate → verify → PR (`chore(release)`) → merge → tag `v{ver}` → wait for prebuilt assets → wait for CI publish → verify the registry. |
| [`npm-emergency-publish.sh`](../../scripts/npm-emergency-publish.sh) | **Emergency path only.** Local interactive publish that drives `npm login --auth-type=web` and any emailed code through the `aside` browser agent, so no person has to supply a number. |

### `release.sh` flags

| Flag | Meaning |
|------|---------|
| `--desc "…"` | One-line note seeded into CHANGELOG + versions README |
| `--no-publish` | Stop after assets are ready |
| `--skip-bump` / `--skip-pr` / `--skip-tag` | Resume from a later stage |
| `--publish-only` | Skip everything, wait for assets + publish |
| `--local-publish` | **Emergency:** publish from this machine instead of waiting for CI (interactive npm login) |
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
- **MAJOR bump gate (fail-close):** cutting a new major (e.g. `6.0.0`) is
  rejected by `release.sh` unless the product version history
  (`docs/product/versions/README.md`) already logs that major (the row is
  added by `bump-version.sh`). The user-facing README intentionally stays
  clean of release-process signals; the version log is the internal record
  the gate checks — the tag must never ship ahead of the documented story.

## Human gates

1. **Trusted Publisher enrollment (one-time, npm website)** — the automatic
   publish path needs a trusted publisher configured for
   `@innocarpe/deepseek-build`. Enrolling it requires **interactive 2FA** on the
   npm account (npm requires 2FA to modify package settings). See
   §Trusted Publisher enrollment. After this is done once, releases need no
   human.
2. **PR body review** — read the generated `chore(release)` PR before it is merged;
   fill CHANGELOG release notes before running the script if a placeholder remains.
3. **CHANGELOG/README honesty** — run `./scripts/reorder-changelog.sh --check`
   before merging anything that touches the changelog; keep the newest-first
   invariant green.

## Account 2FA state, and what it means per path

Measured on 2026-09-25, after the `5.6.0` release:

```
$ npm profile get | grep two-factor
two-factor auth: auth-and-writes
```

| Path | Needs proof of presence? | Why |
|------|--------------------------|-----|
| **CI publish** (`publish-npm.yml`, OIDC) | **No** | npm accepts the workflow's OIDC token; 2FA is not part of the exchange |
| `npm stage publish` | No | staging defers proof of presence by design |
| `npm stage approve <id>` | **Yes** | approval is the interactive step (CLI or npmjs.com) |
| **Local `npm publish`** (emergency path) | **Yes** | the account is `auth-and-writes`, so a direct publish asks for 2FA |

`auth-and-writes` is what makes a **local** publish need 2FA. It does not affect
the OIDC path, which is why the trusted publisher is the default and a local
publish is only a fallback. `auth-only` would relax the local case;
`auth-and-writes` is the stricter and currently chosen setting.

**How the proof is supplied here.** The registered 2FA method is a **security
key**, not an authenticator app, so there is no code to type. Under a terminal
npm prints a browser URL and polls until that page is approved:

```
Authenticate your account at:
https://www.npmjs.com/auth/cli/<single-use>
```

`npm-emergency-publish.sh` captures that URL and hands it to `aside exec`, which
approves it with the registered key. Two measured requirements make this work:

- **A pty is required.** Piped, npm answers `EOTP` and exits without offering
  the URL at all. The script runs the publish under `script`.
- **`--browser=false` is required.** With a browser configured, npm prints
  `Press ENTER to open in the browser...` and blocks on that read instead of
  polling. Every npm call in the script passes the flag.

## Trusted Publisher enrollment (one-time)

**Enrolled 2026-09-25.** Re-check it on the package settings page:
`https://www.npmjs.com/package/@innocarpe/deepseek-build/access` -> *Trusted
Publisher*. The saved connection should read
`innocarpe/deepseek-build publish-npm.yml`, `Permissions: npm publish, npm
stage publish`.

`npm trust list <package>` is **not** a read-only check any more: with 2FA
enabled it answers `EOTP` and waits for a browser approval. Approving it does
then print the connection, which is a valid way to confirm it.

Enrollment, on npmjs.com as the package's maintainer:

1. Enable 2FA on the npm account (security key, or an authenticator app).
2. Package -> **Settings** -> **Trusted Publisher** -> **GitHub Actions**:
   - Organization or user: `innocarpe`
   - Repository: `deepseek-build`
   - Workflow filename: `publish-npm.yml` — **filename only, no path, case-sensitive**
   - Environment: empty
   - Allowed actions: `npm stage publish` **and** `npm publish`
3. Read the saved connection back off the page to confirm it stored what you
   intended (npm does not validate on save; errors surface only at publish).

The `aside` CLI can drive steps 2-3, including approving the security-key
prompt; step 1 is an account-identity action and needs a person.

## CI notes (`release-prebuilt.yml`)

> **Operational reality (do not rely on CI):** the `release-prebuilt.yml` tag
> run routinely stays stuck in the GitHub Actions queue ("queued" forever), so
> release assets have been attached **manually from a local tag worktree** for
> every shipped version. The wait loop in `release.sh` is a fast-path when CI
> works; the manual fallback below is the reliable path. The current release
> matrix is intentionally limited to Apple Silicon macOS (`darwin-arm64`).
> `publish-npm.yml` is built for the same reality: it waits for the asset, so a
> late manual attach still publishes, and it can be re-run on demand.

- **Change-scope fast path:** if `third_party/` is unchanged since the previous
  SemVer tag, the vendored agent binary is extracted from that release's
  tarball and only `dsb-cli` is rebuilt (minutes). If the previous Apple
  Silicon tarball is missing, the job falls back to a full build.
  Checkout uses `fetch-depth: 0` + explicit tag fetch so `prev_tag` is not
  empty (v5.1.0 regressed to `scope=full prev_tag=none` under shallow clone).
- **sccache:** `RUSTC_WRAPPER=sccache` + `SCCACHE_GHA_CACHE=true`; every run
  prints `sccache --show-stats` so hit rate is visible in the job log.
- **rust-cache:** `Swatinem/rust-cache@v2` covers the product workspace and
  `third_party/grok-build` (registry + target) for warmer full rebuilds.
- **Honest limits:** GitHub runner queue time is outside our control; a first
  full build after a large vendored change is still long; non-Apple-Silicon
  users are outside the current product support boundary and receive a clear
  unsupported-platform message.

## CI notes (`publish-npm.yml`)

- **Trigger:** `v*.*.*` tag push. `workflow_dispatch` exists only to retry a
  tag that already contains the workflow; a tag older than the workflow has no
  dispatchable run, so cut a new tag or use the emergency path.
- **Refuse-to-publish gates** (all before `npm publish` runs): full SemVer;
  the tag exists and `HEAD` **is** the tag commit; `package.json` matches the
  tag; the `darwin-arm64` tarball is attached; the packaged agent **executes**
  and reports the release version; the version is not already on the registry.
- **Auth:** `id-token: write` only. No npm secret is read.
- **Provenance:** generated automatically by trusted publishing, and
  `--provenance` is passed explicitly. Verify after publish with
  `npm view @innocarpe/deepseek-build@<version> dist.attestations`.
- **Runner:** `macos-14`, so the job can execute the packaged `darwin-arm64`
  agent. npm trusted publishing supports GitHub-hosted runners only.
- **After publish** the job runs the user-facing path: a clean
  `npm install -g` and `dsb --version` / `deepseek-build --version`.

### Emergency path (CI cannot publish)

```bash
./scripts/npm-emergency-publish.sh <version>
# or, from the orchestrator:
./scripts/release.sh <version> --publish-only --local-publish
```

It refuses to publish without the release asset, then drives
`npm login --auth-type=web` through `aside exec`: the browser agent signs in as
the npm account, completes the CLI session, and reads any emailed one-time code
from Gmail. **No person is asked for a number**, and no code or token is
written to a log, commit or PR. The local publish carries **no provenance
attestation** (there is no local OIDC provider), so it is a fallback, not a
peer of the CI path.

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
4. **Publish:** re-run the CI job (`gh workflow run publish-npm.yml --ref
   v<version>`) so the publish still goes through OIDC and gets provenance;
   use the emergency path only if CI itself is unavailable.

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
