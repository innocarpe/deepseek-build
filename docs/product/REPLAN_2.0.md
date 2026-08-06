# Replan — DeepSeek Build **2.0.0** (Grok Build base)

**Status:** Normative product replan (2026-08-06)  
**Supersedes for product direction:** prior overnight interpretation of Waves A–D as “done 1.0.0 product”  
**Does not delete:** 1.x code, specs, tests, or published packages (historical scaffold)

**Related**

| Doc | Role |
|-----|------|
| [GROKBASE_2X_GOALS.md](./GROKBASE_2X_GOALS.md) | **One-plate ultragoal story board** (G001–G012 → 2.0.0) |
| [ULTRAGOAL_PROMPT_COLD_START_2.0.md](./ULTRAGOAL_PROMPT_COLD_START_2.0.md) | Paste-ready cold start until 12/12 |
| [ULTRAGOAL_BRIEF_2.0.md](./ULTRAGOAL_BRIEF_2.0.md) | Mission brief for `create-goals` / overnight |
| [WAVE_2x_PR_DAG.md](./WAVE_2x_PR_DAG.md) | Fixed PR units (W0–W4) — no overnight invention |
| [ULTRAGOAL_CHAIN.md](./ULTRAGOAL_CHAIN.md) | Active plan = `grokbase-2x` only |
| [versioning.md](../contributing/versioning.md) | SemVer bands after replan |

---

## 0. Why this document exists

### Owner intent (restated, fail-close)

1. **Product category** = Claude Code / Codex CLI / **Grok Build** class  
   → terminal **coding agent** (marketed as “CLI”, but Grok is **full-screen TUI + agent runtime**).
2. **Base** = open-source **Grok Build** (`grok` / `xai-grok-pager` tree).  
   Owner judgment: best experience + OSS so DeepSeek can fork/adapt it.
3. **Overlay** = Deep Code (L1 contracts) + Reasonix (L2 cache/cost) strengths.
4. **Typing `dsb` / `deepseek-build`** must open **that agent immediately**.  
   That was the **original meaning of 1.0.0**.

### What actually happened (1.x)

| Claimed | Reality |
|---------|---------|
| “1.0.0 shipped” | Greenfield `dsb-*` scaffold + docs/gates train |
| “Grok-class” | MVP parallel/bg/subagent heuristics, **not** Grok runtime |
| “dsb opens agent” | Subcommand clap → thin REPL; **not** Grok TUI |
| Onboarding | Late (1.1.0); not the core product |

**Judgment:** 1.x is a **useful research / contract scaffold**. It is **not** the product the owner ordered.  
**Action:** keep 1.x code and npm history; **re-version product success to `2.0.0`**.

---

## 1. Versioning reset (already published — no history rewrite)

npm / git already have **`1.0.0`** and **`1.1.0`**. We **do not** unpublish or force-delete.

| Line | Meaning (from now) |
|------|---------------------|
| **`1.0.0` – `1.x.y`** | **Legacy scaffold line** — clap agent, contracts, tests, npm package that works as a thin agent. **Not** Grok-base product. |
| **`2.0.0`** | **First real product** matching owner intent: Grok Build–class agent UX, Grok tree as base, DeepSeek native. |
| **`2.0.0-alpha.*` / `2.0.0-beta.*`** | Previews while integrating Grok base. |
| **`2.x.y` (after 2.0.0)** | Compatible evolution of the real product line. |

### SemVer honesty

- **2.0.0 is a major** because the **user-facing product identity changes** (entry experience, runtime base), not because we “finished more checklists.”
- 1.x remains installable for experiments; README must say **scaffold / not the Grok-base product**.
- Tags: `v2.0.0` only when **§2 DoD P0** is green.
- Optional 1.x patch releases only for critical bugs / install; **no new product features on thin REPL** (see §5 freeze).

### Messaging (npm / GitHub)

| Surface | Text |
|---------|------|
| README badge / status | “**1.x = scaffold.** Target product is **2.0.0** (Grok Build base).” |
| `package.json` description (at 2.0 cut) | DeepSeek-native agent on Grok Build runtime |
| 1.x release notes | “Legacy scaffold; prefer 2.x for product intent” |
| GitHub About / topics | Prefer “coding agent” language; avoid “1.0 complete product” |

---

## 2. Product definition of done — **`2.0.0`**

`2.0.0` is **not** “all specs green on paper.” It is:

### Must (P0) — ship blockers

1. **`dsb` / `deepseek-build` with no args** launches a **Grok Build–class full-screen coding agent** (TUI + agent loop), not a subcommand help line and not a bare `❯` line REPL as the only UX.
2. **Base runtime** is **derived from Grok Build open source** (fork, workspace vendor, or monorepo extract — see §4). Not a from-scratch reimplementation of “Grok vibes.”
3. **DeepSeek** is the **default model provider** (API key / setup onboarding works on first run).
4. **L1 minimum still true under that shell:** snippet-safe edit path, permission fail-closed, no YOLO-only default.
5. **L2 minimum still true:** stable tool/system prefix discipline (or documented equivalent in the Grok context stack with tests).
6. **Install story boring enough to dogfood:** binary or npm path that lands `dsb` on PATH and opens the agent.

