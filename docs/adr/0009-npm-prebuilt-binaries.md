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

### Source-checkout amendment (2026-09-26)

`npm install` in a git checkout must not download or compile the product.
The script's package root is a checkout when **all three** are present:

| Signal | Checkout | Packed install (`npm pack` / registry) |
|--------|----------|----------------------------------------|
| `.git` | directory (clone) or a file whose first line is `gitdir:` (worktree) | absent — npm never packs `.git` |
| `Cargo.toml` | file at the package root | absent — not in `package.json` `"files"` |
| `scripts/install.sh` | file | absent — `"files"` ships `npm/scripts/**` only |

Missing any one means the tree is **not** a checkout and `postinstall` runs.
The check does not walk up to a parent `.git`. A GitHub zipball (source
files, no `.git`) still installs.

The three files alone are not enough to skip. Measured on npm 12.1.0,
`npm install -g .` symlinks the checkout into the prefix and runs
`postinstall` **in the checkout** with `npm_config_global=true`. That smoke
path (ADR 0007) must still install. Skip only when the tree is a checkout
and npm is not doing a global install, and either:

- `INIT_CWD` is the checkout (`npm install` run there), or
- `npm_config_prefix` is the checkout (`npm install --prefix <checkout>`), or
- neither variable is set (`node npm/scripts/postinstall.js`)

`npm install <checkout>` from another directory leaves `INIT_CWD` elsewhere
and the prefix at the user's prefix, so it still installs.
`DEEPSEEK_BUILD_ALLOW_SOURCE_BUILD` does not override the skip. Build a
checkout with `./scripts/install.sh`.

The package root is the script directory (`npm/scripts/../..`), not
`process.cwd()`.

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
- **The ordering rule is enforced, not just documented:** `publish-npm.yml`
  waits for the asset, executes the packaged agent to confirm it reports the
  release version, and refuses to publish otherwise.
- **npm publish is CI-published** over OIDC trusted publishing
  ([ADR 0012](./0012-npm-trusted-publishing.md), amending ADR 0007's human gate).

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
