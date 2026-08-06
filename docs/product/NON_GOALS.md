# Non-goals (v1)

Things we **will not** optimize for or design into the first product shape.

## Out of product scope

| Non-goal | Why |
|----------|-----|
| Gajae-style multi-stage planning harness | Blocks progress; too slow in practice |
| Being the best multi-vendor router (Claude/GPT first-class) | Dilutes DeepSeek-native tuning |
| Full Grok Build hard-fork on day one | Months of auth/sampler/branding work before value |
| Desktop app MVP | CLI/TUI first |
| VS Code extension MVP | Optional later; Deep Code already has one |
| YOLO / full-auto as default or sole mode | Side-effect permissions are required product quality (Deep Code D). A power-user “yolo” profile, if ever added, is **opt-in**, never default, and still audited |
| Matching star counts or marketing vs Reasonix/Deep Code | Ship speed + cost + quality |

## Out of repo process scope (for now)

- Public npm/binary release pipeline
- Stable public API guarantees
- Accepting drive-by feature PRs without specs

## Revisit triggers

Only reopen a non-goal with a new ADR, e.g.:

- “Minimal post-task verification gate” (not full Gajae stack)
- “Editor ACP integration after CLI is solid”
