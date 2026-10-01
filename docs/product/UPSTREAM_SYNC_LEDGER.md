# Upstream sync ledger — Grok Build

**Purpose:** what every sync of `third_party/grok-build/` found and decided, so
the next session inherits the judgment instead of redoing the reading.

**How to add a row:** `skills/grok-sync` §5. Record what was measured, not what
was hoped; a sync with an honest *held* list is worth more than one that claims
everything landed.

Companions: [grok-sync-runbook](../contributing/grok-sync-runbook.md) ·
[GROK_VENDOR.md](../architecture/GROK_VENDOR.md) §Refresh ·
[research/grok-build.md](../research/grok-build.md) (the per-release matrix).

---

## Sync 1 — `1.0.0` (initial vendor land, 2026-08-09)

| Field | Value |
|---|---|
| Pin before | — (no vendor tree) |
| Pin after | `27b3c666…` (upstream `8a14c91d`) |
| Upstream version | `1.0.0` |
| Method | initial vendor land + 13-patch overlay series |
| PRs | #173 (port completion), #177 / #179 (sync) |
| Gates | owner-bar green through `5.x` |

Established the layout for that pin: vendored tree at
`third_party/grok-build/`, `SOURCE_REV` pin, overlay carried partly in-tree and
partly as `patches/grok-build/`. Sync 2 is what holds after the port: the
overlay is in the tree and `patches/grok-build/` has no patch files (git
drops the directory when the last one is deleted; the apply script treats
that as exit 0).

---

## Sync 2 — `1.0.0` → `1.0.41` (2026-09-25)

| Field | Value |
|---|---|
| Pin before | `27b3c666…` (upstream `8a14c91d`, `1.0.0`) |
| Pin after | `f0e3be11…` (upstream `1.0.41`) |
| Releases covered | **41** (`1.0.1` … `1.0.41`, 2026-08-10 → 2026-09-22) |
| Gap | 25 commits, 3,587 files, +750,612 / −383,434 |
| Changelog bullets | **472** (36 Performance, 5 Breaking sections, 19 xAI-only) |
| Workspace crates | 81 → 102 |
| Method | **three-way merge** — the 13-patch series conflicted on all 13 files |
| Target release | dsb **`6.0.0`** |

### Why the method changed

The old procedure (`rsync --delete` + `apply-grok-build-patches.sh`) was
measured against `1.0.41` and cannot work: **0 of 13 patches applied**, and the
overlay had grown to 134 files (5.3k changed + 3.3k new lines) with direct
in-tree edits the patch series does not carry. A raw swap would have silently
dropped the DeepSeek status line, the theme skins, and the prompt identity.

The merge roles were: base `afbc0fb` (upstream `1.0.0`) · ours = the vendored
tree · theirs = `f0e3be11` (`1.0.41`).

### Cluster inventory of the range

| Cluster | Bullets | Notes |
|---|---:|---|
| TUI / pager UX | 114 | largest visible win |
| Sessions | 61 | resume speed, crash markers, headless |
| Subagents | 41 | bounded spawning, lifecycle ordering |
| Skills / plugins / workflows | 38 | workflows catalog, memory modal |
| Performance | 36 | startup, git on large repos, subagent spawn |
| Tools | 34 | edit line numbers, images across compaction |
| Model plumbing | 34 | per-effort model ids, effort from API, retries |
| Permissions / sandbox / hooks | 33 | never-allow grants, hook confirm/defer |
| MCP | 32 | elicitation, OAuth, policy blocks |
| Worktrees / git | 16 | `grok clone` content store, reclamation |
| xAI-only (N/A) | 19 | telemetry, billing, voice, video, xAI login |
| Config / policy | 9 | `GROK_CONFIG*` overrides, status-line timer |
| **Breaking** | **5** | see below |

### The five breaking changes

| Version | Change | Verdict |
|---|---|---|
| 1.0.1 | `/rewind` truncates conversation only | inherited |
| 1.0.1 | Managed MCP servers only via gateway catalog | **N/A** — xAI-hosted |
| 1.0.6 | `spawn_subagent` drops `capability_mode` | **impact** — dsb spec 60 / harness brief reference it |
| 1.0.16 | Enterprise `requirements.toml` model restrictions | **N/A** — xAI-hosted |
| 1.0.19 | Scheduled `/loop` tasks always background | inherited |

### Overlay carried through the merge

sampling types (DeepSeek cache mapping) · shell extension
(`x.ai/deepseek/status`) · status-line balance row · prompt identity (no
third-party vendor claim; encrypted copy regenerated) · seeded product
changelog preservation · DeepSeek theme skins and their registrations ·
Path A cache signal + spec-10 assembly helpers · invocation branding.

### Patch directory

`844c7be` removed `0001`–`0013` after none of them re-applied onto `1.0.41`.
`0014` (OSC 9999 agent status) was cut in the same window and was the only
file left under `patches/grok-build/`. It applies neither forward nor reverse:
`agent_status.rs` is already in the tree, and `mod.rs` has moved past the
hunk. The vendor-patch-gate change deletes `0014` and does not edit those
sources. A later refresh preserves the overlay by the three-way merge
(runbook §2) checked against the overlay table in `GROK_VENDOR.md`, not by
re-applying a series.

### Gates

Recorded in the sync PR. (Fill from the merged PR; do not mark green here for a
gate that did not run.)

### Owner-facing artifact

[`CHANGELIST_6_0_0.md`](../product/CHANGELIST_6_0_0.md) — the single file
answering "what is better in `6.0.0`, and why".

---

## Sync 3 — `1.0.41` → `1.0.45` (2026-10-01)

