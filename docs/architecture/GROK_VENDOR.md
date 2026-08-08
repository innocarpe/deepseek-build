# Grok Build vendor layout (ADR-0008)

**Status:** Normative for product 2.0.0 base integration  
**Story:** `grokbase-2x` G004 / unit `2x-W1-1`  
**Spike:** [GROK_BASE_SPIKE.md](./GROK_BASE_SPIKE.md)

---

## Layout

| Path | Role |
|------|------|
| `third_party/grok-build/` | Vendored open-source Grok Build workspace (own `Cargo.toml`) |
| `third_party/grok-build/SOURCE_REV` | Upstream monorepo/sync pin SHA |
| `third_party/grok-build/LICENSE` | Apache-2.0 |
| `third_party/grok-build/THIRD-PARTY-NOTICES` | Upstream third-party notices |
| Root `NOTICE` | Product attribution pointing at this vendor tree |
| `crates/dsb-*` | **Overlay** (provider, config, L1/L2 policy, legacy thin REPL) |
| Product workspace root `Cargo.toml` | 1.x overlay crates only (for now) |

**Two Cargo workspaces on purpose:**

1. **Product workspace** (repo root) — `dsb-*` crates, SemVer product line, npm/install surface.  
2. **Vendor workspace** (`third_party/grok-build`) — Grok pager/agent/tools as upstream ships them.

W1+ product binaries build from the **vendor** workspace (pager composition root) and install as `deepseek-build` / `dsb`. Overlay crates stay in the product workspace for credentials, policy tests, and `repl-legacy`.

---

## Build (local)

Host tools (see spike):

- Rust **1.94.0** (vendor `rust-toolchain.toml`)
- **protoc** on PATH (or `cargo install dotslash` so vendor `bin/protoc` works)
- Recommended: `export PATH="/opt/homebrew/bin:$HOME/.cargo/bin:$PATH"` on macOS Homebrew

```bash
# From product repo root:
./scripts/build-grok-pager.sh check   # cargo check -p xai-grok-pager-bin
./scripts/build-grok-pager.sh release # cargo build -p xai-grok-pager-bin --release
```

Evidence for G004: the script must exit 0 for `check` on a machine with the host tools above.

---

## Refresh procedure

Never silent-copy. Always a dedicated PR:

1. Update sibling or clone upstream Grok Build to the desired rev.  
2. `rsync -a --delete --exclude target --exclude .git <src>/ third_party/grok-build/`  
   (or `git subtree pull` if that workflow is adopted later).  
3. Confirm `SOURCE_REV` matches the intended pin.  
4. Re-run `./scripts/build-grok-pager.sh check`.  
5. PR title: `chore(vendor): refresh grok-build to <short-sha>` with license note unchanged.

---

## Local patches

Local work on the vendored tree comes in two forms, and an
`rsync -a --delete` refresh would silently drop both — re-apply after every
refresh and keep this list current:

### Applied directly in the tree

The tree ships "as upstream", but a small set of **deliberate local deviations** lives in the vendored sources. `rsync -a --delete` in the refresh procedure would silently drop them — re-apply after every refresh and keep this list current:

| File | Patch | Why |
|------|-------|-----|
| `crates/codegen/xai-grok-pager/src/app/mod.rs` | `print_exit_resume_hint` prints the command from env `GROK_INVOCATION_NAME` (default `grok`) via `invocation_name()` + pure `resume_hint_line()` | dsb-cli brands quit hints `dsb --resume <id>` so the printed command is pasteable |
| `crates/codegen/xai-grok-pager/src/app/screen_mode_relaunch.rs` | `screen_mode_relaunch_resume_hint` uses `super::invocation_name()` (pure `_with` variant for tests) | Same branding for the screen-mode relaunch failure hint |
| `crates/codegen/xai-grok-pager-render/src/theme/deepseeknight.rs` | Keeps the classic blue-ramp and neutral-ramp DeepSeek constructors via shared `deepseeknight_inner(neutral)` plus ramp/blue-accent unit tests | Classic `deepseeknight` remains picker-selectable; neutral `deepseeknight-neutral` remains accepted through legacy names/config but is compatibility-only and hidden from pickers |
| `crates/codegen/xai-grok-pager-render/src/theme/deepseeknight_v2.rs` | Defines the measured C-balanced `deepseeknight_v2()` palette and its integrity tests | Selectable DeepSeek Night v2 alternate, exposed as `deepseeknight-v2`; classic remains the product/default skin |
| `crates/codegen/xai-grok-pager-render/src/theme/mod.rs` | Registers `DeepSeekNightV2` as the first picker-visible alternate; keeps classic in `ALL` and `NO_TRUECOLOR` as the product/default skin; retains neutral/GrokNight parser aliases for compatibility; preserves the `"dark"` alias to `GrokNight` | Classic owns product/runtime defaults while V2 remains selectable and neutral/GrokNight stay compatibility-only and hidden |
| `crates/codegen/xai-grok-pager-render/src/theme/cache.rs` | `CURRENT` plus config/appearance resolution defaults to `DeepSeekNight`; terminal-native lock intentionally reports nominal `GrokNight` | Classic is the product/default dark fallback while explicit V2 and legacy theme values remain honored |
| `crates/codegen/xai-grok-pager-render/src/theme/system_appearance.rs` | Dark appearance fallback → `DeepSeekNight`; light appearance fallback → `GrokDay`; explicit overrides remain honored | Auto mode follows the classic dark and GrokDay light defaults |
| `crates/codegen/xai-grok-pager-render/src/syntax.rs` | Night syntax group includes V2, classic/neutral DeepSeek kinds, `GrokNight`, `TokyoNight`, `RosePineMoon`, and `OscuraMidnight` | All supported dark variants share the night syntax palette; terminal-native mode remains nominal `GrokNight` |
| `crates/codegen/xai-grok-pager/src/settings/defs.rs` | `THEME_CHOICES` / `CONCRETE_THEME_CHOICES` place `deepseeknight-v2` before canonical `deepseeknight` (`DeepSeek Night (classic)`); neutral remains omitted from picker catalogs but accepted for compatibility | Settings pickers surface V2 and classic; classic is the product/default choice, V2 remains selectable, and neutral remains compatibility-only |
| `crates/codegen/xai-grok-pager/src/views/settings_modal/tests.rs` | Exhaustive preview coverage retains V2, classic, and neutral `ThemeKind` arms | Preview remains compatible with legacy/config values while the picker catalogs expose classic after the V2 picker entry |
| `crates/dsb-cli/src/agent_launch.rs` | First-launch picker writes the chosen skin into the seed config; `GROK_THEME`/`LC_GROK_THEME` only set from explicit env (`DEEPSEEK_BUILD_THEME`/`GROK_THEME`) so in-pager `/theme` persists | First-launch classic/V2 onboarding plus persistent theme changes; classic remains the product/default and V2 remains selectable |

