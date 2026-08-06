# DeepSeek Build

**DeepSeek-native terminal coding agent** (`dsb`).

Combines three first-class references:

| Source | What we take |
|--------|----------------|
| **Grok Build** | Wall-clock speed: parallel tools, subagents, background tasks, worktree isolation, native runtime patterns |
| **[Reasonix](https://github.com/esengine/DeepSeek-Reasonix)** | Prefix-cache-first loop, Flash/Pro cost control, tool-call repair |
| **[Deep Code CLI](https://github.com/lessweb/deepcode-cli)** | Official DeepSeek-oriented surface: thinking, reasoning effort, Skills, MCP, permissions, plan mode |

**Not in v1 scope:** Gajae-code multi-stage planning/team harness (too slow for our north star).

> Status: **M1 runtime** — provider + stable prefix + repair/routing + thin CLI. Tools (snippet edit, shell) are M2+.

## Quickstart (M1)

### Requirements

- Rust **1.94+** (see `rust-toolchain.toml`)
- A DeepSeek API key

### Build

```bash
cargo build -p dsb-cli
cargo run -p dsb-cli -- --version
# → dsb 0.1.0
```

Release binary:

```bash
cargo build --release -p dsb-cli
./target/release/dsb --help
```

### Set API key

Either:

```bash
export DEEPSEEK_API_KEY=sk-...
```

Or create `~/.deepseek-build/credentials.json` (mode `0600` recommended):

```json
{ "api_key": "sk-..." }
```

**Never commit secrets.** Override home with `DEEPSEEK_BUILD_HOME` if needed.

### Run chat

One-shot (default model: **`deepseek-v4-flash`**):

```bash
cargo run -p dsb-cli -- run "Say hello in one short sentence."
```

One-shot **Pro** (user-visible model line):

```bash
cargo run -p dsb-cli -- run --pro "Outline a high-level architecture for a CLI agent in 3 bullets."
# stderr includes: [model=deepseek-v4-pro …] and [model_used=…]
```

Multi-turn REPL:

```bash
cargo run -p dsb-cli -- chat
# > hello
# > /pro design the system briefly
# > /preset flash
# > /quit
```

### Offline tests

```bash
cargo test --workspace
```

Covers specs **10** (prefix goldens), **15** (repair + pairing), **20** (routing), **30** (thinking request shape), plus mock HTTP SSE.

### Live smoke (when `DEEPSEEK_API_KEY` is set)

```bash
# Multi-turn on flash
cargo run -p dsb-cli -- run "Reply with exactly: pong"
cargo run -p dsb-cli -- chat   # type two turns, then /quit

# Pro escalate is visible
cargo run -p dsb-cli -- run --pro "Reply with exactly: pro-ok"
# Expect model=deepseek-v4-pro in stderr
```

Cache evidence: when the API returns cache hit/miss usage fields they are logged; otherwise dual-call substitute is available via provider API (`cache_evidence=substitute_dual_call`).

## Product docs

| Doc | Description |
|-----|-------------|
| [**Harness philosophy**](docs/architecture/HARNESS_PHILOSOPHY.md) | **Design spine** — Deep Code / Reasonix / Grok layers & conflict rules |
| [Gates](docs/GATES.md) | G0–G6 ledger (G0–G2 green; G3+ red until specs 45/90…) |
| [PRD v1](docs/product/PRD-v1.md) | Problem, goals, scope, journeys, success |
| [Milestones](docs/product/MILESTONES.md) | M0–M6 development plan |
| [Vision](docs/product/VISION.md) | North star and pillars |
| [Sources](docs/product/SOURCES.md) | Layered source ownership (L1/L2/L3) |
| [Non-goals](docs/product/NON_GOALS.md) | Explicit v1 exclusions |

## Crates

| Crate | Role |
|-------|------|
| `dsb-cli` | Binary `dsb` |
| `dsb-config` | Credentials / home |
| `dsb-provider-deepseek` | Chat Completions client (ADR 0005) |
| `dsb-context` | Stable prefix / epochs (spec 10) |
| `dsb-agent` | Repair, routing, turn loop (specs 15/20/30) |

See [`crates/README.md`](crates/README.md).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). **All meaningful work lands via PR.**

| Doc | Topic |
|-----|--------|
| [PR conventions](docs/contributing/pull-requests.md) | Units of work, titles, labels, merge |
| [PR body standard](docs/contributing/pr-body-standard.md) | Orca-level narrative bar |
| [PR examples](docs/contributing/examples.md) | Filled bodies for spec/feat/fix/docs |
| [Review checklist](docs/contributing/review-checklist.md) | Self-merge / reviewer gates |

PR title must be Conventional Commits; ready PRs need exactly one kind label.  
Enforced by **agent skill + review harness**, not by process-police CI.

## Repository map

```text
deepseek-build/
├── docs/                 # Product truth lives here first
├── crates/               # Rust workspace (dsb-*)
├── skills/               # Bundled Agent Skills (SKILL.md)
├── .deepseek-build/      # Project-local agent config surface
├── scripts/              # Dev / release helpers
└── third_party/          # Vendored / ported code notices
```

Start reading: **[docs/README.md](docs/README.md)**.

## References

- Grok Build — primary structural and orchestration reference (local sibling `OpenSources/grok-build` when developing offline)
- [Reasonix](https://github.com/esengine/DeepSeek-Reasonix) — DeepSeek cache-native agent
- [Deep Code CLI](https://github.com/lessweb/deepcode-cli) — official DeepSeek-oriented CLI surface

## License

[Apache License 2.0](LICENSE). See also [NOTICE](NOTICE).