| Field | Value |
|---|---|
| Pin before | `036a5d8348cd744767cd0b08518ab17bf608fa7f` (upstream git `f0e3be11`, `1.0.41`) |
| Pin after | `559751fdcec02d413e4c57c8832ab275e4f44980` (upstream git `2bdd1d6a`, `1.0.45`) |
| Releases covered | **4** (`1.0.42` … `1.0.45`, 2026-09-26 → 2026-09-29) |
| Gap | 2 commits, 991 files, +129,569 / −27,951 |
| Changelog bullets | 22 features/fixes, 1 performance, 0 breaking sections. N/A: smart-auto served-model label, WinGet `grok update` |
| Method | **three-way merge** — base `f0e3be11`, ours = vendored tree, theirs = `2bdd1d6a`. 21 content conflicts |
| Target release | none in this sync. The pin moves; product SemVer is unchanged |

### Why this method

The overlay is still in the tree (`patches/grok-build/` is empty). A raw swap would drop version injection, announcement stripping, the cache-session log, and the phone layout. The merge kept those and took upstream's edits on the same files.

### Verdicts

| Bullet | Verdict |
|---|---|
| Bracketed-paste leftover images, `-p` interrupt cleanup, Ctrl+C on plan comments, `--minimal` resize, stashed-draft caret, image repaint, plugin hook reload, marketplace URL normalization | **Take** |
| MCP prompts in minimal mode; plan-comment delete; shortcuts search; Windows paste crash; same-file edits run in sequence | **Take** |
| Seatbelt sandbox backend; optional context-window sizes and `/context-window`; custom agents on `spawn_subagent`; MCP token file; model notice banner; subagent wait count | **Take** — they ride the vendor tree. Context-window choices appear only when the model advertises them |
| `grok worktree create` | **Take** as upstream CLI behavior inside the vendor tree. This product's worktrees stay the Orca path |
| Auto permission mode labeled Auto-review | **Take** |
| Footer shows the model smart-auto actually served | **N/A** — xAI smart-auto routing |
| `grok update` prints `winget upgrade` and does not write files | **N/A** — grok self-update channel |

No breaking-change section in `1.0.42`–`1.0.45`.

### Overlay carried through the conflicts

Product version injection (`DEEPSEEK_BUILD_VERSION`, sccache file) · `strip_remote_announcements` on the new `settings()` accessor · Spec 10 cache-session log beside `cancellation_meta` · phone welcome height (frame grid plus upstream `notice_rows`) · prompt caret clamp plus upstream interim caret · tasks-pane wide-glyph background · turn-status spinner shift · text-selection default `hold` · `request_log_invariant` kept next to upstream `rewind_preview`.

### Other upstreams measured the same day, not vendored

Deep Code, Reasonix, and DeepSeek Harness are design sources, not trees under `third_party/`. Heads on 2026-10-01:

| Source | Previous read | Head now | What moved |
|---|---|---|---|
| Deep Code CLI `lessweb/deepcode-cli` | sweep 2026-09-25, no SHA pin; latest release then was `v0.4.1` (2026-09-17) | `f1d9909d`, release `v0.4.2` (2026-09-28) | PLUS subscription host/key, slash-command prefix ordering, `undici` bump |
| Reasonix `esengine/DeepSeek-Reasonix` | sweep 2026-09-25, shallow, no SHA | `c9daeddf` | 884 commits since 2026-09-25; the sample is Electron studio, edit-menu locale, plugin export |
| DeepSeek Harness `deepseek-ai/deepseek-harness` | `477b4f4` = `0.1.7-rc.2` | `639ed015` = `dsh-v0.2.0-rc.2` | 78 first-parent merges: desktop CLI, Windows ACL, sidebar, plugins |

None of those commits is a pin this repo updates by copying the tree. PLUS routing, the Electron studio, and the desktop harness are already in the leave column of `SOURCES.md`. No spec change follows from them in this sync.

### Gates (commands + real outcome)

| Command | Outcome |
|---|---|
| `./scripts/build-grok-pager.sh check` | exit 0. First run 2026-10-01T07:35:17Z, `Finished dev` in 4m 27s, with one `unused_assignments` warning in `cancel.rs` (a leftover snapshot binding). After aligning that call with upstream `1.0.45`, the rerun at 2026-10-01T07:38:38Z finished in 41.97s, exit 0, no warning. `SOURCE_REV=559751fdcec02d413e4c57c8832ab275e4f44980`, product version injected `6.9.0`. |
| `./scripts/test-owner-bar.sh` | not run |
| `./scripts/check-path-a-linkage.sh` | not run |
| `./scripts/test-heart-regression.sh` | not run |
| `./scripts/test-grok-vendor-offline.sh` | not run |
| `./scripts/test-product-offline.sh` | not run |
| `./scripts/check-semver.sh` | not run |

A gate that did not run is not green. Product SemVer is not part of this sync.

### Owner-facing artifact

[`CHANGELIST_GROK_1_0_45.md`](CHANGELIST_GROK_1_0_45.md).

---

## Template for the next sync

```markdown
## Sync N — `<old>` → `<new>` (YYYY-MM-DD)

| Field | Value |
|---|---|
| Pin before / after | … |
| Releases covered | … |
| Gap | commits / files / lines |
| Changelog bullets | total, with breaking and xAI-only counts |
| Method | patch re-apply \| three-way merge |
| Target release | dsb `X.Y.Z` |

### Why this method
### Cluster inventory
### Breaking changes and verdicts
### Overlay carried
### Held / rejected, with reasons
### Gates (commands + real outcome)
### Owner-facing artifact
```
