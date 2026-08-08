<div align="center">

**[English](README.md)** · [简体中文](README.zh-CN.md) · [日本語](README.ja-JP.md) · [한국어](README.ko-KR.md)

<!-- Temporary hero source: deepseek-ai/DeepSeek-V2 figures/logo.svg, as used by DeepSeek-V3. -->
<a href="https://github.com/deepseek-ai/DeepSeek-V3">
  <img src="assets/deepseek-logo.svg" width="60%" alt="DeepSeek logo">
</a>

<h1>DeepSeek Build</h1>

<p><strong>DeepSeek-native coding. Grok-class execution.</strong></p>

<p>
  A full-screen terminal coding agent with safe edits, cache-aware sessions,
  and parallel execution built around DeepSeek models.
</p>

<p>
  <a href="https://github.com/innocarpe/deepseek-build/releases"><img alt="GitHub release" src="https://img.shields.io/github/v/release/innocarpe/deepseek-build?style=flat-square&label=release"></a>
  <a href="https://www.npmjs.com/package/@innocarpe/deepseek-build"><img alt="npm version" src="https://img.shields.io/npm/v/%40innocarpe%2Fdeepseek-build?style=flat-square&label=npm"></a>
  <a href="LICENSE"><img alt="Apache 2.0 license" src="https://img.shields.io/badge/license-Apache--2.0-blue?style=flat-square"></a>
</p>

<p>
  <a href="#quick-start">Quick start</a> ·
  <a href="#why-deepseek-build">Why DeepSeek Build</a> ·
  <a href="#how-it-works">How it works</a> ·
  <a href="#documentation">Documentation</a> ·
  <a href="#contributing">Contributing</a>
</p>

</div>

<p align="center">
  <img src="assets/deepseek-build-welcome.png" alt="DeepSeek Build welcome screen — the full-screen DeepSeek agent TUI opened by dsb" width="85%">
</p>

> [!NOTE]
> **Product status:** the `5.x` line is the owner-bar-complete product.
> **`5.5.0`** is the vision-complete freeze — Deep Code (L1), Reasonix (L2),
> and Grok throughput (L3) closed on Path A. The
> [`5.0.0` cut](docs/product/evidence/CUT_5_0_0_2026-08-07.md) passed the Path A
> ledger and independent reviews. **npm and GitHub Latest ship `5.5.0`.**
> Earlier `3.x` and `4.x` tags are
> documented as partial attempts in the [version history](docs/product/versions/README.md).

## Quick start

Install from npm, add your DeepSeek API key, and open the TUI:

```bash
npm install -g @innocarpe/deepseek-build
deepseek-build setup
deepseek-build
```

The registry install requires Node.js 18 or newer and uses a prebuilt binary
when a matching release asset is available. It does not require Rust on that
path; see the [npm installation guide](docs/user-guide/05-npm.md) for platform
and source-fallback details.

`deepseek-build` is the primary command. `dsb` is the fully supported short
alias with the same behavior and full Semantic Version:

```bash
deepseek-build --version
dsb --version
```

If the installer reports that the product bin directory is not on `PATH`, add
it before launching:

```bash
export PATH="$HOME/.deepseek-build/bin:$PATH"
```

## Why DeepSeek Build

| Capability | What it means |
| --- | --- |
| **DeepSeek-native** | DeepSeek API defaults, Flash/Pro routing, reasoning effort, and a DeepSeek-branded TUI. |
| **Safe edits** | Version-bound snippet editing and fail-closed workspace permissions instead of silent whole-file replacement. |
| **Long-session economics** | A stable prompt prefix, lazy skill loading, and tool-call repair keep resumed work coherent and cache-aware. |
| **Wall-clock throughput** | Parallel tools, background shell jobs, subagents, and opt-in worktrees run beneath the safety and cache layers. |
| **Durable sessions** | Resume the most recent full-screen session or address a saved session directly. |

The result is a coding agent that keeps the speed of a Grok-derived execution
engine while making DeepSeek-specific cost, edit, and permission rules part of
the product path.

## Everyday use

