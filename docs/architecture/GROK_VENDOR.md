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

### Capturing the pager

- Build the pager binary **once** and reuse it for every width you capture
  (55×41, desktop). Rebuilding per width pays the same link cost again.
- A PTY launched with an empty `GROK_HOME` stops on the sign-in screen
  (`Approve in your browser to finish signing in.`) and paints no layout frame
  (measured 2026-09-26, tmux at 55×41). Capture through
  `xai-grok-pager-pty-harness` instead: `flows::seed_fake_oauth` for auth plus
  `spawn_minimal_sized(content, rows, cols)` for the sized spawn — the helper
  and the pattern are in
  `crates/codegen/xai-grok-pager-pty-harness/tests/pty_e2e/common.rs`.

---

## Refresh procedure

**The full procedure lives in
[`skills/grok-sync`](../../skills/grok-sync/SKILL.md) and
[`grok-sync-runbook.md`](../../docs/contributing/grok-sync-runbook.md).** Start
with the inventory, which measures the gap before anyone plans a method:

```bash
./scripts/grok-sync-inventory.sh
```

Never silent-copy. Always a dedicated PR, and choose the method from the
measurement:

| Method | When |
|---|---|
| **Patch re-apply** (below) | the series is non-empty **and** `./scripts/apply-grok-build-patches.sh --check` passes |
| **Three-way merge** (runbook §2) | the series is empty (overlay in-tree), any patch conflicts, or the overlay is large |

### Patch re-apply path (small overlay)

1. Update sibling or clone upstream Grok Build to the desired rev.  
2. `rsync -a --delete --exclude target --exclude .git <src>/ third_party/grok-build/`  
   (or `git subtree pull` if that workflow is adopted later), then
   `./scripts/apply-grok-build-patches.sh` to re-apply the local patches.
3. Confirm `SOURCE_REV` matches the intended pin.  
4. Re-run `./scripts/build-grok-pager.sh check`.  
5. PR title: `chore(vendor): refresh grok-build to <short-sha>` with license note unchanged.

