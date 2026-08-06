# DeepSeek Build

**DeepSeek-native terminal coding agent.**

**Commands:** `deepseek-build` (primary) · `dsb` (alias) — same program ([ADR 0006](docs/adr/0006-cli-names-and-semver.md)).  
**Versions:** always full SemVer `MAJOR.MINOR.PATCH` (e.g. `0.2.0`, never bare `0.2`) — [versioning.md](docs/contributing/versioning.md).

Combines three first-class references:

| Source | What we take |
|--------|----------------|
| **Grok Build** | Wall-clock speed: parallel tools, subagents, background tasks, worktree isolation, native runtime patterns |
| **[Reasonix](https://github.com/esengine/DeepSeek-Reasonix)** | Prefix-cache-first loop, Flash/Pro cost control, tool-call repair |
| **[Deep Code CLI](https://github.com/lessweb/deepcode-cli)** | Official DeepSeek-oriented surface: thinking, reasoning effort, Skills, MCP, permissions, plan mode |

**Not in v1 scope:** Gajae-code multi-stage planning/team harness (too slow for our north star).

> Status: see `Cargo.toml` SemVer. **Master plan (final goal + waves):** [docs/product/MASTER_PLAN.md](docs/product/MASTER_PLAN.md).  
> Wave A: [RELEASE_TRAIN_0x.md](docs/product/RELEASE_TRAIN_0x.md) · Chain: [ULTRAGOAL_CHAIN.md](docs/product/ULTRAGOAL_CHAIN.md) · Architecture: [SYSTEM_ARCHITECTURE.md](docs/architecture/SYSTEM_ARCHITECTURE.md).

## Install (PATH)

Requirements: **Rust 1.94+** (`rustup`), and a checkout of this repo.

### Recommended: install script

From the repo root (default prefix `~/.deepseek-build/bin`):

```bash
git clone https://github.com/innocarpe/deepseek-build.git
cd deepseek-build
./scripts/install.sh
# If PATH note printed:
export PATH="$HOME/.deepseek-build/bin:$PATH"
```

Alternative — install into Cargo’s bin dir (often already on PATH):

```bash
./scripts/install.sh --cargo
# → ~/.cargo/bin/deepseek-build and ~/.cargo/bin/dsb
```

Custom prefix:

```bash
./scripts/install.sh --prefix "$HOME/.local"
export PATH="$HOME/.local/bin:$PATH"
```

### Equivalent: `cargo install`

```bash
# From repo root after clone
cargo install --path crates/dsb-cli --locked --force
# bins: $CARGO_HOME/bin/deepseek-build and …/dsb (default CARGO_HOME=~/.cargo)
```

### Smoke (clean shell)

Open a **new** terminal (or source your shell config), then:

```bash
deepseek-build --version
# → deepseek-build 0.13.0
dsb --version
# → dsb 0.13.0
./scripts/check-semver.sh
# → check-semver: ok (0.13.0)
```

Both commands must report the **same** full SemVer.

### npm (both bins)

| | |
|--|--|
| **Package name** | `@innocarpe/deepseek-build` |
| **CLI commands** (unchanged) | `deepseek-build` · `dsb` |
| **version** | matches Cargo workspace SemVer |

```bash
# After registry publish (owner):
npm install -g @innocarpe/deepseek-build
# or:
npx @innocarpe/deepseek-build --version

# Commands are still the product names (not the scoped package id):
deepseek-build --version
dsb --version

# From a git checkout (dev / pre-publish):
npm install -g .
# postinstall builds native bins when cargo is available

# Version consistency (cargo ↔ npm):
node npm/scripts/check-version-match.js
./scripts/check-semver.sh
```

Native binary still needs Rust/cargo once (postinstall or `./scripts/install.sh`) until prebuilt binaries land.

Config directory (not a command name): `~/.deepseek-build/`.

## Auth

Either:

```bash
export DEEPSEEK_API_KEY=sk-...
```

Or create `~/.deepseek-build/credentials.json` (mode `0600` recommended):

```json
{ "api_key": "sk-..." }
```

**Never commit secrets.** Override home with `DEEPSEEK_BUILD_HOME` if needed.

## Run

One-shot (default model: **`deepseek-v4-flash`**):

```bash
deepseek-build run "Say hello in one short sentence."
# alias:
dsb run "Say hello in one short sentence."
```

### Model / thinking / effort

Each turn logs visibility like: `model=deepseek-v4-flash thinking=on effort=high`.

```bash
deepseek-build --effort max run "design the system"
deepseek-build --no-thinking run "quick yes/no"
deepseek-build --thinking --effort high chat
# In chat: /pro  /flash  /preset max|flash|balanced  /model
```

Skills: index of `skills/*/SKILL.md` (and `~/.deepseek-build/skills/`) goes into the **stable** prefix; full body loads only via the `skill` tool (no prefix thrash).

### Sessions (persist / resume)

Transcripts live under `~/.deepseek-build/sessions/<id>.jsonl`. On resume, unpaired tool calls are repaired (spec 15).

```bash
# New or resume named session
deepseek-build --session my-work --dogfood chat
dsb --session my-work run "continue from last turn"

# Manage
deepseek-build sessions list
deepseek-build sessions show my-work
deepseek-build sessions delete my-work
```

One-shot **Pro** (user-visible model line):

```bash
deepseek-build run --pro "Outline a high-level architecture for a CLI agent in 3 bullets."
# stderr includes: [model=deepseek-v4-pro …] and [model_used=…]
```

Multi-turn REPL:

```bash
deepseek-build chat
# > hello
# > /pro design the system briefly
# > /preset flash
# > /quit
```

### Dogfood profile (trusted local coding)

For daily use on a repo you trust, prefer **one flag**:

```bash
deepseek-build --dogfood chat
# alias:
dsb --dogfood chat
```

`--dogfood` enables:

| Capability | Behavior |
|------------|----------|
| Workspace write/edit/create/delete | Allowed **inside** the workspace |
| Bash execution | Real execute (not dry-run-only) under classifier + policy |
| Out-of-workspace write/delete | **Still denied** (fail-closed) |

Or combine finer flags:

```bash
# Workspace write only (bash still dry-run unless also --bash-execute)
deepseek-build --allow-workspace-write chat

# Execute bash under policy (writes still need --allow-workspace-write or --dogfood)
deepseek-build --bash-execute chat
```

Built-in tools (model-visible): `read`, `edit`, `write`, **`grep`**, `bash`.

## Develop from source

If you are hacking on the crates (not needed for normal use):

```bash
cargo build -p dsb-cli
cargo run -p dsb-cli --bin deepseek-build -- --version
cargo run -p dsb-cli --bin dsb -- --version
cargo build --release -p dsb-cli
./target/release/deepseek-build --help
./target/release/dsb --help
```

### Offline tests

```bash
cargo test --workspace
```

Covers specs **10** (prefix goldens), **15** (repair + pairing), **20** (routing), **30** (thinking request shape), **45/90** tools core, plus mock HTTP SSE.

### Live smoke (when `DEEPSEEK_API_KEY` is set)

```bash
deepseek-build run "Reply with exactly: pong"
dsb chat   # type two turns, then /quit

deepseek-build run --pro "Reply with exactly: pro-ok"
# Expect model=deepseek-v4-pro in stderr
```

Cache evidence: when the API returns cache hit/miss usage fields they are logged; otherwise dual-call substitute is available via provider API (`cache_evidence=substitute_dual_call`).

## Product docs

| Doc | Description |
|-----|-------------|
| [**Release train 0.x**](docs/product/RELEASE_TRAIN_0x.md) | SemVer train + dogfood-usable definition |
| [**Harness philosophy**](docs/architecture/HARNESS_PHILOSOPHY.md) | **Design spine** — Deep Code / Reasonix / Grok layers & conflict rules |
| [Gates](docs/GATES.md) | G0–G6 ledger (G0–G3 green; G4–G6 red until specs) |
| [PRD v1](docs/product/PRD-v1.md) | Problem, goals, scope, journeys, success |
| [Milestones](docs/product/MILESTONES.md) | M0–M6 development plan |
| [Vision](docs/product/VISION.md) | North star and pillars |
| [Sources](docs/product/SOURCES.md) | Layered source ownership (L1/L2/L3) |
| [Non-goals](docs/product/NON_GOALS.md) | Explicit v1 exclusions |

## Crates

| Crate | Role |
|-------|------|
| `dsb-cli` | Binaries `deepseek-build` + `dsb` |
| `dsb-config` | Credentials / home |
| `dsb-provider-deepseek` | Chat Completions client (ADR 0005) |
| `dsb-context` | Stable prefix / epochs (spec 10) |
| `dsb-agent` | Repair, routing, turn loop (specs 15/20/30) |
| `dsb-tools` | Snippets, permissions, read/edit/write/bash (45/90) |

See [`crates/README.md`](crates/README.md).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). **All meaningful work lands via PR.**

| Doc | Topic |
|-----|--------|
| [PR conventions](docs/contributing/pull-requests.md) | Units of work, titles, labels, merge |
| [PR body standard](docs/contributing/pr-body-standard.md) | Orca-level narrative bar |
| [PR examples](docs/contributing/examples.md) | Filled bodies for spec/feat/fix/docs |
| [Review checklist](docs/contributing/review-checklist.md) | Self-merge / reviewer gates |
| [Versioning](docs/contributing/versioning.md) | Full SemVer only |

PR title must be Conventional Commits; ready PRs need exactly one kind label.  
Enforced by **agent skill + review harness**, not by process-police CI.

## Repository map

```text
deepseek-build/
├── docs/                 # Product truth lives here first
├── crates/               # Rust workspace (dsb-*)
├── skills/               # Bundled Agent Skills (SKILL.md)
├── .deepseek-build/      # Project-local agent config surface
├── scripts/              # install.sh, check-semver, helpers
└── third_party/          # Vendored / ported code notices
```

Start reading: **[docs/README.md](docs/README.md)**.

## References

- Grok Build — primary structural and orchestration reference (local sibling `OpenSources/grok-build` when developing offline)
- [Reasonix](https://github.com/esengine/DeepSeek-Reasonix) — DeepSeek cache-native agent
- [Deep Code CLI](https://github.com/lessweb/deepcode-cli) — official DeepSeek-oriented CLI surface

## License

[Apache License 2.0](LICENSE). See also [NOTICE](NOTICE).
