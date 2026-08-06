# Changelog

## Unreleased

## 2.0.0-alpha.2 — 2026-08-06

### Added

- **No-args TTY** `dsb` / `deepseek-build` launches Grok-class full-screen agent (`deepseek-build-agent`)
- `agent` subcommand + `repl-legacy` thin REPL path
- Install builds/installs vendored `xai-grok-pager` as `deepseek-build-agent`
- Product `config.toml` seed: DeepSeek models, `api.deepseek.com`, chat_completions
- First-run setup before agent launch; credentials 0600 under `~/.deepseek-build/`
- Smoke note: `docs/product/evidence/W1_ENTRY_SMOKE.md`


## 2.0.0-alpha.1 — 2026-08-06

### Added

- **Grok Build vendor pin** under `third_party/grok-build/` (ADR-0008 strategy B)
- `SOURCE_REV` pin + Apache-2.0 / `THIRD-PARTY-NOTICES` retained in vendor tree
- Root `NOTICE` attribution for SpaceXAI Grok Build
- `scripts/build-grok-pager.sh` + [GROK_VENDOR.md](docs/architecture/GROK_VENDOR.md) (dual workspace + CI plan)
- Product SemVer band opens at **`2.0.0-alpha.1`** (not 2.0.0 cut)

### Notes

- Default `dsb` entry still product overlay until W1 entry/TUI stories land
- Full Grok `cargo check -p xai-grok-pager-bin` verified on vendor tree (local evidence)


## Prior unreleased notes (folded)


### Added

- **Welcome banner v2** — DeepSeek braille whale mark + boxed product card on interactive chat (`banner.rs`)
- REPL prompt `❯` uses DeepSeek blue accent when color is enabled
- Theme/docs: official `#4D6BFE` tokens documented for mark, box chrome, and prompt

### Documentation

- **Product replan for 2.0.0** — [REPLAN_2.0.md](docs/product/REPLAN_2.0.md): 1.x repositioned as **scaffold**; real product DoD is **`dsb` opens a Grok Build–class coding agent** on open-source Grok Build + DeepSeek/Deep Code/Reasonix overlays
- **One-plate ultragoal** `grokbase-2x` G001–G012: [GROKBASE_2X_GOALS.md](docs/product/GROKBASE_2X_GOALS.md), [ULTRAGOAL_BRIEF_2.0.md](docs/product/ULTRAGOAL_BRIEF_2.0.md), cold-start through cut
- Fixed PR units: [WAVE_2x_PR_DAG.md](docs/product/WAVE_2x_PR_DAG.md)
- README / SSOT / versioning / MASTER_PLAN / KNOWN_LIMITS honesty banners

## 1.1.0 — 2026-08-06

### Added

- **First-run setup onboarding** — `setup` / `auth login|status|logout`
- TTY `chat`/`run` auto-prompt for API key when missing; saves `credentials.json` (0600)
- Bare `deepseek-build` with no key starts setup on TTY
- User guide `00-setup.md`

### Notes

- Still the **1.x scaffold line** (thin agent UX). Product target is **2.0.0** — see REPLAN_2.0.

## 1.0.0 — 2026-08-06

### Release

- First **1.0.0** after Waves A–D **scaffold train**: dogfood core, DeepSeek-native surface, throughput **MVP**, RC harden/docs
- Product CI (later refined into split path-gated workflows)
- Full user-guide + known limits
- Dual CLI `deepseek-build` / `dsb`; npm package `@innocarpe/deepseek-build` (registry publish remains owner-gated)

### Notes

- **Repositioned (2026-08-06):** this release is a **contract/scaffold line**, not a Grok Build–class full agent product. See [REPLAN_2.0.md](docs/product/REPLAN_2.0.md) and [KNOWN_LIMITS.md](docs/product/KNOWN_LIMITS.md).

All notable product versions use full SemVer `MAJOR.MINOR.PATCH`.

## 0.16.0 — 2026-08-06

### Documentation

- Expanded user-guide: auth, chat/run, permissions, theme, tools
- Added `docs/product/KNOWN_LIMITS.md`
- This CHANGELOG

## 0.15.0 — 2026-08-06

### Added

- Product CI: `.github/workflows/ci.yml` (fmt, clippy, test, offline smoke)
- Harden path for Wave D RC

## 0.14.0 — 2026-08-06

### Added

- Spec 60 + G5: in-process subagents, worker cache law
- Tool `subagent` (explore | implement)

## 0.13.0 — 2026-08-06

### Added

- Background bash (`background: true` → `job_id`)
- Tool `bash_collect`

## 0.12.0 — 2026-08-06

### Added

- Spec 50 + G4: parallel read-only tools (serial mutating)

## 0.11.0 — 2026-08-06

### Added

- Specs 80/110; G6c/G6d green
- MCP catalog + schema fingerprint; tool `plan`

## 0.10.0 — 2026-08-06

### Added

- Skills product expand: opt-out frontmatter, `skills list`

## 0.9.0 — 2026-08-06

### Added

- TTY permission ask once/always grants
- DeepSeek blue theme v1 + DESIGN.md

## 0.8.0 — 2026-08-06

### Added

- Spec 40 core tools surface ready-for-impl + registry pins

## 0.7.0 – 0.7.1 — 2026-08-06

### Added

- npm package `@innocarpe/deepseek-build` dual bins
- Help SemVer example tracks package version

## 0.2.0 – 0.6.0 — 2026-08-06

Wave A dogfood core: install, tools daily, dogfood proof, sessions, skills index min + effort UX.

## 0.1.0 — 2026-08-06

Initial engine + tools core preview.