### Should (P1) — same major if ready

7. Skills index vs body (Deep Code) without thrashing prefix.  
8. Flash-first / Pro escalate (or DeepSeek model routing equivalent).  
9. Parallel tools / bg shell / subagents **via Grok’s real mechanisms**, not only dsb MVP shims.  
10. Theme: readable DeepSeek blue accents in the TUI (not Grok-black-only default).

### Explicit non-goals for **2.0.0**

- Multi-vendor “works equally on Claude/GPT” as identity  
- Gajae multi-stage planning harness as core loop  
- Perfect 1:1 pixel clone of every Grok screen  
- Deleting 1.x from npm history  
- Extending the 1.x thin REPL as the “real product” path  

---

## 3. Architecture plan

```text
┌────────────────────────────────────────────────────────────┐
│  bins: deepseek-build / dsb                                │
│  → entry = Grok pager/TUI composition root (renamed)       │
└───────────────────────────┬────────────────────────────────┘
                            │
        ┌───────────────────┼───────────────────┐
        ▼                   ▼                   ▼
   Grok agent runtime   Grok TUI/pager    Grok tools/shell
   (xai-grok-agent…)    (xai-grok-pager)  (native speed)
        │
        ▼
   DeepSeek provider adapter (from dsb-provider-deepseek + auth)
        │
        ▼
   L1/L2 policy overlays (snippet, permissions, cache epoch)
   ← port tests/contracts from 1.x dsb-tools / dsb-context
```

### Grok Build reference map (local sibling `../grok-build`)

Upstream is **Apache-2.0** (SpaceXAI). Pin via `SOURCE_REV` when vendoring/forking.

| Area | Crates (examples) | 2.0 relevance |
|------|-------------------|---------------|
| **Entry / TUI** | `xai-grok-pager-bin`, `xai-grok-pager`, `xai-grok-pager-render` | Default `dsb` entry = this class of binary |
| **Agent loop** | `xai-grok-agent`, `xai-agent-lifecycle`, `xai-chat-state` | Real agent runtime (replace thin REPL) |
| **Auth / HTTP** | `xai-grok-auth`, `xai-grok-http`, `xai-grok-secrets` | Plug DeepSeek keys / base URL |
| **Config / models** | `xai-grok-config`, `xai-grok-models`, `xai-grok-env` | Default model = DeepSeek |
| **Tools / shell** | `xai-grok-tools`, `xai-grok-shell*`, `xai-grok-sandbox` | L1 overlay injection points |
| **MCP / skills-ish** | `xai-grok-mcp`, hooks/plugins crates | Prefer Grok surfaces over static catalog |
| **Subagent / worktree** | `xai-grok-subagent-resolution`, `xai-fast-worktree` | Real L3 (not 1.x in-process shims) |
| **Upstream pin** | `SOURCE_REV`, `LICENSE`, `THIRD-PARTY-NOTICES` | ADR + NOTICE obligations |

Build smoke (W0 must document pass/fail on agent machine):

```sh
# From ../grok-build (requires rustup + dotslash/protoc per their README)
cargo check -p xai-grok-pager-bin
# optional interactive: cargo run -p xai-grok-pager-bin
```

### Reuse from 1.x (keep — do not throw away)

| 1.x asset | Role in 2.0 |
|-----------|-------------|
| `dsb-provider-deepseek` | DeepSeek API client, SSE, models |
| `dsb-config` credentials + setup | First-run key onboarding (`~/.deepseek-build/…`) |
| Specs 10/15/20/30/40/45/90/… | Contract tests / acceptance |
| `dsb-tools` snippet/permissions | Policy layer or port into Grok tool path |
| `dsb-context` prefix/epoch | L2 tests / adapter |
| CI required check | Keep product CI culture |
| Docs user-guide structure | Rewrite against real TUI |

### Likely deprecate as *product entry* (keep as lib/tests)

| 1.x | Note |
|-----|------|
| clap thin REPL as default UX | Secondary: `dsb repl-legacy` or drop from default |
| In-process subagent heuristics | Prefer Grok’s real subagent/worktree |
| Static-only MCP catalog | Prefer Grok MCP surface + DeepSeek auth |

---

## 4. Base integration strategies (pick one early ADR)

| Strategy | Pros | Cons | Recommendation |
|----------|------|------|----------------|
| **A. Fork grok-build → deepseek-build 2.x tree** | Cleanest “base is Grok” | License/sync cost; large tree; history rewrite of *this* repo | **Default preferred** if operationally OK |
| **B. Git subtree/submodule of grok-build** | Clear upstream pin; keeps 1.x crates as overlay package | Dual-root / CI complexity | Strong alternative |
| **C. Cargo path deps to sibling `../grok-build`** | Fast local | Bad for npm/release | **Spike only (W0)** |
| **D. Continue greenfield** | — | **Fails owner intent** | **Rejected** for 2.0 |

