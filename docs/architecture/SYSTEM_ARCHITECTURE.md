# System architecture — DeepSeek Build

**Status:** Living design (implementation may lag; specs + ADRs win on conflict)  
**Spine:** [HARNESS_PHILOSOPHY.md](./HARNESS_PHILOSOPHY.md)  
**Roadmap:** [MASTER_PLAN.md](../product/MASTER_PLAN.md)

---

## 1. One-paragraph overview

DeepSeek Build is a **local-first CLI agent**. The user invokes **`deepseek-build`** or **`dsb`**. The process loads config and credentials, builds a **cache-stable message prefix** plus a **volatile turn tail**, calls **DeepSeek Chat Completions** (Flash by default, Pro on escalate), optionally runs **tools** under **snippet** and **permission** rules, and streams **reasoning** and **content** separately to the terminal (themeable UI).

---

## 2. Context diagram

```mermaid
flowchart TB
  User[User terminal]
  CLI["deepseek-build / dsb"]
  Home["~/.deepseek-build/\ncredentials, config, sessions"]
  Proj["Project tree\nAGENTS.md, .deepseek-build/"]
  API["DeepSeek API\napi.deepseek.com"]

  User --> CLI
  CLI --> Home
  CLI --> Proj
  CLI --> API
```

---

## 3. Process / crate architecture

```mermaid
flowchart LR
  subgraph bins["Binaries"]
    B1[deepseek-build]
    B2[dsb]
  end

  subgraph crates["Cargo workspace"]
    CLI[dsb-cli]
    AG[dsb-agent]
    PR[dsb-provider-deepseek]
    CX[dsb-context]
    TL[dsb-tools]
    CF[dsb-config]
  end

  B1 --> CLI
  B2 --> CLI
  CLI --> AG
  CLI --> CF
  AG --> PR
  AG --> CX
  AG --> TL
  CX --> PR
  TL --> PR
  CF --> Home[(user home)]
```

| Crate | Responsibility |
|-------|----------------|
| `dsb-cli` | argv, REPL, install surface, theme I/O later |
| `dsb-config` | `DEEPSEEK_API_KEY`, credentials file, home root |
| `dsb-provider-deepseek` | HTTP/SSE, models, thinking wire, usage/cache |
| `dsb-context` | Stable prefix builder, epochs, project instructions |
| `dsb-agent` | Turn loop, routing, repair, tool dispatch |
| `dsb-tools` | Snippets (45), permissions (90), read/edit/write/bash/search… |

---

## 4. Request pipeline (single turn)

```mermaid
sequenceDiagram
  participant U as User
  participant CLI as dsb-cli
  participant AG as dsb-agent
  participant CX as dsb-context
  participant R as ModelRouter
  participant P as provider
  participant T as dsb-tools
  participant API as DeepSeek API

  U->>CLI: message / chat line
  CLI->>AG: run_turn
  AG->>R: route Flash or Pro
  AG->>CX: assemble stable_prefix + volatile_tail
  AG->>P: ChatRequest stream + thinking + effort
  P->>API: POST /chat/completions
  API-->>P: SSE deltas
  P-->>AG: reasoning / content / tool_calls
  alt tool_calls present
    AG->>AG: repair args (spec 15)
    AG->>T: execute under permissions
    T-->>AG: tool results
    Note over AG,API: re-call with reasoning_content if tools in play
  end
  AG-->>CLI: stream + model visibility
  CLI-->>U: terminal render themed
```

---

## 5. Cache contract (L2)

```mermaid
flowchart TB
  subgraph stable["Stable prefix — byte-stable across turns"]
    S1[System template]
    S2[Tool schemas canonical JSON]
    S3[Skills index only]
    S4[Env summary small]
    S5[Project instructions]
  end

  subgraph volatile["Volatile tail"]
    V1[User turn]
    V2[Assistant + tool chain]
    V3[Dynamic reminders]
    V4[Large tool outputs]
  end

  stable --> API[API messages array]
  volatile --> API
```

- Epoch = SHA-256 of stable prefix bytes (`dsb-context`).  
- Tool schema / skills index change → new epoch (expected).  
- Snippet table is **session state**, **not** in stable prefix.

---

## 6. Tools + permissions + snippets (L1)

```mermaid
stateDiagram-v2
  [*] --> Read: read tool
  Read --> SnippetIssued: snippet_id + version
  SnippetIssued --> Edit: edit with snippet_id
  Edit --> VersionCheck
  VersionCheck --> Applied: match unique in scope
  VersionCheck --> Stale: file changed
  VersionCheck --> Ambiguous: multi match
  Applied --> [*]: expire path snippets
  Stale --> [*]
  Ambiguous --> [*]

  note right of Edit
    write: create-only by default
    bash: declare + classifier
    permission decide allow/deny/ask
  end note
```

