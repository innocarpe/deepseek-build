# 05 — npm install

**Product version:** `4.0.1`+ (prebuilt path — ADR 0009)

| Surface | Value |
|---------|--------|
| **npm package** | `@innocarpe/deepseek-build` |
| **CLI commands** | `deepseek-build` (primary) · `dsb` (alias) — [ADR 0006](../adr/0006-cli-names-and-semver.md) |

## Install (normal users)

npm **12.0.0+** blocks dependency install scripts unless you opt in. The
command below is the one that installs the agent. npm 11 and older accept it
too; the flag does not change their behavior for this package.

```bash
npm install -g --allow-scripts=@innocarpe/deepseek-build @innocarpe/deepseek-build

deepseek-build --version
dsb --version
dsb setup    # API key once
dsb          # full-screen DeepSeek TUI
```

Allow the package once for every later global install, then the plain command works:

```bash
npm config set allow-scripts=@innocarpe/deepseek-build --location=user
npm install -g @innocarpe/deepseek-build
```

**Expectations (same class as Claude Code / Codex / Grok Build npm):**

- **No Rust required** for registry install.  
- `postinstall` downloads a **platform tarball** from GitHub Releases for this SemVer and installs into `~/.deepseek-build/bin/`.  
- Should finish in **seconds** (network + extract), not tens of minutes.  
- Node **≥ 18**.  
- Platform: `darwin-arm64` (Apple Silicon macOS only).
- npm **12.0.0+** does not run this package's `postinstall` unless `allowScripts` covers `@innocarpe/deepseek-build`. A denied script still ends with `added 1 package` and does not install the agent. `npm install -g --allow-scripts=@innocarpe/deepseek-build` with no package spec fails (`ENOENT package.json`). `npm rebuild -g @innocarpe/deepseek-build` is denied the same way; pass `--allow-scripts=@innocarpe/deepseek-build` and repeat the package name.

If `dsb` is not on PATH after install:

```bash
export PATH="$HOME/.deepseek-build/bin:$PATH"
# zsh permanent:
# echo 'export PATH="$HOME/.deepseek-build/bin:$PATH"' >> ~/.zshrc
```

## npm 12 (install scripts denied by default)

npm 12.0.0 and newer do not run dependency install scripts unless the
installer opts in (`allowScripts`). This package cannot opt itself in: the
setting is read only from the installer (CLI flag, user `.npmrc`, or the
installing project's `package.json`, which `npm i -g` skips). The trusted
name comes from the lockfile URL, not from the tarball manifest.

The published tarball does not contain the agent. `postinstall` downloads it
into `~/.deepseek-build/bin/` and mirrors it at `npm/native-bin/`. A blocked
script leaves the npm shims in place and the agent missing. If an older agent
is already in `~/.deepseek-build/bin`, the new shim runs that binary and sets
`DEEPSEEK_BUILD_VERSION` from the package, so the old agent can print the new
version. With no binary at all, `deepseek-build` / `dsb` exits 127 and prints
the commands below.

Allow it per install:

```bash
npm install -g --allow-scripts=@innocarpe/deepseek-build @innocarpe/deepseek-build
```

Or once, for every later global install:

```bash
npm config set allow-scripts=@innocarpe/deepseek-build --location=user
```

If a script did run and the download failed, retry with the same opt-in.
Plain `npm rebuild -g @innocarpe/deepseek-build` stays blocked on npm 12:

```bash
npm rebuild -g --allow-scripts=@innocarpe/deepseek-build @innocarpe/deepseek-build
```

npm 11 and older run `postinstall` with or without the flag.

## Skip / source (dev only)

On npm 12 the script never starts unless `--allow-scripts` is set, so these
variables are only read after that opt-in. npm 11 and older run the script
either way.

```bash
# skip postinstall download
DEEPSEEK_BUILD_SKIP_POSTINSTALL=1 npm install -g --allow-scripts=@innocarpe/deepseek-build @innocarpe/deepseek-build

# allow slow cargo build if prebuilt asset is missing
DEEPSEEK_BUILD_ALLOW_SOURCE_BUILD=1 npm install -g --allow-scripts=@innocarpe/deepseek-build @innocarpe/deepseek-build

# from git checkout
git clone https://github.com/innocarpe/deepseek-build.git
cd deepseek-build
./scripts/install.sh
```

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
