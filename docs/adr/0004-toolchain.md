# ADR 0004 — Toolchain and runtime layout

- **Status:** Accepted  
- **Date:** 2026-08-06  
- **Gate:** G1  

## Context

M1 needs a real package layout. Options considered:

| Option | Pros | Cons |
|--------|------|------|
| **Rust** | Matches Grok L3 speed goals; strong TUI ecosystem (ratatui); single static-ish binary | Longer ramp |
| Go | Reasonix-style single binary; fast compile | Weaker fit to Grok-derived orchestration patterns we study |
| TypeScript/Bun | Fast iterate; Deep Code is Node | Harder to hit Grok-class tool wall-clock; less ideal for “native speed” north star |

## Decision

1. **Language:** **Rust** (edition 2024 toolchain pinned later in `rust-toolchain.toml` at first workspace commit).  
2. **Binary / CLI names:** **`deepseek-build`** (primary) and **`dsb`** (alias) — see **[ADR 0006](./0006-cli-names-and-semver.md)** (supersedes the earlier “dsb-only” wording). Package crate names still use `dsb-*` / workspace `deepseek-build`. Product version is always full **SemVer** `MAJOR.MINOR.PATCH` ([versioning.md](../contributing/versioning.md)).  
3. **Layout:** Cargo workspace under `crates/` (Grok-inspired modularity; **not** a Grok hard-fork).  
4. **User config root:**  
   - Unix: `~/.deepseek-build/` (override: `DEEPSEEK_BUILD_HOME`)  
   - Windows: `%APPDATA%\deepseek-build\`  
5. **Project surface:** `.deepseek-build/` in repo (skills, agents, workflows later).  
6. **Secrets:** API key from env `DEEPSEEK_API_KEY` **or** `~/.deepseek-build/credentials.json` (mode `0600`, never commit). No keys in project tree.  
7. **Supported hosts (v1 target):** macOS (arm64/x64), Linux (x64/arm64). Windows best-effort later.  
8. **Async runtime:** `tokio`. HTTP: `reqwest` (or equivalent).  

Illustrative crate map (names may refine without new ADR if boundaries hold):

| Crate | Role |
|-------|------|
| `dsb-cli` / binary | Entry + TUI/headless composition |
| `dsb-provider-deepseek` | API client (ADR 0005) |
| `dsb-agent` | Turn loop |
| `dsb-context` | Prefix builder / cache epochs (spec 10) |
| `dsb-tools` | Tool runtime (M2+) |
| `dsb-config` | Config load |

## Consequences

- First runtime PR may introduce `Cargo.toml` workspace + empty/minimal crates **only after this ADR is merged** (G1 green).  
- Node/Go ports are non-goals for v1.  
- Dual CLI names and SemVer rules: ADR 0006 (not a free rename without ADR).

## References

- [HARNESS_PHILOSOPHY](../architecture/HARNESS_PHILOSOPHY.md) L3 speed + L1/L2 contracts  
- [GATES.md](../GATES.md)  
