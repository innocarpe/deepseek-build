---
name: release
description: >
  Cut a DeepSeek Build release end-to-end without omissions: version bump,
  CHANGELOG newest-first invariant, MAJOR/README gate, chore(release) PR,
  merge, tag, prebuilt asset attach (manual fallback when CI is stuck), npm
  publish over OIDC trusted publishing, post-publish verification. Use when the
  user asks to release, bump, tag, publish to npm, or ship a version.
---

# Release (DeepSeek Build harness)

This skill is the **agent checklist** for cutting a release. It is not CI — it
is the completeness gate so nothing (bump, tag, assets, npm, CHANGELOG, README)
is silently skipped.

## Load these docs (in order)

1. `docs/contributing/release-cycle.md` — normative runbook (this skill mirrors it)
2. `docs/contributing/versioning.md` — SemVer fail-close rules
3. `docs/contributing/pr-body-standard.md` — PR narrative bar for the release PR
4. `docs/adr/0009-npm-prebuilt-binaries.md` — npm wrapper + prebuilt tarball model
5. `docs/adr/0012-npm-trusted-publishing.md` — OIDC publish path + emergency path
6. `docs/adr/0008-grok-build-base.md` — vendored Grok TUI (SOURCE_REV pin)

## Hard rules

1. **SemVer only:** full `MAJOR.MINOR.PATCH` everywhere (tag `v4.0.4`, never
   `4.0`). Never claim a release "at 1.0" unless `1.0.0` shipped + verified.
2. **CHANGELOG invariant (fail-close):** `# Changelog` → `## Unreleased` at the
   very top → version sections newest-first. After any bump, run
   `./scripts/reorder-changelog.sh --check` — non-zero exit means fix first.
3. **MAJOR gate (fail-close):** a MAJOR bump is blocked unless README's
   product-status banner already references the new major (`**5.0.0** …` row).
   Update `docs/product/` + README *before* running the release.
4. **Build from the tag tree, never a diverged worktree HEAD.** The tag may
   point at a commit the release worktree is not on. `git checkout v<ver>`
   first, then build.
5. **No silent asset skip.** `release-prebuilt.yml` tag runs routinely stay
   "queued" forever. If `gh release view v<ver> --json assets` shows no tarball
   for `darwin-arm64`, attach manually (fallback below) — do not publish an
   npm version whose binary is missing or stale. `publish-npm.yml` enforces
   this too: it waits for the asset and fails the run without it.
6. **Verify after publish** with a real global install + `dsb --version` +
   a `strings` check for the release's markers (e.g. `deepseek.com` status
   handling) on the installed binary.
7. **Publishing is CI's job (ADR 0012).** The tag push triggers
   `publish-npm.yml`, which publishes over OIDC trusted publishing — no npm
   token, no one-time code. Do not publish locally unless that path is broken;
   the emergency path is below.

## Standard cycle

```bash
# 1. Prepare (clean tree, on main)
git fetch origin && git checkout main && git pull

# 2. Bump + release orchestrator (creates PR, merges, tags, waits, publishes)
./scripts/release.sh 4.0.4 --desc "one-line release note"

# 3. Human verification
npm i -g @innocarpe/deepseek-build@4.0.4
dsb --version
```

`release.sh` stages: bump → MAJOR/README gate → verify → PR (`chore(release)`)
→ merge → tag → asset wait → **CI publishes over OIDC** (`publish-npm.yml`) →
registry verified. No npm token and no one-time code are involved.

## npm Trusted Publisher enrollment (one-time, npm website)

ADR 0012's automatic path only works once the package has a trusted publisher
configured on npmjs.com. **It is enrolled (2026-09-25)** for
`innocarpe/deepseek-build` + `publish-npm.yml`, so this section is here to
re-verify or redo it, not as a pending task.

Enrolling or editing it requires **interactive 2FA on the npm account** (npm
requires 2FA to modify package settings). Note that `npm trust list <package>`
is not a cheap read-only check any more — with 2FA enabled it answers `EOTP`
and waits for a browser approval. Read the package settings page instead.

Enrollment (browser, as the npm account):

1. Enable 2FA on the npm account (npm's setup page offers a security key; an
   authenticator app is the alternative).
2. Package → **Settings** → **Trusted Publisher** → **GitHub Actions**:
   - Organization or user: `innocarpe`
   - Repository: `deepseek-build`
   - Workflow filename: `publish-npm.yml` (filename only — no path; case-sensitive)
   - Environment: empty
   - Allowed actions: `npm stage publish` **and** `npm publish`
