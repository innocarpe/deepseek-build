# Research: Grok Build

**Local path:** `../../../grok-build` (from this file: `OpenSources/grok-build`)

## Why it matters

Primary answer to: *why does work finish so fast?*

## Observations (scaffolding-era summary)

- **Rust monorepo**: TUI (`xai-grok-pager`), agent shell, tools, workspace as separate crates
- **Parallel tools** + **background shell** + multi-wait
- **Subagents** with own context; optional **worktree** isolation (`xai-fast-worktree`)
- **Workflow engine** (Rhai) with parallel agent panels and budgets
- Hashline-style edit anchors, codebase graph, fast git/fs helpers

## Takeaways for DeepSeek Build

- Optimize for fewer serial waits
- Subagents should not re-pay huge uncached prefixes carelessly (tension with DeepSeek cache — design carefully)
- Modular package boundaries beat one `src/` blob

## Not taking yet

- Full hard-fork
- xAI auth/models catalog as-is

## 0.2.121 sync record (non-binding)

This note records the source-refresh evidence and adoption decisions; it is
research, not a product policy. The integration preserves the already-shipped
DSB product version at full SemVer `5.2.1` on `main`; this vendor PR does not
change the product version.

| Source identity | Open sync | `SOURCE_REV` | Source version |
| --- | --- | --- | --- |
| Old baseline | `a5589e958437d79e13db026eedcb1720bffd4063` | `4d6d11372ab8f73026a78c45a7b7e7b1310eb39f` | `0.2.120` |
| Refresh target | `393430ee4934bc791b0d538f304a21691c517433` | `796754a8bf947b7c6c579049f94c7cfd0ac0ec03` | `0.2.121` |

The refresh used the old open-sync tree as merge base, the pre-refresh DSB
tree as ours, and the target open-sync tree as theirs. It reconciled 263
upstream-changed paths, 70 pre-refresh DSB overlay paths, and 23 overlaps,
matching the independent supervisor probe. The only textual conflict was the
welcome version badge: DSB branding was retained while the upstream `Beta`
semantics were removed. The five existing DSB patch overlays were mechanically
regenerated against the target context so they remain re-applicable without
changing their owned behavior.

### 0.2.121 adoption matrix

The rows below follow every item in
`crates/codegen/xai-grok-shell/changelogs/0.2.121.md`, including the duplicate
wrapped-error entry. “Adopted” means the source/runtime change is present;
“DSB-specific regression required” identifies behavior that also needs a DSB
regression, rather than relying on the public source note alone.

