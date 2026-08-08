# Changelog

## Unreleased

- Fix the TUI update banner advertising Grok Build's version (e.g. `v1.0.0`)
  as an available update for DeepSeek Build (`5.5.0`). Update checks now
  always consult the product npm feed (`@innocarpe/deepseek-build`), never the
  upstream Grok x.ai channel pointers, npm installs are classified as npm, and
  the product never auto-downgrades for any installer classification.

## 5.5.0 — 2026-08-08

- Vision-complete freeze cut merged on `main`: V1 Deep Code + V2 Reasonix + V3
  Grok throughput + V4 product finish criteria closed on public Path A evidence
- Closes V3-60-3 residual: parent `snippet_id` mint → implement-class worker
  mutates same path → parent pre-mutation edit rejected (`snippet_stale`)
- Published to npm (`5.5.0`) and GitHub Releases (`v5.5.0`) on 2026-08-08;
  dual adversarial review is external

## 5.4.0 — 2026-08-08

- L3 Path A R0A train cut (multi-tool/bg, subagent/worker-cache, worktree dogfood) + optional live L3 matrix
- **Not published:** in-repo cut merged on `main` (PR #145) only — npm and GitHub Releases skipped `5.4.0` (published `5.2.2` → `5.5.0`)

## 5.3.0 — 2026-08-08

- Spec 45 Path A Deep Code cut: public `deepseek-build`/`dsb` agent R0A multi-edit
  with session-local `snippet_id`, plus stale-id / bash invalidation fail-closed proof
  (stacked on VC003–VC005 mint/require/expire laws)
- **Not published:** in-repo cut merged on `main` (PR #138) only — npm and GitHub Releases skipped `5.3.0` (published `5.2.2` → `5.5.0`)

## 5.2.2 — 2026-08-08

- installer self-check + fresh inode (fix silent corrupt install)

## 5.2.1 — 2026-08-08

- DeepSeek Night v2 markdown hierarchy restore: h2 headings, code, and command
  lines regain hue (blue h2/code, v1-yellow commands) on top of the unchanged
  C-balanced surfaces

## 5.2.0 — 2026-08-08

- theme classic default + vision complete + theme picker restore

## 5.1.0 — 2026-08-07

### Changed
- Default theme is now **DeepSeek Night v2**, a measured C-balanced palette:
  six semantic hue families, zero hue collisions, and every text role at WCAG AA
  or better on both the base and raised surfaces.
- Hover and selection separate on different axes (lightness vs chroma).
- Grok Night, classic DeepSeek Night, and DeepSeek Night Neutral are no longer
  listed in theme pickers. Existing configs naming them keep working.

### Fixed
- Settings theme sheet now lists the shipped product theme, so users can switch
  back without using `/theme`.
- `oscura-midnight` renders as "Oscura Midnight" instead of a raw identifier.

## 5.0.1 — 2026-08-07

- widen the DeepSeek whale logo to official terminal proportions

## 5.0.0 — 2026-08-07

- Owner-bar complete product cut (`owner-bar-5x`): Path A P0 ledger green, dual adversarial reviews, tag `v5.0.0`.

## 4.0.4 — 2026-08-07

- **Image attachments** on text-only DeepSeek endpoints no longer 400: images persist to session assets with an agent-driven OCR hint (matches Reasonix/DeepCode preprocessing instead of silent drop)
- **DeepSeek status line** with account balance & cache hit rate
- **G003:** `mint file_version` on Path A `read_file` (snippet contract)
- **G004:** Standard `snippet_safe` tool_configs now always applied (dead wiring fix) + `liveness-3edits` harness scenario
- CI: dotslash install + full-build fallback fix for prebuilt tag runs

## 4.0.3 — 2026-08-07

### CLI

- **`dsb --resume [<id>]` / `-r`** resumes a full-screen TUI session (bare flag = most-recent session); `--minimal` / `--fullscreen` forwarded to the TUI
- Quit and screen-mode relaunch hints branded as **`dsb`** / **`deepseek-build`** (via `GROK_INVOCATION_NAME`) instead of upstream `grok`
- `--resume` conflicts with `--session`; TUI-only flags rejected on line-mode subcommands (`run`, `chat`, …)

## 4.0.2 — 2026-08-07

### UX

- `dsb setup` next steps: bare **`dsb`** (full-screen agent TUI), not legacy `dsb chat`
- `chat` documented as line-mode only

## 4.0.1 — 2026-08-07

### Install DX (critical)

- **`npm i -g` no longer compiles Grok from source** (ADR 0009).
- `postinstall` downloads platform prebuilts from GitHub Releases into `~/.deepseek-build/bin/`.
- npm package is thin (wrappers only) — no `third_party/grok-build` in the tarball.
- Optional source fallback: `DEEPSEEK_BUILD_ALLOW_SOURCE_BUILD=1` only.

## 4.0.0 — 2026-08-07

### L3 productization (PRD-v4 / fleet-4x)

- **Product defaults:** `[subagents] enabled = true` in auto-created product config; keep **`yolo = false`** (hearts)
- **Capability matrix** + user guides 11–14 (subagent / bg / worktree / throughput)
- **Smoke:** `./scripts/test-l3-smoke.sh`
- Tag **`v4.0.0`** (full SemVer only)

## 3.0.0 — 2026-08-07

### Heart fusion (product major)

- **2.x was shell cut; 3.0.0 is heart fusion** (PRD-v3 P0)
- L1: Path A snippet_safe edit + Spec 90 permissions matrix (not YOLO default)
- L2: Path A prefix assembly + tool-call repair + Flash-first / Pro escalate
- Honesty docs: README, KNOWN_LIMITS, cut evidence
- Tag **`v3.0.0`** (full SemVer only)

### Residual (honest)

- Spec 45 **file_version** equivalent on Grok path (full snippet_id mint polish → 3.x minor)
- Live dogfood env-gated; L3 product identity → 4.x

## 3.0.0-beta.2 — 2026-08-07

### L2 repair + Flash/Pro (Path A)

- `dsb-agent` `path_a_turn`: Spec 15 prep-before-execute + Spec 20 Flash default / Pro once under agent defaults
- H15.* / H20.* contract tests; H2 exit band

## 3.0.0-beta.1 — 2026-08-07

### L2 prefix (Path A agent context)

- `dsb-context` `assemble_path_a_context`: stable prefix + volatile tail under Spec 10 for default agent path
- H10.* epoch stability tests (identical inputs, tool/skills thrash, volatile isolation)

## 3.0.0-alpha.2 — 2026-08-07

### L1 permissions (Path A)

- Spec 90 spirit matrix for default agent: headless Ask→Deny, TTY Ask for writes, deny out-of-cwd
- Product seed/repair: explicit `yolo = false` when missing (does not clobber user `yolo = true`)
- `dsb-tools` `path_a_permissions` contract tests (H90.*)

## 3.0.0-alpha.1 — 2026-08-07

### L1 heart (Path A snippet-safe)

- Spec 45 spirit on **default Grok** `search_replace` path: `snippet_safe` + `file_version` gate; empty-old whole-file overwrite fail-closed
- Product adapter: `dsb-tools` `path_a_edit` contract tests (H45.*)
- Standard file toolset injects `snippet_safe=true` for DeepSeek agent

## 2.0.3 — 2026-08-07

### Install (product contract)

- **`npm install -g @innocarpe/deepseek-build` postinstall** now builds and installs:
  1. wrapper `dsb` / `deepseek-build`
  2. full-screen agent `deepseek-build-agent` (DeepSeek TUI)
- After install + PATH: **`dsb`** alone opens DeepSeek full-screen TUI
- Requires Rust + protoc (or dotslash). First agent build may take several minutes.
- Skip agent only: `DEEPSEEK_BUILD_SKIP_AGENT_BUILD=1` (not recommended)

## 2.0.2 — 2026-08-07

### Product entry (DeepSeek TUI only)

- Bare `dsb` / `deepseek-build` = **DeepSeek Build full-screen TUI** (product)
- CLI binary name / help Usage: **dsb** (not `grok`)
- User-facing help no longer describes the product as "Grok-class" / Grok Build UI
- `repl-legacy` hidden; line-mode remains as `chat` only for legacy/script use

## 2.0.1 — 2026-08-07

### UI / UX (DeepSeek product chrome)

- **DeepSeekNight** default TUI theme (`#4D6BFE` accents) in vendored Grok pager
- Welcome hero: DeepSeek whale braille logo + **DeepSeek Build** product strings
- Launcher splash: whale + DeepSeek Build before full-screen agent
- Force `GROK_THEME=deepseeknight` from `dsb` entry (override with `DEEPSEEK_BUILD_THEME`)
- Product config seed includes `theme = "deepseeknight"`

## 2.0.0 — 2026-08-06

### Product

- **First product release** matching REPLAN_2.0 P0: Grok Build–class agent entry with DeepSeek default
- Vendored Grok Build under `third_party/grok-build/` (ADR-0008)
- No-args TTY `dsb` / `deepseek-build` launches `deepseek-build-agent` (Grok pager)
- DeepSeek models + `api.deepseek.com` + chat_completions config seed
- Setup/auth under `~/.deepseek-build/` (credentials 0600)
- L1/L2 evidence: `docs/product/evidence/W3_L1_L2_MATRIX.md`
- W2 chat/edit dogfood evidence under `docs/product/evidence/`
- `1.x` remains legacy scaffold on npm history

### Notes

- npm publish may still require human OTP (ADR 0007 residual)
- Upstream pager chrome may still say “Grok” in places; product CLI/docs use DeepSeek Build

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

- **Welcome banner v2** — DeepSeek braille whale mark (official logo silhouette raster) + boxed product card (`banner.rs`)
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