**First ADR for 2.0:** `docs/adr/0008-grok-build-base.md` (number next free) choosing A/B, Apache-2.0 attribution, `SOURCE_REV` pin, and how `dsb` binary is produced.

---

## 5. Execution waves (2.x train)

Plan id: **`grokbase-2x`** (ultragoal).  
Fixed PR units: **[WAVE_2x_PR_DAG.md](./WAVE_2x_PR_DAG.md)**.

| Wave | SemVer band | Outcome |
|------|-------------|---------|
| **W0 Research** | docs / optional `2.0.0-alpha.0` | Map crates; auth/provider plug points; license; `cargo check` TUI bin; ADR draft |
| **W1 Shell** | `2.0.0-alpha.N` | `dsb` launches Grok TUI composition root; branding DeepSeek Build; setup/auth path exists |
| **W2 DeepSeek wire** | `2.0.0-beta.N` | Default models DeepSeek; chat/edit loop live dogfood |
| **W3 L1/L2 overlay** | `2.0.0-beta.N` | Snippet + permissions + prefix discipline under real shell |
| **W4 Product cut** | **`2.0.0`** | §2 P0 green; README/npm point here; 1.x marked legacy |

### 1.x freeze policy (fail-close)

**Allowed on 1.x** after replan merges:

- Critical install/auth bugs  
- Security fixes  
- Docs honesty (this replan family)

**Not allowed on 1.x** as “product progress”:

- New REPL UX features (banner/theme polish is optional vanity — do not block 2.0)  
- New MVP L3 shims presented as Grok-class  
- Tagging another “1.0 complete” narrative  

### PR unit principles (unchanged culture)

- Spec/ADR before large runtime moves  
- Small mergeable PRs; path-gated CI  
- No claiming gate green without evidence  
- **Do not** bump to `2.0.0` until W4 DoD  
- Parent runtime = parent family only (Grok session → `grok` children)

---

## 6. Ultragoal — one plate through 2.0.0

Previous A–D chain closed **scaffold** work (historical).  
**Product work is a single plan id:** **`grokbase-2x`** with **12 stories** (G001–G012).

| Artifact | Role |
|----------|------|
| [GROKBASE_2X_GOALS.md](./GROKBASE_2X_GOALS.md) | Story board + create-goals command |
| [ULTRAGOAL_PROMPT_COLD_START_2.0.md](./ULTRAGOAL_PROMPT_COLD_START_2.0.md) | Overnight paste prompt — run until 12/12 |
| Local ledger | `.omc/ultragoal/plans/grokbase-2x/goals.json` |

```text
G001 ReplanOnMain → G002 ADR-0008 → G003 W0 spike
  → G004–G006 W1 alpha → G007–G008 W2 beta
  → G009–G010 W3 overlays → G011–G012 cut 2.0.0
```

Do **not** restart `dogfood-0x` / `native-0x` / `throughput-0x` / `rc-1.0.0` as product SSOT.  
Do **not** split into a second product plan-id mid-train.

---

## 7. Communication plan

| Audience | Message |
|----------|---------|
| Owner | 1.x ≠ product; 2.0.0 is real goal; plan is Grok base |
| npm users of 1.x | Scaffold remains; breaking UX comes at 2.0.0 |
| Contributors | Stop extending thin REPL as if it were Grok; invest in base integration |

README top status (required — already wired in this replan PR):

> **Current npm `1.x` is a contract/scaffold line.**  
> **Product target: `2.0.0` — Grok Build–class agent (`dsb` opens full agent), DeepSeek-native, Deep Code + Reasonix overlays.**  
> See [REPLAN_2.0.md](./REPLAN_2.0.md).

---

## 8. Immediate next actions (ordered)

1. **Merge this replan** (docs + versioning honesty + WAVE_2x DAG).  
2. **ADR-0008: Grok Build base strategy** (A vs B) + license/attribution + `SOURCE_REV`.  
3. **W0 spike:** build/check `xai-grok-pager-bin` from local `../grok-build`; document run path; list provider/auth injection points in a research note under `docs/architecture/`.  
4. **Freeze feature creep on 1.x REPL** unless it unblocks 2.0 spike.  
5. **Ultragoal `grokbase-2x` create-goals** after ADR (or with W0).  

---

## 9. Success feeling (2.0.0)

> I type `dsb` (or `deepseek-build`).  
> A **Grok-class coding agent** opens.  
> It uses **DeepSeek**.  
> Edits are **safe**, long sessions stay **affordable**, work stays **fast**.  
> I do not need a subcommand to “enter” the product.

Until that is true, **do not call the product done** — regardless of checklist green or prior 1.0.0 tags.
