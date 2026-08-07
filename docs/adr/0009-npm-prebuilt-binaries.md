# ADR 0009 — npm install via prebuilt binaries (fast path)

- **Status:** Accepted  
- **Date:** 2026-08-07  
- **Amends:** [ADR 0007](./0007-npm-packaging.md) distribution strategy for product installs  
- **Gate:** `4.0.1` install DX

## Context

ADR 0007 chose **source-assisted** install (postinstall `cargo build` of the vendored agent). That made `npm i -g @innocarpe/deepseek-build` take **tens of minutes** and require Rust/protoc — unacceptable next to Claude Code / Codex / Grok Build, where install is seconds and the CLI works immediately.

Product contract remains: **`dsb` opens the DeepSeek full-screen TUI**. How binaries arrive must not force every user to compile Grok.

## Decision

### Default install path (registry users)

1. npm package is a **thin Node wrapper** (bins + install scripts). It does **not** ship `third_party/grok-build` or require compile.  
2. `postinstall` **downloads a platform tarball** from GitHub Releases for the matching SemVer tag:  
   `https://github.com/innocarpe/deepseek-build/releases/download/v{VERSION}/deepseek-build-{VERSION}-{platform}.tar.gz`  
3. Extract into `~/.deepseek-build/bin/`:  
   - `deepseek-build`  
   - `dsb`  
   - `deepseek-build-agent`  
4. **No Cargo / protoc** on the default path.  
5. Skip flags:  
   - `DEEPSEEK_BUILD_SKIP_POSTINSTALL=1` — skip download  
   - `DEEPSEEK_BUILD_ALLOW_SOURCE_BUILD=1` — optional fallback to local `scripts/install.sh` when download fails **and** Rust is available (dev / missing platform)

### Platforms (release assets)

| `platform` id | Target |
|---------------|--------|
| `darwin-arm64` | Apple Silicon macOS (current supported target) |

The other platform mappings are deferred candidates, not current release
targets. Windows is not a first-class prebuilt target yet; document the
source/dev path.

### Current scope amendment (2026-08-07)

The immediate product contract is **Apple Silicon macOS only**. The npm
platform resolver, release packager, release wait loop, and tag workflow all
fail closed or build only for `darwin-arm64`. Re-enabling another platform
requires an explicit product decision and a matching release-harness change;
it must not happen merely because a runner is available.

### Release engineering

- Tag `vMAJOR.MINOR.PATCH` **must** attach the `darwin-arm64` tarball before or with npm publish.
- Script: `scripts/package-release-binaries.sh` builds/packages local or CI artifacts.  
- CI workflow (recommended): on tag push, build the single target → upload the asset.
- **npm publish remains human-gated** (ADR 0007).

### Package identity (unchanged)

- Name `@innocarpe/deepseek-build`, dual CLI names, SemVer = Cargo (ADR 0006/0007).

## Consequences

- `npm i -g` is **seconds** when the release asset exists (download + extract).  
- Missing platform asset → clear error + optional source fallback.  
- npm tarball size drops dramatically (no vendor tree).  
- Maintainers must attach prebuilts on every version tag that is published to npm.

## Non-goals

- Signed binaries / notarization (follow-up)  
- musl / static Linux variants (follow-up)  
- Shipping secrets in the package  

## References

- [05-npm.md](../user-guide/05-npm.md)  
- [npm/scripts/postinstall.js](../../npm/scripts/postinstall.js)  
- [scripts/package-release-binaries.sh](../../scripts/package-release-binaries.sh)  