Record the sync in
[`docs/product/UPSTREAM_SYNC_LEDGER.md`](../product/UPSTREAM_SYNC_LEDGER.md) —
what was taken, held and rejected, and why.

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
| `crates/codegen/xai-grok-pager-render/src/syntax.rs` | Night syntax group includes V2, classic/neutral DeepSeek kinds, `GrokNight`, `TokyoNight`, `RosePineMoon`, and `OscuraMidnight` | All supported dark variants share the night syntax palette; terminal-native mode remains nominal `GrokNight` |
| `crates/codegen/xai-grok-pager/src/settings/defs.rs` | `THEME_CHOICES` / `CONCRETE_THEME_CHOICES` place `deepseeknight-v2` before canonical `deepseeknight` (`DeepSeek Night (classic)`) and include neutral as a selectable DeepSeek skin | Settings pickers surface V2, classic, and neutral; classic is the product/default choice, and only GrokNight remains hidden compatibility |
| `crates/codegen/xai-grok-pager/src/views/settings_modal/tests.rs` | Exhaustive preview coverage retains V2, classic, and neutral `ThemeKind` arms | Preview remains compatible with legacy/config values while the picker catalogs expose classic after the V2 picker entry |
| `crates/codegen/xai-grok-sampler/src/client.rs` | `strip_image_content_blocks()` + `is_text_only_deepseek_model()` / `is_text_only_wire()` gate the wire flattening on **endpoint and model**; `conversation` / `conversation_stream` call the pair | DeepSeek's official V4.1 Flash accepts `image_url` (`input_modalities: ["text", "image"]`), so the old endpoint-only gate silently dropped images the model could read; V4 Pro (`["text"]`, Vision "Not supported") keeps the text-only wire |
| `crates/codegen/xai-grok-sampler/src/client.rs`, `crates/codegen/xai-grok-sampling-types/src/{types.rs,conversation/chat_completions.rs}` | `is_openrouter_endpoint` + `pin_openrouter_session` put the session id in the Chat Completions body as OpenRouter's `session_id` sticky-routing key; `chat_completion` and `chat_completion_stream` call the pin, and `ChatCompletionRequest::session_id` is skipped when unset | OpenRouter re-picks an upstream provider per request unless the body carries `session_id`, and each provider keeps its own prefix cache — a mid-session switch re-bills the whole prompt at the input rate. Every session (subagents included) carries its own id, and every other endpoint's body stays byte-identical |
| `crates/codegen/xai-grok-shell/src/session/image_describe.rs` | `<image_files>` envelope states the OCR fallback conditionally ("If the image is not visible to you directly (a text-only API model, for example DeepSeek V4 Pro)…") | The envelope text is what a text-only model has instead of the image; the unconditional upstream wording no longer describes the V4.1 Flash wire |
| `crates/codegen/xai-grok-shell/src/session/acp_session_impl/turn.rs` | The non-cursor branch persists pasted **and** base64-extracted images together before prepending `<image_files>` | A text-only model (V4 Pro) cannot receive `image_url`, so the on-disk paths are its only image channel; vision-capable models keep the inline parts too |
| `crates/dsb-cli/src/agent_launch.rs` | First-launch picker writes the chosen skin into the seed config; `GROK_THEME`/`LC_GROK_THEME` only set from explicit env (`DEEPSEEK_BUILD_THEME`/`GROK_THEME`) so in-pager `/theme` persists | First-launch classic/V2 onboarding plus persistent theme changes; classic remains the product/default and V2 remains selectable |
| `crates/codegen/xai-grok-pager/src/notifications/{agent_status.rs,mod.rs}` | OSC 9999 explicit agent-status frames for hosts that read them (Orca): `working` / `waiting` / `done`, ridden on the same tick as the title, and closed explicitly on turn end and pane exit | A host that reads the status stream needs the turn boundary stated rather than inferred from the tab title; silent on every other host |
| `crates/codegen/xai-grok-pager/src/scrollback/blocks/user.rs` | `collapsed_max_lines(width, mode)` replaces the fixed `COLLAPSED_MAX_LINES` budget: at or below `COLLAPSED_NARROW_TERMINAL_COLS` (60) a collapsed prompt folds to `COLLAPSED_NARROW_MAX_LINES` (1); wider keeps 3; `Expanded` still never folds | The measured iPhone Orca pane is 55 columns, where a three-line echo of the submitted prompt eats a third of the viewport; desktop widths (80+) keep today's three-line budget |
| `crates/codegen/xai-grok-pager/src/app/mouse.rs`, `app/agent_view/selection.rs`, `scrollback/state/selection.rs` | At or below `COLLAPSED_NARROW_TERMINAL_COLS`, a same-cell tap on a collapsed foldable user prompt expands it and pins that entry to the top of the scrollback with follow mode off. A later tap collapses only the expanded echo's first line; the body does not fold, and a drag stays a text selection. `word_select` does not swallow that tap. Wider panes keep double-click fold and do not expand on a single click | A one-line phone echo is the whole prompt a touch pane can show, and one tap is the gesture that pane delivers. Double-click and `word_select` were eating it, and leaving follow mode on scrolled the opened text away |
| `crates/codegen/xai-grok-pager/src/scrollback/block.rs`, `scrollback/blocks/user.rs` | `BlockContent` gains width-aware `is_foldable_at(content_width)` / `has_vpad_for_width(appearance, content_width)` (both defaulting to the width-blind forms); `UserPromptBlock` overrides both. Its fold is measured against `collapsed_max_lines` at the width the text wraps at, and that width comes from `prefix_for` — the same function `wrap_prompt_lines` paints — so the budget sees the prefix that was actually drawn. Vpad is dropped at or below `COLLAPSED_NARROW_TERMINAL_COLS` | The width-blind `is_foldable()` divides by a 60-column constant, so a ~100-column one-liner scored 2 rows, stayed `Expanded`, and `max_lines = None` skipped the narrow budget entirely |
| `crates/codegen/xai-grok-pager/src/scrollback/{entry.rs, scrollback_pane.rs, wrappers/entry_renderer.rs, state/*}` | The entry-area → content-width step is shared (`EntryRenderer::block_content_width`, which still honours `hide_accent`, and `wrappers::block_content_width_for` for the state and the pane). Fold and vpad gates on push, resize, selection and reveal ask the width-aware form. Unpinned prompts re-derive their default fold when the pane width changes | One number for how wide this prompt wraps, or a block is measured at one width and painted at another. A desktop-submitted prompt folds in the phone pane and unfolds again when the pane widens |
| `crates/codegen/xai-grok-pager/src/scrollback/blocks/{user.rs, mod.rs}`, `views/prompt_widget/mod.rs` | `UserPromptBlock::prefix_for(show_prefix, width)` owns the echo prefix: `$ ` and `↻  ` always draw, the decorative `❯ ` is dropped at or below `COLLAPSED_NARROW_TERMINAL_COLS`. The composer's `shows_prefix` uses the same constant, compared against the pane width, and keeps history `? ` and `prefix_override` (`! `) | Two columns of a 55-column pane are two columns of the user's own text. Comparing the threshold to content width instead of pane width drops the arrow on a 60-column chromeless box and shifts mouse hit-testing |
| `crates/codegen/xai-grok-pager/src/views/prompt_widget/mod.rs` | The bottom info line's rect is inset one cell per corner (`area.x + 1`, `area.width - 2`) instead of starting at `content_area.x` | The label no longer paints the divider rule immediately after `╰` when it overflows a phone-width pane, and keeps a blank pad before `╯` |
| `crates/codegen/xai-grok-pager/src/views/prompt_widget/mod.rs`, `app/agent_view/render.rs` | At or below `COLLAPSED_NARROW_TERMINAL_COLS` (60) the prompt box's bottom divider is a plain `╰──╯` rule and the model/mode label takes its own row below the box: `info_block` reserves that second row and `render_info_line` paints it on `divider_y + 1` with the same content, style and usage-warning chip. The same threshold drops the shortcut-hint row from the agent layout (`shortcuts_height = 0`, a no-op in `ShortcutsBar::render`) in every state — idle, turn running, viewer. Wide panes keep the label on the divider row | On the measured 55-column iPhone Orca pane the label on the border and the key hints underneath each waste a row of a 41-row viewport. The label row takes the slot the hints held, so the bottom stack (box + label + DeepSeek status row) is exactly as tall as before, and no hint row is reserved in any state |
| `crates/codegen/xai-grok-pager-render/src/appearance/config.rs`, `crates/codegen/xai-grok-pager/docs/user-guide/{05-configuration.md,06-theming.md}` | `LayoutConfig`/`RawLayoutConfig` defaults are flush at every width: `outer_vpad 0`, `outer_hpad_left/right = MIN_HPAD (1)`, `block_pad_left/right 0`; the user-guide tables carry the same numbers, and `eff_box_pad_*` keeps the composer's one-column border inset. The one horizontal column is the selection border's — `selection_area()` extends into each side pad, pinned by `selection_border_fits_the_reserved_outer_column` | Before/after frames: 55x41 phone and 120x40 / 180x50 desktops moved from a blank row above the status bar and text at column 5 to the status bar on row 0 and text at column 2, gaining 4 rows and 4 text columns per pane. Claude Code and Codex CLI sit flush, and the margin rows cost content on every pane, not only the phone |
| `crates/codegen/xai-grok-pager/src/scrollback/block.rs`, `views/agent.rs`, `app/{app_view.rs,event_loop.rs}` | `NARROW_TERMINAL_COLS` (60) and `LayoutConfig.narrow` gate the phone-only *behavior*: the default `has_vpad_for_width` drops the block vpad when the flag is set, and the flag is derived from `last_known_terminal_cols` by `views::agent::effective_narrow`, fanned out with the compact flag by `AppView::apply_effective_density` on startup and every `Event::Resize` | The echo fold, the dropped `❯` and the thinner phone frame must not follow a desktop pane. Desktop layouts keep the three-line echo, the arrow and the block vpad; the pad values themselves no longer depend on the width |
| `crates/codegen/xai-grok-pager/src/app/agent_view/paste.rs` | `try_handle_dropped_paths_paste()` drops its `is_ssh` early-return: an image path pasted into a remote pane is classified like any other, and the PTY case `ssh_image_path_attaches` pins it | The host that runs the pager is the host that can resolve the path and read its bytes, so SSH changes nothing about the classification. Hosts that paste images this way upload the bytes to that same host first — an Orca SSH pane writes the temp file over SFTP to the remote `$TMPDIR` and pastes the remote path. The early-return made such a paste land as literal path text, so image attachment was impossible over SSH |
| `crates/codegen/xai-grok-shell/src/session/helpers/spec10_path_a_assembly.rs` | `place_stable_body`: the first Spec 10 body is written into the leading system message; a later body is appended and the earlier system message stays byte-for-byte. The file `#[path]`-includes `spec10_path_a_cache_guard.rs` | Spec 10 §1.10. Rewriting the leading system measured `cached_tokens = 0`; an appended system message did not force that |
| `crates/codegen/xai-grok-shell/src/session/helpers/spec10_path_a_assembly.rs` and `crates/codegen/xai-grok-shell/src/session/acp_session_impl/turn.rs` | `observe_path_a_prefix_change`: when the Path A epoch differs from this session's previous assembly in this process, the turn logs which of the five assembled documents moved (`prefix_change=`). No baseline and an unchanged epoch log no change line | Spec 10 Path A attribution. The shell does not depend on `dsb-context`, so the overlay shape never ran on this turn. Axes are the documents the assembly already concatenates |
| `crates/codegen/xai-grok-sampling-types/src/types.rs`, `conversation.rs`, `crates/codegen/xai-chat-state/src/usage.rs`, `crates/codegen/xai-grok-shell/src/session/acp_session_impl/turn_end.rs` | Chat Completions `Usage` keeps `prompt_cache_miss_tokens`. The session ledger sums hit and miss only when those fields were sent, and `emit_turn_completed` logs `cache_session=` once per turn | Spec 10 §1.5.2 on Path A. The official host sends the miss; dropping the field made the client report a hit with no miss. The overlay counter in `dsb-agent` is unchanged |
| `crates/codegen/xai-grok-shell/src/session/helpers/spec10_path_a_cache_guard.rs` | Spec 10 §1.9 Path A bench. Scored bytes are `assemble_spec10_path_a_turn` + `place_stable_body`, then `ChatCompletionRequest::from` (`conversation_to_chat_messages`). The mock accounts the `messages` array | The overlay bench scores `dsb-context`. Path A does not link that crate, so a guard that lives only there stays green when this assembly breaks |
| `crates/codegen/xai-grok-shell/src/util/config/announcements.rs` + `agent/init.rs`, `agent/mvp_agent/agent_ops.rs` | `strip_remote_announcements` removes the `announcements` field at every `remote_settings` write (startup getter, boot wait, shared store, poll-only apply); `resolve_announcements` never merges the `remote` layer | dsb carries no xAI/Grok announcements: the fetch stays (the same blob carries harness settings), but announcements neither enter stored settings nor reach the pager's display set, so the welcome hero, session banner, header/dashboard CTA and `/announcements` gate stay empty for remote items. `GROK_ANNOUNCEMENTS_OVERRIDE` and config TOML layers are kept |

