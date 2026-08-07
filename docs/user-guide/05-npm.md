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
