# ADR 0012 — npm publishing via OIDC trusted publishing

- **Status:** Accepted
- **Date:** 2026-09-25
- **Amends:** [ADR 0007](./0007-npm-packaging.md) (publish is no longer human-gated) · [ADR 0009](./0009-npm-prebuilt-binaries.md) (asset ordering unchanged)
- **Gate:** first proven by the first release cut after this ADR

## Context

ADR 0007 made `npm publish` **human-gated** (OTP/2FA). Every release since has
therefore needed a person present at publish time: `scripts/release.sh` used a
publish-capable token when there was one, otherwise stopped and asked for a
one-time code.

Two developments made that the wrong default:

1. **npm is retiring the bypass.** As of August 2026, tokens that bypass 2FA
   can no longer perform account-identity or account-governance actions, and
   from January 2027 they cannot publish directly either. "Have a bypass token
   in `~/.npmrc`" stops working.
2. **Trusted publishing exists and is strictly better.** npm supports OIDC
   trusted publishing from GitHub Actions: the workflow exchanges a
   short-lived, workflow-scoped OIDC token for a publish credential. There is
   no long-lived secret to leak, rotate or revoke, and npm generates
   [provenance attestations](https://docs.npmjs.com/generating-provenance-statements)
   automatically for a public package built from a public repository.

The cost of the old gate was not only a person's attention. It was that the
release's final step — the one that cannot be undone — was the one step outside
CI, taken from a laptop, with the least evidence recorded.

### The constraint this ADR cannot design around

Enrolling a trusted publisher — and, in general, modifying a package's settings —
requires **interactive 2FA** on the npm account. At the time of writing,
`@innocarpe/deepseek-build`'s maintainer account had 2FA **disabled**
(`npm profile get` → `two-factor auth: disabled`), and npm's package-settings
form redirected the unenrolled account to the account 2FA setup page. The npm
CLI said the same thing:

```
$ npx npm@latest trust github @innocarpe/deepseek-build --file publish-npm.yml \
    --repo innocarpe/deepseek-build --allow-publish --allow-stage-publish
npm error code E403
npm error 403 403 Forbidden - POST .../trust - Please enable 2fa for your account
```

This is not a design problem to engineer around. It is npm protecting the
account: a security key or an authenticator secret has to be registered by a
person, once. A browser agent can drive everything around it — signing in,
reading an emailed code, approving the security key prompt, saving the form —
which is how the enrollment was ultimately done.

So this ADR has two parts: an automatic publish path that needs no human, and an
honest statement of the one-time enrollment step that does need one. The
enrollment was completed on 2026-09-25 (after 2FA was enabled on the account);
the publish path itself has not yet run (see Consequences).

### How `5.6.0` actually shipped

`5.6.0` was released the same evening, before this ADR's workflow existed on a
tag, and it did **not** use the OIDC path. It went out through **staged
publishing**, with the approval supplied by hand:

```
npm stage publish                  # no 2FA: staging defers proof of presence
npm stage approve <stage-id>       # 2FA required: this is the interactive step
```

The account had just been switched to 2FA and the approval needed a person with
the registered security key, so the release paused between the two commands. The
registry record for `5.6.0` shows this route: `_npmUser.approver` is present
(absent on `5.5.4` and earlier), and `_npmVersion` is `12.1.0`, the CLI version
that first had `npm stage`.

That is a workable release route, but it is not the one this ADR chooses: the
`approve` step is interactive by design, so it cannot run unattended. It is
recorded here because it is what the first post-2FA release actually did, and
because it is the fallback if a tag predates `publish-npm.yml`.

## Decision

### Publish path: tag → asset → CI publishes over OIDC

```
./scripts/release.sh <ver>
  └─ tag v<ver> pushed
       ├─ release-prebuilt.yml  → builds + attaches darwin-arm64 tarball
       └─ publish-npm.yml       → waits for that asset, verifies it, publishes
```

`publish-npm.yml` is the only publishing path. It runs on the `v*.*.*` tag push
with the two things trusted publishing requires:

```yaml
permissions:
  id-token: write   # OIDC token exchange with the registry
  contents: read    # no token secret is read anywhere
```

npm's requirements, verified against `docs.npmjs.com/trusted-publishers`
(2026-09-04 revision) and the npm CLI itself:

| Requirement | Where it is satisfied |
|-------------|----------------------|
| npm CLI ≥ `11.5.1`, Node ≥ `22.14.0` | `setup-node` at Node `24`; the workflow raises a stale bundled npm only if it is older than `11.5.1` |
| `id-token: write` permission | job-level `permissions` |
| Workflow filename matches the npm configuration exactly, `.yml`/`.yaml`, in `.github/workflows/` | `.github/workflows/publish-npm.yml` |
| GitHub-hosted runner | `macos-14` |
| `repository.url` in `package.json` matches the repo | `git+https://github.com/innocarpe/deepseek-build.git` |

The ordering contract from ADR 0009 is preserved and strengthened. The job
refuses to publish unless the release asset exists **and** the packaged agent
executes and reports the release version; a missing or stale tarball fails the
run instead of shipping an npm version whose binary is absent.

`macos-14` rather than `ubuntu-latest` is deliberate: `darwin-arm64` is the only
released platform, and running the packaged agent is the strongest available
check that the asset is really the release build.

### The human step, and how it is done when it is needed

Enrolling the trusted publisher is done by a browser agent, not by a person
typing a code into a terminal. The `aside` CLI drives the npm website as the
maintainer's logged-in session, and reads any emailed code from Gmail itself.

The enrollment itself has to happen **after 2FA is enabled on the account**,
because npm requires 2FA to modify package settings:

1. Enable 2FA on the npm account (the npm website offers a security key; a
   time-based authenticator is the alternative if a security key is not at
   hand).
2. Add the trusted publisher for the package:
   - Provider **GitHub Actions**
   - Organization or user **`innocarpe`**
   - Repository **`deepseek-build`**
   - Workflow filename **`publish-npm.yml`**
   - Environment: **empty**
   - Allowed actions: **`npm stage publish` and `npm publish`**
3. Verify by reading the saved connection back off the page.

Step 1 is the only part a machine cannot do unattended: npm's setup page expects
a physical security key or an authenticator secret, and approving that is an
account-identity action by design.

**Enrolled 2026-09-25**, with the values above (environment empty, both stage and
direct publish allowed). Verified by reading the connection back off the package
settings page: a single connection, `innocarpe/deepseek-build publish-npm.yml`,
`Permissions: npm publish, npm stage publish`. Note that
`npm trust list <package>` is no longer a cheap read-only check — with 2FA
enabled it answers `EOTP` and waits for a browser approval.

### Emergency path: local interactive publish

If CI cannot publish (Actions unavailable, publisher misconfigured, or a release
must land immediately), `scripts/npm-emergency-publish.sh` still publishes from
this machine **without asking a person for a number**:

1. It refuses to run unless the release tarball is attached to the GitHub
   release (ADR 0009 ordering is not bypassed by the emergency path).
2. It ensures an npm session, driving `npm login --auth-type=web` through
   `aside exec` if there is none.
3. It runs the publish **under a pty with `--browser=false`**, captures the 2FA
   approval URL npm prints, and passes that URL to `aside exec`, which approves
   it with the account's registered security key.
4. It publishes and verifies the registry.

Both flags are load-bearing and were measured, not guessed: piped, npm answers
`EOTP` and exits without offering an approval URL; with a browser configured it
prints `Press ENTER to open in the browser...` and blocks on that read instead
of polling. Because the account's second factor is a security key, there is no
emailed code in this flow at all — an earlier version of this script looked for
one and would have stalled.

The login and approval URLs carry single-use tokens, so they are never echoed to
a log, commit or PR. `--local-publish` on `release.sh` is the same path wired into the
orchestrator. Provenance is **not** attached on this path (there is no local
OIDC provider); the script says so rather than implying the release is signed.

## Alternatives considered

| Option | Why not |
|--------|---------|
| Keep the human OTP gate | It is the path npm is closing in January 2027, and it puts the irreversible step outside CI. |
| Granular access token with bypass 2FA in a repo secret | Long-lived secret to leak and rotate; the same January 2027 restriction applies; strictly worse provenance story than OIDC. |
| `npm stage publish` only (maintainer approves each release with 2FA) | Materially safer, but the stage flow's `approve` is an interactive 2FA action, so it keeps a human in the loop on every release. Kept available at the npm configuration level so this can be tightened later without re-enrolling. |
| Publish from the maintainer's machine with a stored OTP source | Leaves publish outside CI and out of the audit trail; the emergency script covers the genuine fallback case without a stored code. |
| `workflow_dispatch` as the primary trigger | A manual trigger is not a release event and adds a second way to publish. It is kept only as a retry lever for a tag that already exists, guarded by a HEAD-equals-tag check. |

## Consequences

- **Publishing needs no human once the publisher is enrolled.** The release
  half of the cycle becomes tag → wait → verify, and the irreversible step
  leaves the laptop for CI with provenance attached.
- **The publisher is enrolled (2026-09-25), but the path has not run yet.**
  Enrollment needed 2FA enabled on the npm account first: npm requires
  interactive 2FA to modify package settings, and the account had it disabled,
  so both the website and `npm trust github` refused
  (`403 … Please enable 2fa for your account`). With 2FA on, the enrollment
  succeeded for `innocarpe/deepseek-build` + `publish-npm.yml`, allowing both
  `npm publish` and `npm stage publish`. No release has been cut since, so the
  next one is the first real exercise of this path — treat a first-run failure
  as an untested path, not as a regression.
- **The account is at `auth-and-writes`.** That is what makes a *local* publish
  need proof of presence; it does not touch the OIDC exchange, which is why
  the trusted publisher is the default path and a local publish is only the
  fallback. The registered second factor is a **security key**, so the proof is
  a browser approval rather than a typed code — see the emergency-path notes.
- **A failed OIDC publish must not be "fixed" by weakening the configuration.**
  The troubleshooting order is: workflow filename and case, `id-token: write`,
  GitHub-hosted runner, then `repository.url`.
- The trusted publisher is configured for `npm publish` **and**
  `npm stage publish`, so moving to staged publishing later is a workflow
  change, not a re-enrollment.
- ADR 0007's "inspect + local install smoke is agent work; registry publish is
  human work" split no longer holds for the registry half. The package DoD in
  ADR 0007 otherwise stands.

## References

- [docs.npmjs.com/trusted-publishers](https://docs.npmjs.com/trusted-publishers)
- [docs.npmjs.com/requiring-2fa-for-package-publishing-and-settings-modification](https://docs.npmjs.com/requiring-2fa-for-package-publishing-and-settings-modification)
- `.github/workflows/publish-npm.yml`
- [scripts/npm-emergency-publish.sh](../../scripts/npm-emergency-publish.sh)
- [ADR 0007](./0007-npm-packaging.md) · [ADR 0009](./0009-npm-prebuilt-binaries.md)
- [release-cycle.md](../contributing/release-cycle.md) · [skills/release](../../skills/release/SKILL.md)