Tests: `resume_hint_line_brands_invocation_name` and
`failed_relaunch_hint_brands_invocation_name` pin the `dsb` output; upstream
default (`grok`) assertions keep passing. Theme tests pin the official
`#4D6BFE` accent on the classic and neutral constructors, the v2 palette
invariants, the classic dark/GrokDay light resolution defaults, and the picker
contract. Settings preview coverage retains legacy theme kinds while both
catalogs expose `deepseeknight-v2`, `DeepSeek Night (classic)` and
`DeepSeek Night Neutral`.
The OpenRouter pin is pinned from both ends:
`openrouter_session_pin_adds_the_session_id_and_nothing_else` asserts the
pinned body differs by exactly that one field (and that a subagent session
carries its own id), `openrouter_session_pin_stays_off_every_other_endpoint`
and `openrouter_session_id_rides_only_the_openrouter_wire` keep the field off
every other wire — the latter through the mock server on both the unary and
the streaming entry point — and
`is_openrouter_endpoint_matches_only_openrouter_hosts` keeps the lookalike
`notopenrouter.ai` out.
 The announcement boundaries are pinned from both sides:
`install_remote_settings_strips_remote_announcements_before_the_gate` keeps
the stored field gone and the push gate silent, `resolve_announcements`
ignores the remote layer while the override seed still wins, and the pty
announcement cases assert that remote pushes never paint.
The flush frame and the phone-only render gates are pinned end to end by the
PTY cases `tight_frame_and_phone_only_gates` (55x41 flush, text at column 2, the
arrow absent; 120x40 flush with the arrow back) and
`flush_frame_holds_status_bar_on_row_zero` — all `#[ignore]`d for ordinary
cargo runs, like the rest of the `pty_e2e` family.
 The SSH paste deviation is pinned by the PTY case
