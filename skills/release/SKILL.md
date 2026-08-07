---
name: release
description: >
  Cut a DeepSeek Build release end-to-end without omissions: version bump,
  CHANGELOG newest-first invariant, MAJOR/README gate, chore(release) PR,
  merge, tag, prebuilt asset attach (manual fallback when CI is stuck), npm
  publish (no-OTP first), post-publish verification. Use when the user asks to
  release, bump, tag, publish to npm, or ship a version.
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
5. `docs/adr/0008-grok-build-base.md` — vendored Grok TUI (SOURCE_REV pin)

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
   for the publishing platform, attach manually (fallback below) — do not
   publish an npm version whose binary is missing or stale.
6. **Verify after publish** with a real global install + `dsb --version` +
   a `strings` check for the release's markers (e.g. `deepseek.com` status
   handling) on the installed binary.

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
→ merge → tag → asset wait → npm publish (no-OTP first, EOTP fallback asks for
the one-time code or `NPM_OTP`).

## Manual asset fallback (reliable path when CI is stuck)

```bash
WT=~/Projects/OpenSources/deepseek-build-release-4.0.4   # tag worktree
git -C "$WT" fetch origin && git -C "$WT" checkout v4.0.4
cd "$WT" && ./scripts/build-grok-pager.sh release        # cold build: 30-60+ min
# stage the fresh agent binary
cp "$WT/third_party/grok-build/target/release/xai-grok-pager-bin" \
   ~/.deepseek-build/bin/deepseek-build-agent
# attach tarball to the GitHub release (creates v4.0.4 if missing)
cd "$WT" && ./scripts/package-release-binaries.sh --upload
gh release view v4.0.4 --json assets                      # confirm tarball
# publish
./scripts/release.sh 4.0.4 --publish-only                 # or: cd npm && npm publish --access public
```

## Post-publish verification checklist

- [ ] `npm i -g @innocarpe/deepseek-build@<ver>` succeeds
- [ ] `dsb --version` prints `<ver>`
- [ ] `strings $(command -v dsb)` (or the agent binary) shows the release's
      behavior markers — e.g. for the image-fix release, the DeepSeek
      endpoint / status markers — not the pre-fix build
- [ ] `gh release view v<ver> --json tagName,assets` shows the tarball
- [ ] CHANGELOG still newest-first: `./scripts/reorder-changelog.sh --check`
- [ ] README version literals match `<ver>`

## Anti-patterns

| Bad | Why |
|-----|-----|
| Publishing from a worktree whose HEAD ≠ tag | Ships unreleased/unmerged code as the binary |
| Skipping asset check because CI "should" attach | CI queue routinely never runs; 404s for users |
| `4.0` / `v4` in any public text | SemVer fail-close (Agents.md) |
| Bumping to a new MAJOR with stale README banner | Tag ships ahead of the documented story |
| Claiming done after `npm publish` | Unverified global install is not a release |

## Done means

- [ ] Tag `v<ver>` exists on origin and has the platform tarball attached
- [ ] `@innocarpe/deepseek-build@<ver>` is live and a clean global install works
- [ ] CHANGELOG newest-first + README literals/banner consistent with `<ver>`
- [ ] No omission: bump, PR, merge, tag, assets, publish, verify all happened
