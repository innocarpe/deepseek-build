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

> **Status of the automatic path.** The trusted publisher is **enrolled**, and
> `v6.0.1` published through `publish-npm.yml` on 2026-09-25 (the publish step
> itself took about 9 seconds once the asset existed). If a later tag fails at
> the publish step, the emergency path is the working route, and the
> troubleshooting order is in ADR 0012.
>
> Enrolling it needed 2FA enabled on the npm account first — npm requires
> interactive 2FA to modify package settings, and the account had it disabled.
> That is why the enrollment is a browser step and not part of this cycle.

## Where the time actually goes

| Step | Time | Why |
|------|------|-----|
| `npm publish` | ~10 s | The npm package is a thin JS wrapper (no compiled code, ADR 0009) |
| User `npm i -g` | ~10 s | `postinstall` downloads `deepseek-build-{ver}-{platform}.tar.gz` from GitHub Releases |
| Prebuilt build (cold, no default-branch cache) | 40–60+ min | Whole Grok TUI workspace compiled for `darwin-arm64`. `v6.0.1` took 46 minutes, of which the agent build was 41. `rust-cache` logged `No cache found` and sccache's Rust hit rate was 0.15% |
| Prebuilt build (warm default-branch cache) | recompile of the version-marked crates | The agent is always rebuilt so `--version` matches the package. With a restored `target/`, unchanged dependencies are not recompiled. The first release after the cache seed is still cold; the seed is what the release after that restores |

There is no wrapper-reuse fast path. Reusing the previous tag's agent binary
shipped a version string the npm postinstall check rejected. Local sccache
still helps a developer's own rebuilds. CI does not use it: a cache saved on
a tag ref cannot be restored by the next tag.

## Scripts

| Script | Role |
|--------|------|
| [`bump-version.sh`](../../scripts/bump-version.sh) | Single-command bump: `Cargo.toml`, `package.json`, `Cargo.lock`, `CHANGELOG.md` (moves the `Unreleased` items into the new version section), README.md version literals, `docs/product/versions/README.md`. Requires a clean tree; `--dry-run` previews the move. |
| [`reorder-changelog.sh`](../../scripts/reorder-changelog.sh) | Reorder CHANGELOG.md to the invariant (Unreleased top, versions newest-first) without touching non-version sections; `--check` exits non-zero if out of order. Reorders only — it does not move items between sections. |
| [`test-changelog-release.sh`](../../scripts/test-changelog-release.sh) | Hermetic regression test for the `Unreleased` move (`lib/changelog_release.py`); fixture CHANGELOGs in a temp dir, no network, no repo state |
| [`lib/version_log.py`](../../scripts/lib/version_log.py) | Fill the decision-log row's `PR #_(fill in)_` with the release PR number; called by `release.sh` the moment `gh pr create` returns (idempotent, so a resumed release re-runs safely) |
| [`release.sh`](../../scripts/release.sh) | Orchestrator: bump → MAJOR/README gate → verify → PR (`chore(release)`) → merge → tag `v{ver}` → wait for prebuilt assets → wait for CI publish → verify the registry. |
| [`npm-emergency-publish.sh`](../../scripts/npm-emergency-publish.sh) | **Emergency path only.** Local interactive publish that drives `npm login --auth-type=web` and any emailed code through the `aside` browser agent, so no person has to supply a number. |
| [`cache-guard.sh`](../../scripts/cache-guard.sh) | Release gate for spec 10 §1.9: overlay bench always, Path A bench only when `xai-grok-shell` is already compiled in the vendored target. Skips unless `DSB_RELEASE_CACHE_GUARD=1`; threshold via `DSB_CACHE_GUARD_THRESHOLD` (default 90). |

### Cache guard (spec 10 §1.9, release gate)

```bash
DSB_RELEASE_CACHE_GUARD=1 ./scripts/cache-guard.sh
```

`cache-first` is a claim about a curve, not a turn, and this gate scores it
before a release. Scenarios (plain and long dialogue, mixed message sizes, tool
loops, with and without the reasoning round-trip) run through the real turn
loop against a prefix-accounting mock provider that derives the cache hit from
the request bytes it receives. Two layers must both pass: the distinct-epoch
count is exactly 1 on a scenario that changes no §1.1 input (2 when one input
is changed by design), and the last-3-request hit rate is at or above
`DSB_CACHE_GUARD_THRESHOLD` (default 90%). A negative control perturbs the
prefix every turn and must fail the rate layer — if it passes, the bench
fails, because the mock stopped accounting from the request.

Without `DSB_RELEASE_CACHE_GUARD=1` both the wrapper and the gated tests skip, so
the ordinary `cargo test --workspace` never pays for the bench (the contract
tests inside each harness run always).