`ssh_image_path_attaches`, which spawns the pager with `SSH_CONNECTION` set —
a unit test cannot, because `terminal_context()` is a process-wide static
computed once from the environment.
The phone bottom stack is pinned at the widget by
`phone_pane_moves_the_label_off_the_divider_rule` (plus the width loops in
`phone_divider_and_label_rows_keep_the_pane_width` and
`phone_width_budgets_the_label_row_on_top_of_the_divider`) and end to end by
`app::agent_view::phone_bottom_tests`, which renders the whole view at 55×41
and 120×40 and asserts the label row, the absent hint row and the DeepSeek
status row.

### Patch series

The overlay is carried **in the tree**. `patches/grok-build/` has no `*.patch`
files. Git does not keep that directory once the last patch is deleted; the
script treats a missing directory the same as an empty one and exits 0.

Sync 2 (ledger: `1.0.0` → `1.0.41`) measured **0 of 13** patches applying.
`844c7be` deleted `0001`–`0013` and moved that overlay in with a three-way
merge. `0014` (OSC 9999 agent status) was the one file left in the directory.
On parent `0ff7d8c` it applies neither way:

| Direction | Result |
|---|---|
| `git apply --check` | `agent_status.rs` already exists, and `mod.rs` does not apply |
| `git apply --reverse --check` | `mod.rs` does not apply |