3. Read the saved connection back off the page to confirm.

The `aside` CLI drove this (signing in, approving the security key, reading
emailed codes from Gmail). It cannot perform the 2FA enrollment itself — that
needs the account holder's key or authenticator secret once.

## Account 2FA state (check before choosing a path)

```bash
npm profile get | grep two-factor      # currently: auth-and-writes
```

`auth-and-writes` means a **local** `npm publish` asks for proof of presence.
The OIDC path is unaffected, and `npm stage publish` is unaffected too — only
`npm stage approve` and a direct publish are interactive. The registered second
factor here is a **security key**, so the proof is a browser approval, not a
typed code.

## Emergency path (CI cannot publish)

```bash
./scripts/npm-emergency-publish.sh <ver>        # publishes locally
./scripts/release.sh <ver> --publish-only --local-publish   # same, in the orchestrator
```

It refuses to publish without the release asset, ensures an npm session (driving
`npm login --auth-type=web` through `aside exec` if needed), then runs the
publish **under a pty with `--browser=false`**, captures the 2FA approval URL
npm prints, and hands that URL to `aside exec` to approve with the registered
security key. **It does not ask a person for a number**, and no code or
single-use URL reaches a log, commit or PR.

Both pty and `--browser=false` are load-bearing: piped, npm refuses with `EOTP`
without offering the URL; with a browser configured it blocks on `Press ENTER`.

It carries no provenance attestation (there is no local OIDC provider), so
prefer fixing CI over using it. If the account ever gains an authenticator app,
`NPM_OTP=<code>` still works as an override.

## Manual asset fallback (reliable path when CI is stuck)

```bash
WT=/path/to/deepseek-build-release-4.0.4               # tag worktree
git -C "$WT" fetch origin && git -C "$WT" checkout v4.0.4
cd "$WT" && ./scripts/build-grok-pager.sh release        # cold build: 30-60+ min
# stage the fresh agent binary
cp "$WT/third_party/grok-build/target/release/xai-grok-pager-bin" \
   ~/.deepseek-build/bin/deepseek-build-agent
# attach tarball to the GitHub release (creates v4.0.4 if missing)
cd "$WT" && ./scripts/package-release-binaries.sh --upload
gh release view v4.0.4 --json assets                      # confirm tarball
# then let CI publish (it re-runs on demand):
gh workflow run publish-npm.yml --ref v4.0.4
# or, only if CI itself is unavailable:
./scripts/npm-emergency-publish.sh 4.0.4
```

## Post-publish verification checklist

- [ ] `npm i -g @innocarpe/deepseek-build@<ver>` succeeds
- [ ] `dsb --version` prints `<ver>`
- [ ] `strings $(command -v dsb)` (or the agent binary) shows the release's
      behavior markers — e.g. for the image-fix release, the DeepSeek
      endpoint / status markers — not the pre-fix build
- [ ] `gh release view v<ver> --json tagName,assets` shows the `darwin-arm64` tarball
- [ ] CHANGELOG still newest-first: `./scripts/reorder-changelog.sh --check`
- [ ] README version literals match `<ver>`
- [ ] `npm view @innocarpe/deepseek-build@<ver> dist.attestations` shows a
      provenance attestation (CI path; the emergency path has none)

## Anti-patterns

| Bad | Why |
|-----|-----|
| Publishing from a worktree whose HEAD ≠ tag | Ships unreleased/unmerged code as the binary |
| Skipping asset check because CI "should" attach | CI queue routinely never runs; 404s for users |
| `4.0` / `v4` in any public text | SemVer fail-close (Agents.md) |
| Bumping to a new MAJOR with stale README banner | Tag ships ahead of the documented story |
| Claiming done after `npm publish` | Unverified global install is not a release |
| Local publish when CI could publish | Loses provenance and leaves the irreversible step off the audit trail |
| `npm stage publish` then hand-approving every release | Works (it is how `5.6.0` shipped), but `approve` is interactive by design — it cannot run unattended |
| "Fixing" an OIDC 403 by adding a bypass token | The bypass is being retired; fix the publisher or the workflow instead |

## Done means

- [ ] Tag `v<ver>` exists on origin and has the platform tarball attached
- [ ] `@innocarpe/deepseek-build@<ver>` is live and a clean global install works
- [ ] CHANGELOG newest-first + README literals/banner consistent with `<ver>`
- [ ] No omission: bump, PR, merge, tag, assets, publish, verify all happened
