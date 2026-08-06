# 01 — Install

**Product version:** `0.16.0`+ (install path since **0.2.0**)  
**Commands:** `deepseek-build` (primary) · `dsb` (alias) — [ADR 0006](../adr/0006-cli-names-and-semver.md)

## Requirements

- Rust **1.94+** via [rustup](https://rustup.rs/)
- Git clone of [innocarpe/deepseek-build](https://github.com/innocarpe/deepseek-build)

npm package: **`@innocarpe/deepseek-build`** (CLI still `deepseek-build` / `dsb`) — see [05-npm.md](./05-npm.md).

## Install once

From the repository root:

```bash
./scripts/install.sh
```

Default install directory: `~/.deepseek-build/bin`.

If the script reports that directory is not on `PATH`:

```bash
export PATH="$HOME/.deepseek-build/bin:$PATH"
# optional permanent (zsh):
# echo 'export PATH="$HOME/.deepseek-build/bin:$PATH"' >> ~/.zshrc
```

### Other install targets

| Method | Bin directory |
|--------|----------------|
| `./scripts/install.sh` | `~/.deepseek-build/bin` |
| `./scripts/install.sh --cargo` | `~/.cargo/bin` (or `$CARGO_HOME/bin`) |
| `./scripts/install.sh --prefix DIR` | `DIR/bin` |
| `cargo install --path crates/dsb-cli --locked --force` | Cargo bin dir |

## Smoke

```bash
deepseek-build --version
# deepseek-build 0.16.0
dsb --version
# dsb 0.16.0
```

npm (optional): see [05-npm.md](./05-npm.md).

Both must print the **same** full SemVer triple.

## Config home

Product config lives under `~/.deepseek-build/` (credentials, future sessions).  
That path is **not** a CLI command name.

## Next

- Auth: set `DEEPSEEK_API_KEY` or `~/.deepseek-build/credentials.json` — root README
- Daily coding: `deepseek-build --dogfood chat` (workspace write + bash under policy)
- Chat only (read tools, fail-closed mutates): `deepseek-build chat`