Those sources stay in the tree (`notifications/agent_status.rs` and the
`mod.rs` registration — the overlay table above). `0014` is removed the same
way `0001`–`0013` were. Restoring any of `0001`–`0014` on top of the in-tree
overlay double-applies or fails `--check`. Do not put them back to recreate
the old 14-row table.

What keeps that behavior across a refresh is the three-way merge in the
runbook, judged against the overlay table in this file. The ledger records
what Sync 2 carried; this table is the inventory, including overlay added
after that sync (OSC 9999, the narrow-terminal prompt fold). A patch series
does not guard it. The old "never silently drop the 14-patch series" line
described files that are not in the directory.

| `patches/grok-build/*.patch` | `--check` | Refresh method |
|---|---|---|
| one or more, all applicable or already applied | exit 0 | patch re-apply (small-overlay path above) |
| one or more, any conflict | exit 1 | fix or drop the entry; do not `rsync --delete` |
| none, or the directory is absent | exit 0, nothing to apply | overlay is in-tree — three-way merge |

A series is still the right shape when a **small** overlay must survive an
`rsync --delete` refresh. Regenerate with
`git format-patch <base>..HEAD -- third_party/grok-build -o patches/grok-build`
only if that method is chosen again. An empty directory is not that case:
`--check` exiting 0 because there is nothing to apply does **not** make the
patch re-apply path safe. The runbook's size check still decides.

- **Re-apply** when the series is non-empty:
  `./scripts/apply-grok-build-patches.sh` (`--check` for a dry run;
  already-applied patches are skipped).
- **Regenerate** when a small series changes:
  `git format-patch <base>..HEAD -- third_party/grok-build -o patches/grok-build`
  where `<base>` is the merge-base of the vendor PR that carried the patches.
- **Refresh conflicts:** if the script fails after an upstream refresh, fix
  the conflicts by hand, re-run `./scripts/build-grok-pager.sh check`, and
  regenerate before merging the refresh PR.

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
