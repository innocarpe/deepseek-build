# 05 — npm install

**Product version:** `4.0.1`+ (prebuilt path — ADR 0009)

| Surface | Value |
|---------|--------|
| **npm package** | `@innocarpe/deepseek-build` |
| **CLI commands** | `deepseek-build` (primary) · `dsb` (alias) — [ADR 0006](../adr/0006-cli-names-and-semver.md) |

## Install (normal users)

```bash
npm install -g @innocarpe/deepseek-build

deepseek-build --version
dsb --version
dsb setup    # API key once
dsb          # full-screen DeepSeek TUI
```

**Expectations (same class as Claude Code / Codex / Grok Build npm):**

- **No Rust required** for registry install.  
- `postinstall` downloads a **platform tarball** from GitHub Releases for this SemVer and installs into `~/.deepseek-build/bin/`.  
- Should finish in **seconds** (network + extract), not tens of minutes.  
- Node **≥ 18**.  
- Platform: `darwin-arm64` (Apple Silicon macOS only).

If `dsb` is not on PATH after install:

```bash
export PATH="$HOME/.deepseek-build/bin:$PATH"
# zsh permanent:
# echo 'export PATH="$HOME/.deepseek-build/bin:$PATH"' >> ~/.zshrc
```

## Skip / source (dev only)

```bash
# skip postinstall download
DEEPSEEK_BUILD_SKIP_POSTINSTALL=1 npm install -g @innocarpe/deepseek-build

# allow slow cargo build if prebuilt asset is missing
DEEPSEEK_BUILD_ALLOW_SOURCE_BUILD=1 npm install -g @innocarpe/deepseek-build

# from git checkout
git clone https://github.com/innocarpe/deepseek-build.git
cd deepseek-build
./scripts/install.sh
```

`npm install` with no arguments inside that checkout does **not** install
`deepseek-build` or `dsb`. `postinstall` sees the git checkout and returns
without downloading a prebuilt and without compiling. Use `./scripts/install.sh` there.

These are not that skip. They still run the product install when npm itself
runs `postinstall`:

- `npm install -g @innocarpe/deepseek-build`
- `npm install -g ./innocarpe-deepseek-build-<version>.tgz`
- `npm install -g .` from the checkout (npm marks this global, even though the script runs in the checkout)

`DEEPSEEK_BUILD_ALLOW_SOURCE_BUILD` applies when a packed install's download
fails. It does not make `npm install` inside the checkout compile.

## What `--version` reports

`deepseek-build --version` and `dsb --version` print the **native CLI** the
shim executes (`dsb 6.0.0` — the installed CLI's own SemVer).
`deepseek-build-agent --version` prints the **agent**
(`deepseek-build 6.0.0 (<commit>)`). Neither line is rewritten from the npm
package's `package.json`, and a release SemVer does not wear `[alpha]`.
`[alpha]` / `[stable]` remain only for a pre-release build compared with
the updater's cached pointer.

`npm i -g @innocarpe/deepseek-build@X` installs agent `X` (ADR 0009). If the
agent already on disk is **newer** than `X`, postinstall leaves it in place
and the install fails, so a stale package cannot roll the agent backwards.
Set `DEEPSEEK_BUILD_ALLOW_DOWNGRADE=1` to install that older package on
purpose. When the package and the agent disagree, the shim prints a warning
on stderr before the command runs.

## How wrappers work

Node shims (`npm/bin/*.js`) resolve natives from:

1. `DEEPSEEK_BUILD_BIN`  
2. `~/.deepseek-build/bin/{deepseek-build,dsb,deepseek-build-agent}`  
3. `~/.cargo/bin/…`  
4. package `npm/native-bin/`  

## Maintainer: release prebuilts

```bash
# After version bump + building natives for this machine:
./scripts/package-release-binaries.sh --upload
# → dist/deepseek-build-{VERSION}-{platform}.tar.gz on GitHub release v{VERSION}

npm publish --access public   # human OTP (ADR 0007)
```

CI attaches the `darwin-arm64` tarball on each version tag.

## Publish (owner)

```bash
./scripts/check-semver.sh
npm run version-check
npm whoami
npm pack   # should be small (no third_party vendor tree)
npm publish --access public
npm view @innocarpe/deepseek-build version
```
