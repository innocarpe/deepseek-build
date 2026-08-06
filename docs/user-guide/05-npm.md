# 05 — npm install

**Product version:** `0.7.0`+  
**Bins:** `deepseek-build` (primary) · `dsb` (alias)

## Package

Root `package.json`:

| Field | Value |
|-------|--------|
| `version` | Must equal workspace Cargo SemVer (**0.7.0**) |
| `bin.deepseek-build` | `npm/bin/deepseek-build.js` |
| `bin.dsb` | `npm/bin/dsb.js` |

## Install (local / global)

```bash
# Global from git checkout
git clone https://github.com/innocarpe/deepseek-build.git
cd deepseek-build
npm install -g .

# Requires Rust/cargo for postinstall native build (or run ./scripts/install.sh first)
deepseek-build --version
dsb --version
```

Skip postinstall native build:

```bash
DEEPSEEK_BUILD_SKIP_POSTINSTALL=1 npm install -g .
./scripts/install.sh
```

## How wrappers work

Node shims resolve the native binary from:

1. `DEEPSEEK_BUILD_BIN` (absolute path override)
2. `~/.deepseek-build/bin/{deepseek-build,dsb}`
3. `~/.cargo/bin/…`
4. `./target/release/…` (dev checkout)

## Publish (owner)

```bash
./scripts/check-semver.sh
npm publish --access public   # when ready; not required for dogfood-0x complete
```

Never put API keys in the package.