Two harnesses, one threshold. The overlay bench
([`crates/dsb-agent/tests/cache_guard.rs`](../../crates/dsb-agent/tests/cache_guard.rs))
drives the `dsb-agent` turn loop, including live tool execution. The Path A
bench
(`third_party/grok-build/crates/codegen/xai-grok-shell/src/session/helpers/spec10_path_a_cache_guard.rs`)
scores `assemble_spec10_path_a_turn` + `place_stable_body` after the Chat
Completions mapping. The wrapper runs that second bench only when the
vendored target already contains `libxai_grok_shell-*.rlib` or the
`xai_grok_shell-*` test harness. `cargo test --lib` leaves the harness and
not the rlib; a product build leaves the rlib. A missing artifact prints a
skip and exits 0 for that half — this script does not start a cold vendored
build (30–60+ minutes). The contract and test names are spec 10 §1.9 / §4.4.

### `release.sh` flags

| Flag | Meaning |
|------|---------|
| `--desc "…"` | One-line note for the new CHANGELOG section **when `Unreleased` is empty** (the versions-README row uses it either way); the move reports when items outrank it |
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

- **A bump moves the `Unreleased` items into the new section — verbatim.**
  They are the release's own record of what it shipped, so the bump does not
  summarize, rewrite or duplicate them; `--desc` is only a fallback for an
  empty `Unreleased`. Check the move in the preview before committing:
  `./scripts/bump-version.sh <ver> --dry-run` prints how many items would
  move into which section. Skipping the move is how `v5.7.0` shipped seven
  items still reading as unreleased while its own section held a single
  `--desc` line — the release record then said nothing about the release
  ([measured on the tag](#what-the-unreleased-move-prevents)).
- `bump-version.sh` inserts the new section directly below `Unreleased` and
  moves a drifted `Unreleased` back to the top; it **exits non-zero** if the
  file is not newest-first afterward.
- `reorder-changelog.sh` fixes a drifted file in place (one-time cleanup) and
  `--check` fails CI/humans that let the invariant rot. It reorders only — it
  does **not** move items between sections.
- **Keep the blank line after `## Unreleased`.** `## Unreleased` immediately
  followed by a version heading is not cosmetic: a branch that appends an item
  under `Unreleased` then merges into a glued tree gets that item **filed under
  the version heading with no conflict**, so nothing ever flags it. Measured:
  `#209` (cache attribution) landed under `5.7.0` and `#206` (SSH paste) under
  `6.0.0`, each byte-identical to `git merge-tree` — i.e. pure auto-merge, no
  human edit. With the blank line those same merges conflict, and the item is
  filed by hand. `bump-version.sh` now writes the blank line and refuses a
  glued junction; `reorder-changelog.sh --check` reports it and the in-place
  run repairs it. Pinned by `test-changelog-release.sh` case 8, which builds
  both merges and asserts one conflicts while the glued one does not.
- **The release PR number goes into the decision-log row.** `bump-version.sh`
  opens the row before the PR exists, so it writes `PR #_(fill in)_`; that
  placeholder used to survive forever — six rows on `main` read it (`4.0.4`,
  `5.2.0`, `5.2.2`, `5.5.3`, `5.5.4`, `6.0.0`) until they were filled by hand.
  `release.sh` now fills the row from the number `gh pr create` returns and
  commits it into the release PR, so the record lands with the release.
  `scripts/lib/version_log.py` does the edit and refuses a missing row, an
  absent PR column or a non-numeric number; re-running it (a resumed release)
  leaves a recorded number alone.
- Prereleases sort below their release (`4.0.4` > `4.0.4-beta.1` > `4.0.4-alpha.1`).

### What the Unreleased move prevents

`bump-version.sh` used to insert the new section without touching `Unreleased`.
Measured on the tags (`git show <tag>:CHANGELOG.md`, items counted below
`## Unreleased`):

| Tag | Unreleased items at release time | Its own section |
|-----|----------------------------------|-----------------|
| `v5.5.4` | 1 | — |
| `v5.6.0` | 0 | — |
| **`v5.7.0`** | **7** | one `--desc` line (`phone-width layout and Orca pane status`) |
| **`v6.0.0`** | **9** | one `--desc` line (`Grok Build base ported 1.0.0 to 1.0.41 … sync infrastructure`) |

Both tags shipped their items still filed as unreleased while the version
section held only the `--desc` line, so the release record did not name what the
release did. `v6.0.0` is the compounding case: its nine included the seven items
`v5.7.0` had already shipped (the TUI pane fit, the Orca status frame, the 2FA
publish path, the npm self-check, CRLF edits, the image path, CI publishing),
plus the `1.0.41` port and the sync-harness item.

`v5.6.0` (0 items) shows the healthy path needs no attention.

### What the glued junction caused

Fixing the move was not enough, because the *other* way an item reaches a
version section is a merge. With `## Unreleased` glued to the next heading, a
feature branch that adds an item under `Unreleased` merges into the released
tree **cleanly** — git appends the item to the section that now ends at the
glued heading — and the item is filed under a version nobody reviewed it for.
Measured after `v6.0.0` shipped, both reproduced with `git merge-tree` and
byte-identical to the real merge commit:

| PR | Item | Auto-merged into | Shipped? |
|----|------|------------------|----------|
| `#209` | cache-epoch attribution | `## 5.7.0` | no — the code landed after the `5.7.0` tag (`shape.rs` is absent from both tags) |
| `#206` | SSH image paste | `## 6.0.0` | no — merged `00:33`, the `v6.0.0` tag is `23:09` (`paste.rs` has the fix on `main`, not in the tag) |

Both were moved back under `## Unreleased` once the junction was repaired.

The move is covered by `scripts/test-changelog-release.sh` (hermetic: fixture
CHANGELOGs in a temp dir, no repo state touched) and runs in CI's `changelog`
job, so undoing it fails a required check rather than the next release. Case 8
builds the two branches and asserts the merge conflicts with the blank line and
stays silent without it — the regression cannot come back unnoticed.

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

> **Operational reality.** `v6.0.1` and `v6.0.2` both left the queue within
> seconds (2026-09-25 / 2026-09-26). Older tags did sit in the queue, so the
> manual asset fallback below stays. The matrix is Apple Silicon macOS
> (`darwin-arm64`) only. `publish-npm.yml` waits for the asset on a Linux
> runner, so a late manual attach still publishes, and the workflow can be
> re-run on demand. The wait stops early when `release-prebuilt` has already
> finished without the asset.

- **Always a full agent build.** The binary's version string has to match
  the package. `scope=full` is the only scope the job logs.
- **Rust cache across tags.** `release-prebuilt.yml` restores
  `Swatinem/rust-cache@v2` (`shared-key: release-prebuilt-darwin-arm64`,
  `save-if: false`) and uploads `release-rust-cache.tar.gz`.
  `seed-release-cache.yml` runs after that success, in the default-branch
  context, and saves the archive there. The next tag can restore a
  default-branch cache; it cannot restore the previous tag's own save.
  `CARGO_TERM_COLOR` and `CARGO_INCREMENTAL` are part of the cache key and
  must match in both workflows. An archive over 6 GiB is not uploaded, and
  a pack failure does not fail the release.
- **First cache.** `v6.1.0` ran before this workflow existed, so it
  uploaded no archive and the seed saved nothing. `workflow_dispatch` on
  `seed-release-cache.yml` builds the release agent on `main` and the
  following job restores that cache. A cold count of compiled crates fails
  that job.
- **Honest limits.** The release that produces the first archive is still
  cold. A runner-image Rust upgrade changes the cache key. Non-Apple-Silicon
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
- **Runners:** `wait for asset` is `ubuntu-latest` (timeout 100 minutes,
  inner deadline 90). `publish` is `macos-14` (timeout 20 minutes) and
  starts only after the asset exists, so it can execute the packaged
  `darwin-arm64` agent. npm trusted publishing supports GitHub-hosted
  runners only. A build that has already failed does not hold the wait
  until the deadline.
- **After publish** the job runs the user-facing path: a clean
  `npm install -g` and `dsb --version` / `deepseek-build --version`. That step
  runs whenever a publish happened or the version was already live — including
  when the verification step above it failed — because "can a user install it?"
  is the question it answers, and a skip would hide the answer.
- **Registry visibility.** A successful `npm publish` does not make the version
  immediately readable: the registry serves reads through a CDN with its own
  cache, and nothing in npm's documentation promises read-after-write. On the
  `5.7.0` release the verify step read one second after publish and got
  `E404`, while the registry's metadata records the version landing **76 s**
  after that read; the smoke step was skipped as a result. Every post-publish
  read therefore goes through
  [`scripts/verify-npm-version.sh`](../../scripts/verify-npm-version.sh), a
  bounded retry (300 s default, 5 s initial interval doubling to a 30 s
  ceiling). It prints the confirmed version on stdout and a diagnostic on
  timeout. `release.sh` and `npm-emergency-publish.sh` call the same helper, so
  the three publish paths cannot drift apart. The window can be shortened for
  a test with `DSB_NPM_VERIFY_TIMEOUT_SEC`.
  [`scripts/test-npm-verify-retry.sh`](../../scripts/test-npm-verify-retry.sh)
  covers it hermetically, with a local mock registry instead of the network.

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
