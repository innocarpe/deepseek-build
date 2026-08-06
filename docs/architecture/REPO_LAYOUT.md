# Repository layout

**Principle:** directories are product seams. If a concept is first-class in the product, it should have an obvious home. If it is only research, it stays under `docs/research/`.

## Top level

```text
deepseek-build/
├── README.md
├── AGENTS.md                 # Standing instructions for agents
├── LICENSE
├── docs/                     # Product + engineering truth (see docs/README.md)
│   └── architecture/
│       ├── HARNESS_PHILOSOPHY.md   # Normative design spine (Deep Code/Reasonix/Grok layers)
│       └── REPO_LAYOUT.md          # This file
├── crates/                   # Implementation packages (layout inspired by Grok; contracts by L1/L2)
├── skills/                   # Bundled Skills + agent harness skills (e.g. pr-authoring)
├── .deepseek-build/          # Project-local runtime surface (examples, not secrets)
├── scripts/                  # Dev/release automation
└── third_party/              # Notices + vendored ports
```

**Important:** `crates/` layout may follow Grok-style modularity, but **tool/session semantics** follow [HARNESS_PHILOSOPHY.md](./HARNESS_PHILOSOPHY.md), not Grok tool shapes by default.

## Why this shape

### `docs/` not top-level `prds/` or `specs/` alone

- One root for human-readable truth → less drift.
- **product** ≈ PRD/vision layer  
- **specs** ≈ testable behavior  
- **architecture** ≈ structure  
- **adr** ≈ decisions  
- **research** ≈ inputs, non-binding  
- **user-guide** ≈ shipped UX docs (later)

### `crates/` (reserved)

Grok Build is the primary code-structure reference: many focused packages under a workspace-like tree. We reserve `crates/` even before the language is locked so the map stays stable.

When implementation starts, expected seams (names TBD via ADR):

| Future package (illustrative) | Product concern |
|-------------------------------|-----------------|
| `…-cli` / binary | Entry / TUI composition root |
| `…-agent` | Turn loop, prompts |
| `…-provider-deepseek` | API, thinking, effort, cache headers |
| `…-tools` | read/edit/bash/grep/… |
| `…-orchestrator` | subagents, parallel wait, workflows |
| `…-config` | settings, project rules |

Language choice (Rust vs Go vs other) is **not** locked by this folder name; if we choose Go, an ADR may rename `crates/` → `internal/` + `cmd/`. Until then, `crates/` is the placeholder.

### `skills/`

Deep Code / Agent Skills ecosystem: bundled `SKILL.md` packages the product ships.

### `.deepseek-build/`

Project-facing config surface (like `.deepcode/`, `.grok/`, `.reasonix` project files):

```text
.deepseek-build/
├── skills/       # project skills
├── agents/       # agent definitions (later)
└── workflows/    # orchestration scripts (later)
```

Committed content here is **examples or project policy**, never API keys.

### `third_party/`

Any ported snippets from Grok / Reasonix / Deep Code need license notices. Do not silently copy.

## Intentionally absent (for now)

| Path | Status |
|------|--------|
| `packages/` (JS monorepo) | Not assumed |
| `prds/` top-level | Use `docs/product/` |
| `src/` single blob | Avoid until language ADR |
| `apps/desktop` | Non-goal v1 |

## Change control

Adding a **new top-level directory** requires an ADR under `docs/adr/`.
