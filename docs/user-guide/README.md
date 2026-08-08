# User guide

Shipped and on-branch user-facing behavior is documented here. Intent for
unshipped features stays in `docs/specs/`.
**Known limits:** [../product/KNOWN_LIMITS.md](../product/KNOWN_LIMITS.md) ·
**Changelog:** [../../CHANGELOG.md](../../CHANGELOG.md)

## Commands (dual naming)

| Command | Role |
|---------|------|
| **`deepseek-build`** | Primary public CLI |
| **`dsb`** | Short alias — same binary behavior |

Both must print the **same** full SemVer (`MAJOR.MINOR.PATCH`) from product
packaging. Config/home stays under `~/.deepseek-build/` (path ≠ command name).

**Version honesty:** read root `Cargo.toml` / `package.json` in the tree you
built from. Live `main` / npm / GitHub Release may lag an unmerged vision stack
(see [KNOWN_LIMITS](../product/KNOWN_LIMITS.md)).

## Guides

0. **[First-run setup](./00-setup.md)** — API key onboarding (**required** before chat)
1. **[Install](./01-install.md)** — PATH install (`deepseek-build` / `dsb`)
2. **[Dogfood profile](./02-dogfood-profile.md)** — `--dogfood` trusted local write + bash
3. **[Sessions](./03-sessions.md)** — persist/resume JSONL + TUI resume
4. **[Surface](./04-surface.md)** — skills index + thinking/effort + cache signal
5. **[npm](./05-npm.md)** — `@innocarpe/deepseek-build` dual bins (prebuilt)
6. **[Authentication](./06-auth.md)** — load order, login/status/logout
7. **[Chat and run](./07-chat-run.md)** — REPL, one-shot, flags
8. **[Permissions](./08-permissions.md)** — ask once/always, fail-closed
9. **[Theme](./09-theme.md)** — DeepSeek Night product themes
10. **[Tools](./10-tools.md)** — Path A snippet-safe tools vs thin overlay
11. **[Subagents](./11-subagents.md)** — full-screen agent child sessions (L3)
12. **[Background tasks](./12-background-tasks.md)** — bg shell / collect-by-id (L3)
13. **[Worktrees](./13-worktrees.md)** — opt-in git worktree sessions (L3)
14. **[Throughput overview](./14-l3-throughput.md)** — L3 map + smoke / Path A R0A

## Layer map (Path A)

Default product path is public CLI → full-screen agent (**Path A**):

| Layer | What users feel | Honesty pointer |
|-------|-----------------|-----------------|
| **L1 Deep Code** | Snippet-scoped edit (`snippet_id`), create-only write, bash invalidation | [10-tools.md](./10-tools.md) |
| **L2 Reasonix** | Stable prefix assembly, `reasoning_effort` on DeepSeek wire, cache % signal | [04-surface.md](./04-surface.md) |
| **L3 Grok** | Parallel read tools, background shell, subagents, opt-in worktree | [14-l3-throughput.md](./14-l3-throughput.md) |

Thin line-mode tools (`dsb run` / `dsb-tools`) are a **different surface** — tool
names and proofs differ; do not treat thin unit greens as sole Path A proof.

## Quick start

```bash
# Registry install (prebuilt natives on darwin-arm64; no Rust on default path)
npm install -g @innocarpe/deepseek-build

deepseek-build setup                       # paste API key → credentials.json
deepseek-build --dogfood --session demo    # bare TTY = full-screen agent
# same:
dsb --dogfood --session demo
```

Offline L3 help smoke (installed agent, no vendor test build):

```bash
./scripts/test-l3-smoke.sh --offline-only
```

Public-entry Path A hermetic dogfood scripts (dev checkout; scripted wire):

```bash
./scripts/test-path-a-vc010-r0a.sh   # multi-tool + background
./scripts/test-path-a-vc011-r0a.sh   # subagent + worker cache stamp
./scripts/test-path-a-vc012-r0a.sh   # worktree CLI / opt-in / headless honesty
```