Tests: `resume_hint_line_brands_invocation_name` and `failed_relaunch_hint_brands_invocation_name` pin the `dsb` output; upstream default (`grok`) assertions keep passing. Theme tests pin the official `#4D6BFE` accent on the classic and neutral compatibility constructors, the v2 palette invariants, the classic dark/GrokDay light resolution defaults, and the picker contract (V2 first; classic in `ALL`/`NO_TRUECOLOR`; neutral and GrokNight hidden). Settings preview coverage retains legacy theme kinds while both settings catalogs expose `deepseeknight-v2` before `DeepSeek Night (classic)`; classic remains product/default and neutral remains compatibility-only.

### Carried as patch files under `patches/grok-build/`

DSB carries local feature work on the vendored tree as patches under
`patches/grok-build/` — **outside** the vendor tree, so an `rsync --delete`
refresh cannot wipe them.

| Patch | Commit it derives from |
|-------|------------------------|
| `0001-*.patch` | `feat(sampling-types): map DeepSeek prompt_cache_hit_tokens into cached_read_tokens` |
| `0002-*.patch` | `feat(shell): add x.ai/deepseek/status extension for balance and session usage` |
| `0003-*.patch` | `fix(shell): preserve one-pass repair and repair pre-existing lib-test build breakage` |
| `0004-*.patch` | `test(pager): cover DeepSeekNight kinds in settings preview test` |
| `0005-*.patch` | `feat(pager): render DeepSeek status with session-safe polling` |
| `0006-*.patch` | `test(shell): cover large MCP image persistence and resume` |

These patches are the **DeepSeek status line** feature, its shell-side repair
dependency, and the focused large-MCP-image persistence/resume regression. The
status patch records session-bound unsupported capability handling, transient
retry, and stale-result safety; patch 0006 keeps the DSB image regression
durable instead of relying on the upstream changelog claim. A refresh must
never silently drop them.

- **Re-apply after refresh:** `./scripts/apply-grok-build-patches.sh`
  (add `--check` for a dry run; already-applied patches are skipped).
- **Regenerate** when the patch set changes:
  `git format-patch <base>..HEAD -- third_party/grok-build -o patches/grok-build`
  where `<base>` is the merge-base of the vendor PR that carried the patches.
- **Refresh conflicts:** if `apply-grok-build-patches.sh` fails after an
  upstream refresh, fix the conflicts by hand, re-run
  `./scripts/build-grok-pager.sh check`, and regenerate the patches before
  merging the refresh PR.

Refresh procedure step 2 therefore becomes:

2. `rsync -a --delete --exclude target --exclude .git <src>/ third_party/grok-build/`  
   (or `git subtree pull` if that workflow is adopted later), then  
   `./scripts/apply-grok-build-patches.sh` to re-apply the local patches.

---

## CI plan

### Default CI workflow (`ci.yml`)

Path filter for **product** Rust jobs remains root `crates/**`, root `Cargo.toml`, etc.  
**Does not** build the full Grok vendor on every docs or overlay-only PR (clone/build time).

### Vendor check (required when vendor tree changes)

| Item | Plan |
|------|------|
| Trigger | Path filter: `third_party/grok-build/**`, `scripts/build-grok-pager.sh`, this doc |
| Job | Install `protoc` + `dotslash`; run `./scripts/build-grok-pager.sh check` |
| Timeout | Long (30–60+ min cold; cache `third_party/grok-build/target` when practical) |
| Gate | Optional separate check name `grok-vendor-check` — enable as required once stable on ubuntu-latest |

Until the dedicated workflow is green on GitHub-hosted runners, **local** `./scripts/build-grok-pager.sh check` is the merge evidence for vendor PRs, recorded in the PR Testing section.

### Why not single fused workspace yet

- Vendor graph is large (proto, aws-lc, many crates).  
- Overlay SemVer / npm still track product `Cargo.toml` workspace package version.  
- Fusion remains possible later via ADR amendment if dual-root becomes unmaintainable (ADR-0008 notes revisiting strategy A only if B fails operationally).

---

## SemVer

Integration branch / first vendor land may ship product version **`2.0.0-alpha.N`**.  
Tag **`v2.0.0` only** at G012 with REPLAN P0 green.

---

## Related

- ADR-0008, REPLAN_2.0, WAVE_2x W1  
- Install dual bins: G005+ / `scripts/install.sh` evolution  
- npm human publish: ADR 0007
