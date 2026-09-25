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

Established the layout that still holds: vendored tree at
`third_party/grok-build/`, `SOURCE_REV` pin, overlay carried partly in-tree and
partly as `patches/grok-build/`.

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

### Gates

Recorded in the sync PR. (Fill from the merged PR; do not mark green here for a
gate that did not run.)

### Owner-facing artifact

[`CHANGELIST_6_0_0.md`](../product/CHANGELIST_6_0_0.md) — the single file
answering "what is better in `6.0.0`, and why".

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
