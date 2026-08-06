# Changelog

## 1.1.0 — 2026-08-06

### Added

- **First-run setup onboarding** — `setup` / `auth login|status|logout`
- TTY `chat`/`run` auto-prompt for API key when missing; saves `credentials.json` (0600)
- Bare `deepseek-build` with no key starts setup on TTY
- User guide `00-setup.md`

## Unreleased

### CI

- Split product CI into path-gated parallel workflows (`rust-fmt`, `rust-clippy`, `rust-test`, `smoke-dogfood`, `semver`) with shared Cargo cache (`workspace-v1`); docs-only PRs skip Rust jobs

## 1.0.0 — 2026-08-06

### Release

- First **1.0.0** after Waves A–D: dogfood core, DeepSeek-native surface, Grok-class throughput, RC harden/docs
- Product CI (later refined into split path-gated workflows)
- Full user-guide + known limits
- Dual CLI `deepseek-build` / `dsb`; npm package `@innocarpe/deepseek-build` (registry publish remains owner-gated)

### Notes

- See `docs/product/KNOWN_LIMITS.md` for honest limits (MCP thin, in-process subagents, no prebuilt multi-arch CDN)

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
