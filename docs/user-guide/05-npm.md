# 05 — npm install

**Product version:** `0.16.0`+  

| Surface | Value |
|---------|--------|
| **npm package** | `@innocarpe/deepseek-build` |
| **CLI commands** | `deepseek-build` (primary) · `dsb` (alias) — [ADR 0006](../adr/0006-cli-names-and-semver.md) |

Package name and command names are **different on purpose** (same pattern as many scoped CLIs).

## Package layout

Root `package.json`:

| Field | Value |
|-------|--------|
| `name` | `@innocarpe/deepseek-build` |
| `version` | Must equal workspace Cargo SemVer |
| `bin.deepseek-build` | `npm/bin/deepseek-build.js` |
| `bin.dsb` | `npm/bin/dsb.js` |
| `publishConfig.access` | `public` (scoped package is public) |

## Install (after registry publish)

```bash
npm install -g @innocarpe/deepseek-build

deepseek-build --version
dsb --version
```

One-shot:

```bash
npx @innocarpe/deepseek-build --version
```

**Requirements / expectations**

- **Rust/cargo** must be installed for `postinstall` (or run `./scripts/install.sh` yourself). Without cargo, npm package installs but CLI wrappers print how to build natives.
- First install can take **tens of seconds** (native compile). Later installs reuse `~/.deepseek-build/bin` when present.
- Node **≥ 18**.

## Install from git checkout (dev)

```bash
git clone https://github.com/innocarpe/deepseek-build.git
cd deepseek-build
npm install -g .

deepseek-build --version
dsb --version
```

Skip postinstall native build:

```bash
DEEPSEEK_BUILD_SKIP_POSTINSTALL=1 npm install -g .
./scripts/install.sh
```

## How wrappers work

Node shims resolve the **native** binary from:

1. `DEEPSEEK_BUILD_BIN` (absolute path override)
2. `~/.deepseek-build/bin/{deepseek-build,dsb}`
3. `~/.cargo/bin/…`
4. `./target/release/…` (dev checkout)

## Publish (owner)

Prerequisites:

1. npm account with rights to the **`@innocarpe`** org (or create the org on npmjs.com)
2. `npm login` / `npm whoami` succeeds

```bash
cd /path/to/deepseek-build
git checkout main && git pull
./scripts/check-semver.sh
node npm/scripts/check-version-match.js
npm pack --dry-run          # review tarball contents
npm publish --access public # publishConfig also sets access=public
```

Verify:

```bash
npm view @innocarpe/deepseek-build version
npm install -g @innocarpe/deepseek-build
deepseek-build --version
dsb --version
```

Never put API keys in the package.

## Name policy

| Kind | Name | Change often? |
|------|------|----------------|
| npm package | `@innocarpe/deepseek-build` | Rare (ownership / branding) |
| CLI | `deepseek-build`, `dsb` | No — product identity (ADR 0006) |
| Config dir | `~/.deepseek-build/` | No |
