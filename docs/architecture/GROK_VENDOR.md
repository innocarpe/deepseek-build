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

## The DeepSeek overlay

The vendored tree is "as upstream" plus a **product overlay**: the deliberate
local deviations that make this Grok-derived code the DeepSeek Build product.
The overlay is carried **in the tree** — a refresh must preserve it, and
whatever method is used (see the sync runbook) is judged by whether the overlay
survived.

| File | Overlay | Why |
|------|---------|-----|
| `crates/codegen/xai-grok-pager/src/app/mod.rs` | `print_exit_resume_hint` prints the command from env `GROK_INVOCATION_NAME` (default `grok`) via `invocation_name()` + pure `resume_hint_line()` | dsb-cli brands quit hints `dsb --resume <id>` so the printed command is pasteable |
| `crates/codegen/xai-grok-pager/src/app/screen_mode_relaunch.rs` | `screen_mode_relaunch_resume_hint` uses `super::invocation_name()` (pure `_with` variant for tests) | Same branding for the screen-mode relaunch failure hint |
| `crates/codegen/xai-grok-pager/src/app/terminal_restore.rs` | The panic hook ends with `super::disable_mouse_paste_raw()` | The mouse/paste reset must remain last so a panicking agent shutdown cannot leave mouse reporting or bracketed paste enabled in the user's shell |
| `crates/codegen/xai-grok-pager-render/src/theme/deepseeknight.rs` | Classic blue-ramp and neutral-ramp DeepSeek constructors via shared `deepseeknight_inner(neutral)` plus ramp/blue-accent unit tests | Classic `deepseeknight` remains picker-selectable and product/default; `deepseeknight-neutral` is first-class/selectable |
| `crates/codegen/xai-grok-pager-render/src/theme/deepseeknight_v2.rs` | The measured C-balanced `deepseeknight_v2()` palette and its integrity tests | Selectable DeepSeek Night v2 alternate, exposed as `deepseeknight-v2`; classic remains the product/default skin |
| `crates/codegen/xai-grok-pager-render/src/theme/mod.rs` | Registers `DeepSeekNightV2`, classic `DeepSeekNight`, neutral `DeepSeekNightNeutral` in `ThemeKind`/`ALL`/picker order, with `display_name`, `aliases`, `requires_truecolor`, `display_name_for_canonical`, and `Theme::current()`/`Default` resolving to the classic skin | Classic owns product/runtime defaults; V2, classic and neutral are selectable; only `groknight` stays hidden/compatibility-only |
| `crates/codegen/xai-grok-pager-render/src/theme/cache.rs` | `CURRENT` plus config/appearance resolution defaults to `DeepSeekNight`; the byte↔kind table covers the DeepSeek kinds; terminal-native lock intentionally reports nominal `GrokNight` | Classic is the product/default dark fallback while explicit V2 and legacy theme values remain honored |
| `crates/codegen/xai-grok-pager-render/src/theme/system_appearance.rs` | Dark appearance fallback → `DeepSeekNight`; light appearance fallback → `GrokDay`; explicit overrides remain honored | Auto mode follows the classic dark and GrokDay light defaults |
| `crates/codegen/xai-grok-pager-render/src/syntax.rs` | Night syntax group includes V2, classic/neutral DeepSeek kinds, `GrokNight`, `TokyoNight`, `RosePineMoon`, `OscuraMidnight` | All supported dark variants share the night syntax palette; terminal-native mode remains nominal `GrokNight` |
| `crates/codegen/xai-grok-pager/src/settings/defs.rs` | `THEME_CHOICES` / `CONCRETE_THEME_CHOICES` lead with `deepseeknight-v2`, canonical `deepseeknight` (`DeepSeek Night (classic)`) and neutral; `theme` / `auto_dark_theme` default to `deepseeknight` | Settings pickers surface V2, classic and neutral; classic is the product/default choice |
| `crates/codegen/xai-grok-shell/src/extensions/deepseek.rs` + `extensions/mod.rs` | The `x.ai/deepseek/status` extension: balance and session usage for the status line | The product's account/cost surface has no upstream equivalent |
| `crates/codegen/xai-grok-pager/src/app/dispatch/deepseek.rs` (+ `tests/deepseek.rs`) | `Effect::FetchDeepSeekStatus` polling, `TaskResult::DeepSeekStatus*` handling, session-safe staleness | Renders the DeepSeek balance row without leaking results across sessions |
| `crates/codegen/xai-grok-pager/src/app/agent_view/*`, `views/agent_status.rs` | The `deepseek_status` layout row and balance chips (currency formatting) | The bottom status row is product UI |
| `crates/codegen/xai-grok-sampling-types/src/{conversation,types}.rs` | Maps DeepSeek `prompt_cache_hit_tokens` into `cached_read_tokens` | Cache-hit reporting is the L2 contract; upstream has no DeepSeek wire shape |
| `crates/codegen/xai-grok-agent/templates/prompt.md` + `src/prompt/prompt_encrypted.rs` | The agent prompt carries no hardcoded third-party vendor claim; the encrypted copy is regenerated from the template | Product identity. Regenerate with `python3 crates/codegen/xai-grok-agent/scripts/encrypt_templates.py` |
| `crates/codegen/xai-grok-shell/src/builtin.rs` | The version-transition cleanup removes only `CHANGELOG.json`; `$GROK_HOME/CHANGELOG.md` is preserved | `dsb-cli seed_product_changelog` writes it every launch, and deleting it made the welcome-screen CHANGELOG click a silent no-op |
| `crates/codegen/xai-grok-update/src/*` | The product update feed: npm/gh-release coordinates, a default of `"npm"` for unknown installer classifications, and `installer_allows_downgrade` returning `false` unconditionally | The product has no x.ai CDN pointer to roll back, so a lower version from any source must never be installed over it |
| `crates/codegen/xai-grok-pager-bin/build.rs` | `DEEPSEEK_BUILD_VERSION` first, then `GROK_VERSION`; version-derived `--cfg` **and** an `OUT_DIR` file read with `include_str!` | sccache keys on neither `env!` alone — a warm-cache release once shipped the previous version |
| `crates/codegen/xai-grok-pager/src/lib.rs`, `trace_cmd.rs`, `tracing.rs`, `app/cli.rs`, `app/mod.rs`, `app/effects/*` | `crate::VERSION_WITH_COMMIT`, `name = "dsb"`, `about = "DeepSeek Build TUI"`, `DeepSeek Build` notification titles | Product version, command name and identity in the TUI surface |
| `crates/codegen/xai-grok-tools/src/types/snippet_store.rs`, `implementations/grok_build/read_file/mod.rs` | Snippet-store wiring and the snippet-safety/truncation path in `read_file` | Spec 45's snippet contract is L1 and must not be bypassed |
| `crates/codegen/xai-grok-shell/src/session/helpers/{path_a_cache_signal,spec10_path_a_assembly}.rs` | Path A cache-signal and spec-10 assembly helpers | L2 cache discipline and the Path A assembly contract |
| `crates/dsb-cli/src/agent_launch.rs` | First-launch picker writes the chosen skin into the seed config; `GROK_THEME`/`LC_GROK_THEME` only set from explicit env (`DEEPSEEK_BUILD_THEME`/`GROK_THEME`) | First-launch classic/V2 onboarding plus persistent theme changes |

Tests: `resume_hint_line_brands_invocation_name` and
`failed_relaunch_hint_brands_invocation_name` pin the `dsb` output; upstream
default (`grok`) assertions keep passing. Theme tests pin the official
`#4D6BFE` accent on the classic and neutral constructors, the v2 palette
invariants, the classic dark/GrokDay light resolution defaults, and the picker
contract. Settings preview coverage retains legacy theme kinds while both
catalogs expose `deepseeknight-v2`, `DeepSeek Night (classic)` and
`DeepSeek Night Neutral`.

### Patch series (historic)

`patches/grok-build/` carried thirteen patches between the `1.0.0` vendor land
and the `1.0.41` sync. **The `1.0.41` sync superseded them**: measured against
the new base, all thirteen conflicted, and the overlay had grown to 133 files
with in-tree edits the series never carried. The overlay is now carried in the
tree, and `./scripts/apply-grok-build-patches.sh` reports that and exits 0.

A series is still the right shape when a **small** overlay must survive a
`rsync --delete` refresh; regenerate one with
`git format-patch <base>..HEAD -- third_party/grok-build -o patches/grok-build`
if that method is chosen again.
|------|-------|-----|
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