```mermaid
flowchart LR
  Cmd[bash command] --> Decl[declared side_effects advisory]
  Cmd --> Cls[static classifier authoritative]
  Decl --> Merge[fail-closed to more dangerous]
  Cls --> Merge
  Merge --> Pol[policy allow deny ask]
  Pol -->|allow| Exec[execute or dry-run]
  Pol -->|deny ask headless| Stop[error to model]
  Exec -->|mutating| Exp[expire snippets]
```

---

## 7. Model routing (L2)

```mermaid
flowchart TD
  In[User text + slash commands] --> P{precedence}
  P -->|1| U[User /pro /flash /preset]
  P -->|2| Sticky[Session preset max or flash]
  P -->|3| Auto[Optional keyword escalate]
  P -->|4| Def[Default Flash]
  U --> Out[RouteDecision wire model + effort]
  Sticky --> Out
  Auto --> Out
  Def --> Out
  Out --> Vis[Always show model= to user]
```

Wire IDs: `deepseek-v4-flash`, `deepseek-v4-pro` (ADR 0005).

---

## 8. Target architecture (Waves B–C) — not all built yet

```mermaid
flowchart TB
  subgraph parent["Parent agent"]
    Loop[Turn loop]
    Router[Model router]
    Tools[Tool runtime]
    Perm[Permissions]
    Theme[Theme engine]
  end

  subgraph workers["Wave C — subagents"]
    Ex[Explore worker Flash]
    Im[Implement worker]
    WT[Optional git worktree]
  end

  Loop --> Tools
  Loop --> Router
  Tools --> Perm
  Loop -.->|spawn Wave C| Ex
  Loop -.->|spawn Wave C| Im
  Im --> WT
  Ex --> CacheLaw[Shared stable template Flash default]
  Im --> CacheLaw
```

**Worker cache law:** children reuse stable prefix templates; no unique cold system dumps; Flash-default workers.

---

## 9. Packaging (Waves A / D)

```mermaid
flowchart LR
  Src[crates/dsb-cli] --> CargoInstall[cargo install path]
  Src --> Rel[target/release dual bins]
  Rel --> Script[scripts/install.sh]
  Src --> Npm[npm package Wave A 0.7.0]
  Npm --> Bin1[bin deepseek-build]
  Npm --> Bin2[bin dsb]
  Script --> PATH[PATH]
  CargoInstall --> PATH
  Bin1 --> PATH
  Bin2 --> PATH
```

Version: single SemVer in workspace (+ npm match when published).

---

## 10. Trust boundaries

| Boundary | Rule |
|----------|------|
| Secrets | Env or `~/.deepseek-build/credentials.json` mode 0600; never project tree |
| Workspace vs out-of-cwd | Path scopes; default deny write/delete outside |
| Shell | Classifier authoritative; unknown → ask/deny |
| Model output | Never execute unparsed tool args; repair once then error |
| Theme | UX only; no security boundary |

---

## 11. Open design items

| Topic | Track |
|-------|--------|
| TUI stack (ratatui vs rich ANSI CLI) | Wave B theme |
| Session store schema JSONL | Wave A `0.5.0` |
| Parallel tool scheduler | Wave C / spec 50 |
| Subagent IPC | Wave C / spec 60 |
| Prebuilt multi-platform npm optionalDeps | post-`0.7.0` / ADR after 0007 source-assisted strategy |

---

## 12. Heart 3.x (Path A vs Path B)

2.x ships the **Grok-derived full-screen agent** as the default `dsb` entry (Path A).  
Thin crates (`dsb-agent` / `dsb-tools` / `dsb-context`) still own the **reference** Spec 45/90/10/15/20 implementations (Path B) but are **not** the default TUI tool path.

**Normative binding for 3.0.0:** [HEART_3X_SPEC_BINDING.md](./HEART_3X_SPEC_BINDING.md)  
**P0 red→green cases:** [HEART_3X_P0_TEST_PLAN.md](../product/HEART_3X_P0_TEST_PLAN.md)

Until Path A enforces L1/L2 hearts, do not claim heart fusion complete ([PRD-v3.md](../product/PRD-v3.md)).

---

## 13. References

- ADR 0004 toolchain · 0005 provider · 0006 CLI names + SemVer  
- Specs 10, 15, 20, 30, 45, 90 (+ 40/50/60/70/80 later)  
- Heart 3.x binding · test plan (links above)  
