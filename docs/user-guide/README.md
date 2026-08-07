# User guide

Shipped user-facing behavior is documented here. Intent for unshipped features stays in `docs/specs/`.  
**Known limits:** [../product/KNOWN_LIMITS.md](../product/KNOWN_LIMITS.md) · **Changelog:** [../../CHANGELOG.md](../../CHANGELOG.md)

## Guides

0. **[First-run setup](./00-setup.md)** — API key onboarding (**required** before chat)
1. **[Install](./01-install.md)** — PATH install (`deepseek-build` / `dsb`)
2. **[Dogfood profile](./02-dogfood-profile.md)** — `--dogfood` trusted local write + bash
3. **[Sessions](./03-sessions.md)** — persist/resume JSONL
4. **[Surface](./04-surface.md)** — skills index + thinking/effort
5. **[npm](./05-npm.md)** — `@innocarpe/deepseek-build` dual bins
6. **[Authentication](./06-auth.md)** — load order, login/status/logout
7. **[Chat and run](./07-chat-run.md)** — REPL, one-shot, flags
8. **[Permissions](./08-permissions.md)** — ask once/always, fail-closed
9. **[Theme](./09-theme.md)** — DeepSeek blue default
10. **[Tools](./10-tools.md)** — thin-path built-ins (overlay `dsb-tools`)
11. **[Subagents](./11-subagents.md)** — full-screen agent child sessions (L3)
12. **[Background tasks](./12-background-tasks.md)** — bg shell / task output (L3)
13. **[Worktrees](./13-worktrees.md)** — git worktree sessions (L3)
14. **[Throughput overview](./14-l3-throughput.md)** — L3 map + smoke commands

**L3 smoke (installed agent, no vendor test build):** `./scripts/test-l3-smoke.sh`

## Quick start

```bash
npm install -g @innocarpe/deepseek-build   # needs Rust once for postinstall
deepseek-build setup                       # paste API key → credentials.json
deepseek-build --dogfood --session demo chat
```

Both `deepseek-build` and `dsb` must report the **same** full SemVer from `Cargo.toml` / `package.json`.