| # | Changelog item (short form) | Classification |
| ---: | --- | --- |
| 1 | Dashboard rows summarize the previous turn | Adopted |
| 2 | Extensions modal groups/sorts and collapses Skills | Adopted |
| 3 | Background subagent reminder | Adopted |
| 4 | Reattach running session without replay; explicit close | Adopted |
| 5 | No project-directory question from home/non-project cwd | Adopted |
| 6 | Dedicated `/feedback` report box | Adopted |
| 7 | Auto theme detection over SSH/tmux | Adopted |
| 8 | Voice and Finance tool-card icons/labels | xAI-specific/non-applicable |
| 9 | Markdown tables reflow in narrow panes | Adopted |
| 10 | Permission prompts show the complete script | Adopted |
| 11 | Long permission commands expand with Ctrl-F | Adopted |
| 12 | Large MCP image outputs preserve screenshots | DSB-specific regression required |
| 13 | Restored child session after remote parent restore | Adopted |
| 14 | Default branch detection without origin/HEAD | Adopted |
| 15 | Disabled-but-reenableable MCP servers remain visible | Adopted |
| 16 | Rapid send-now preserves earlier queued prompts | Adopted |
| 17 | Esc/stop prevents cancelled background task restart | Adopted |
| 18 | Invalid first-party API key no longer skips login | xAI-specific/non-applicable |
| 19 | Model picker/command palette while reviewing a plan | Adopted |
| 20 | `parallel()` panels bounded by machine parallelism | Adopted |
| 21 | Single-agent dashboard overlay hides useless navigation | Adopted |
| 22 | Pinned prompt headers selectable/copyable | Adopted |
| 23 | Tab/Esc consistent on blocking cards | Adopted |
| 24 | `/new` from empty prompt returns to dashboard | Adopted |
| 25 | Large/shallow repository restore no longer hangs | Adopted |
| 26 | Remote resume restores conversation unless `--restore-code` | Adopted |
| 27 | CJK mouse copy includes edge characters | Adopted |
| 28 | Resume search finds UUID sessions in other directories | Adopted |
| 29 | API errors use clean banners instead of raw JSON | Adopted |
| 30 | Typing `exit`/`quit` exits the dashboard CLI | Adopted |
| 31 | Mode indicator reflects actual resumed/transitioned mode | Adopted |
| 32 | `/delete` returns to dashboard when opened there | Adopted |
| 33 | Enter runs the highlighted slash-menu command | Adopted |
| 34 | Retry more server errors | Adopted runtime logic; xAI branding remains non-product |
| 35 | Wrapped long diff lines highlight correctly | Adopted |
| 36 | Session-required slash commands explain dashboard state | Adopted |
| 37 | CLI exit minimally resets terminal modes | Adopted |
| 38 | Queued prompts remain visible while waiting on subagents | Adopted |
| 39 | Auto recaps avoid middle-of-turn/busy sessions | Adopted |
| 40 | `/btw` errors wrap completely | Adopted |
| 41 | Queued slash commands/images reorder in the queue pane | Adopted |
| 42 | Duplicate `/btw` errors wrap completely | Adopted |
| 43 | `/feedback` preserves composer input mode | Adopted |
| 44 | Forking very large sessions avoids multiplicative memory | Adopted |
| 45 | Exiting an empty session is instant on slow networks | Adopted |

### DSB-specific evidence and boundary audit

- The large-MCP-image path has focused regression coverage for hub
  serialization, text-only harness stripping, multimodal preservation, and
  valid/truncated-image resume validation. Existing storage and bridge tests
  additionally cover persisted assets, OCR/tool guidance, and quarantine of
  poisoned history. The public changelog alone is not treated as proof of the
  large-screenshot fix.
- The refresh retains DeepSeek Build branding and command/configuration
  overlays (`deepseek-build`, `dsb`, `DEEPSEEK_API_KEY`, `api.deepseek.com`,
  and `~/.deepseek-build`). Concrete Path-A evidence is the DeepSeek model
  seed and child environment bridge in `crates/dsb-cli/src/agent_launch.rs`,
  the default-disabled `TelemetryMode` in
  `third_party/grok-build/crates/codegen/xai-grok-telemetry/src/config.rs`, and
  the double opt-in gate in
  `third_party/grok-build/crates/codegen/xai-grok-telemetry/src/external/config.rs`.
  Source-only xAI auth, telemetry, and hosted implementation code may remain
  for compilation, but the static review found no new Path-A enablement of xAI
  login, outbound analytics, or xAI branding.
- Unresolved verification boundary: credential-gated live behavior and
  remotely supplied settings were not executed in this refresh, so a
  deployment that explicitly opts into hosted xAI features still needs its
  own runtime policy review. The local default path remains statically
  DeepSeek-seeded and telemetry-off.
- The upstream tree still contains standalone/source-only Grok labels (for
  example the feedback prompt, billing copy, model labels, and version-mismatch
  restart hint) because those paths are needed by the vendored runtime. They
  are not DSB Path-A product identity, but a complete runtime audit of every
  hosted or alternate invocation remains unresolved; the DSB wrapper's
  DeepSeek invocation, branding, and configuration overlays remain the
  authoritative default path.
- The source refresh is deliberately kept in this research record. Product
  commitments remain governed by the architecture and product specifications.