```bash
# Open the full-screen TUI
deepseek-build

# Resume the most recent TUI session
deepseek-build --resume

# Run one non-interactive turn
deepseek-build run "Explain the architecture of this repository."

# Use the trusted local coding profile
deepseek-build --dogfood
```

`--dogfood` allows writes inside the current workspace and enables shell
execution under policy. Writes and deletes outside the workspace remain denied.

For the short command, replace `deepseek-build` with `dsb` in any example.

## Authentication and configuration

Interactive setup stores the API key in
`~/.deepseek-build/credentials.json` with mode `0600`:

```bash
deepseek-build setup
deepseek-build auth status
deepseek-build auth logout
```

For CI or another non-interactive environment, set `DEEPSEEK_API_KEY`; the
environment variable takes precedence over the credentials file. Product
configuration, credentials, sessions, and user skills live under
`~/.deepseek-build/` by default.

## Build from source

Source installation is intended for contributors and unsupported release
platforms. It requires Rust 1.94 or newer plus `protoc` or DotSlash; the first
agent build can take several minutes.

```bash
git clone https://github.com/innocarpe/deepseek-build.git
cd deepseek-build
./scripts/install.sh

deepseek-build --version
dsb --version
```

See the [installation guide](docs/user-guide/01-install.md) for Cargo and custom
prefix options.

## How it works

```text
deepseek-build | dsb
        │
        ▼
product launcher ── auth · config · model routing
        │
        ▼
deepseek-build-agent ── full-screen TUI · tools · sessions
        │
        ▼
DeepSeek API
```

Three layers have explicit ownership. Higher-throughput machinery cannot bypass
the edit, permission, or cache contracts beneath it.

| Layer | Source | Owns |
| --- | --- | --- |
| **L1** | [Deep Code CLI](https://github.com/lessweb/deepcode-cli) | Snippet-safe edits, skills as context, and side-effect permissions. |
| **L2** | [Reasonix](https://github.com/esengine/DeepSeek-Reasonix) | Stable-prefix economics, Flash/Pro behavior, and tool-call repair. |
| **L3** | [Grok Build](https://github.com/xai-org/grok-build) | The base runtime, TUI, parallel tools, subagents, background work, and worktrees. |

The normative conflict rules live in the
[harness philosophy](docs/architecture/HARNESS_PHILOSOPHY.md), with the complete
system map in [SYSTEM_ARCHITECTURE.md](docs/architecture/SYSTEM_ARCHITECTURE.md).

## Documentation

| Start here | Use it for |
| --- | --- |
| [User guide](docs/user-guide/README.md) | Installation, setup, daily use, and the complete feature index. |
| [First-run setup](docs/user-guide/00-setup.md) | API keys, credential precedence, and headless setup. |
| [Sessions](docs/user-guide/03-sessions.md) | Full-screen resume and line-mode session storage. |
| [Permissions](docs/user-guide/08-permissions.md) | Interactive asks, headless denial, and workspace boundaries. |
| [Subagents](docs/user-guide/11-subagents.md) · [background tasks](docs/user-guide/12-background-tasks.md) · [worktrees](docs/user-guide/13-worktrees.md) | L3 execution surfaces. |
| [Known limitations](docs/product/KNOWN_LIMITS.md) | Current packaging, live-smoke, and platform boundaries. |
| [Product SSOT](docs/product/SSOT.md) | Which artifact wins when product documents disagree. |

## Development

```bash
cargo build -p dsb-cli
cargo test --workspace
./scripts/check-semver.sh
./scripts/test-owner-bar.sh
```

The root Rust workspace covers the product crates. Avoid vendor-full Cargo runs
for everyday checks; the owner-bar scripts use the bounded product path.

For the crate map, see [crates/README.md](crates/README.md). For the repository
map and documentation ownership, start at [docs/README.md](docs/README.md).

## Contributing

Read [CONTRIBUTING.md](CONTRIBUTING.md) before making changes. All meaningful
work lands through a focused PR with an atomic Conventional Commit, an existing
kind label, honest test evidence, and the review narrative defined in the
[PR authoring guide](docs/contributing/pr-body-standard.md).

## License

DeepSeek Build is available under the [Apache License 2.0](LICENSE). Vendored
and third-party code retains its original licensing; see [NOTICE](NOTICE).
